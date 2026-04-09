//! Integration tests for Phase 3: UI Integration Layer.
//!
//! Tests cover:
//! - State machine transitions
//! - Config serialization (TOML + JSON round-trips)
//! - Feature flag toggling
//! - Event dispatching
//! - UIBackend trait with mock implementation
//! - Error handling

use std::sync::{Arc, Mutex};

use xv::config::{AimbotConfig, Config, ConfigManager};
use xv::data::Data;
use xv::events::{EventDispatcher, GameEvent};
use xv::features::{Feature, FeatureFlags};
use xv::state::{AppState, StateMachine};
use xv::ui::traits::{InputEvent, UIBackend};

// ── Mock UIBackend ─────────────────────────────────────────────────────────────

#[derive(Default)]
struct MockBackend {
    frames: u32,
    inputs: Vec<String>,
    notes: Vec<String>,
    focused: bool,
}

impl UIBackend for MockBackend {
    fn render_frame(&mut self, _data: &Data) {
        self.frames += 1;
    }

    fn handle_input(&mut self, event: InputEvent) {
        self.inputs.push(format!("{event:?}"));
    }

    fn show_notification(&mut self, msg: &str) {
        self.notes.push(msg.to_owned());
    }

    fn is_window_focused(&self) -> bool {
        self.focused
    }
}

// ── UIBackend tests ──────────────────────────────────────────────────────────

#[test]
fn mock_backend_counts_frames() {
    let mut backend = MockBackend::default();
    let data = Data::default();
    for _ in 0..10 {
        backend.render_frame(&data);
    }
    assert_eq!(backend.frames, 10);
}

#[test]
fn mock_backend_stores_notifications() {
    let mut backend = MockBackend::default();
    backend.show_notification("hello");
    backend.show_notification("world");
    assert_eq!(backend.notes, vec!["hello", "world"]);
}

#[test]
fn mock_backend_records_input_events() {
    let mut backend = MockBackend::default();
    backend.handle_input(InputEvent::KeyPress { key_code: 42, pressed: true });
    backend.handle_input(InputEvent::MouseButton { button: 0, pressed: false });
    assert_eq!(backend.inputs.len(), 2);
}

#[test]
fn mock_backend_focus_state() {
    let backend = MockBackend { focused: true, ..Default::default() };
    assert!(backend.is_window_focused());
}

// ── State machine tests ──────────────────────────────────────────────────────

#[test]
fn state_machine_starts_initializing() {
    let sm = StateMachine::new();
    assert_eq!(sm.state(), &AppState::Initializing);
    assert!(!sm.is_running());
    assert!(sm.is_active());
}

#[test]
fn state_machine_full_forward_path() {
    let mut sm = StateMachine::new();
    sm.transition(AppState::Connected).expect("Initializing → Connected");
    sm.transition(AppState::Running).expect("Connected → Running");
    assert!(sm.is_running());
    sm.transition(AppState::Paused).expect("Running → Paused");
    assert!(!sm.is_running());
    sm.transition(AppState::Running).expect("Paused → Running");
    sm.transition(AppState::Disconnected).expect("Running → Disconnected");
    assert!(!sm.is_active());
}

#[test]
fn state_machine_invalid_transition_returns_err() {
    let mut sm = StateMachine::new();
    assert!(sm.transition(AppState::Running).is_err());
    assert!(sm.transition(AppState::Paused).is_err());
}

#[test]
fn state_machine_error_from_any_state() {
    for start in [AppState::Initializing, AppState::Connected, AppState::Running, AppState::Paused] {
        let mut sm = StateMachine::with_state(start);
        sm.transition(AppState::Error("oops".into())).expect("should allow → Error");
    }
}

#[test]
fn state_machine_disconnected_can_reinit() {
    let mut sm = StateMachine::with_state(AppState::Disconnected);
    sm.transition(AppState::Initializing).expect("Disconnected → Initializing");
}

#[test]
fn state_machine_display() {
    assert_eq!(format!("{}", AppState::Running),       "Running");
    assert_eq!(format!("{}", AppState::Disconnected),  "Disconnected");
    assert_eq!(format!("{}", AppState::Error("x".into())), "Error(x)");
}

// ── Feature flag tests ───────────────────────────────────────────────────────

#[test]
fn feature_flags_default_only_esp() {
    let f = FeatureFlags::default();
    assert!(f.is_enabled(Feature::Esp));
    assert!(!f.is_enabled(Feature::Aimbot));
    assert!(!f.is_enabled(Feature::Triggerbot));
    assert!(!f.is_enabled(Feature::Autowall));
    assert!(!f.is_enabled(Feature::Backtrack));
}

#[test]
fn feature_flags_toggle_all() {
    let mut f = FeatureFlags::all_disabled();
    for feat in [Feature::Aimbot, Feature::Esp, Feature::Triggerbot, Feature::Autowall, Feature::Backtrack] {
        f.set(feat, true);
        assert!(f.is_enabled(feat));
        f.set(feat, false);
        assert!(!f.is_enabled(feat));
    }
}

#[test]
fn feature_flags_serde_round_trip() {
    let mut f = FeatureFlags::default();
    f.set(Feature::Aimbot, true);
    let json = serde_json::to_string(&f).unwrap();
    let back: FeatureFlags = serde_json::from_str(&json).unwrap();
    assert!(back.is_enabled(Feature::Aimbot));
    assert!(back.is_enabled(Feature::Esp));
}

// ── Config tests ─────────────────────────────────────────────────────────────

#[test]
fn config_default_has_sensible_values() {
    let cfg = Config::default();
    assert!(cfg.esp.boxes);
    assert!(cfg.aimbot.visible_only);
    assert!((cfg.display.opacity - 0.9).abs() < f32::EPSILON);
}

#[test]
fn config_toml_round_trip() {
    let mut cfg = Config::default();
    cfg.aimbot = AimbotConfig { fov: 7.5, smoothing: 0.3, ..AimbotConfig::default() };
    let toml_str = toml::to_string_pretty(&cfg).expect("toml serialise");
    let back: Config = toml::from_str(&toml_str).expect("toml deserialise");
    assert!((back.aimbot.fov - 7.5).abs() < f32::EPSILON);
    assert!((back.aimbot.smoothing - 0.3).abs() < f32::EPSILON);
}

#[test]
fn config_json_round_trip() {
    let mut cfg = Config::default();
    cfg.esp.max_distance = 500.0;
    let json = serde_json::to_string(&cfg).expect("json serialise");
    let back: Config = serde_json::from_str(&json).expect("json deserialise");
    assert!((back.esp.max_distance - 500.0).abs() < f32::EPSILON);
}

#[test]
fn config_manager_save_load_toml() {
    let mut mgr = ConfigManager::new();
    mgr.config_mut().aimbot.fov = 9.0;
    let path = std::env::temp_dir().join("xv_ui_test.toml");
    mgr.save(&path).expect("save toml");

    let mut mgr2 = ConfigManager::new();
    mgr2.load(&path).expect("load toml");
    assert!((mgr2.config().aimbot.fov - 9.0).abs() < f32::EPSILON);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn config_manager_save_load_json() {
    let mut mgr = ConfigManager::new();
    mgr.config_mut().esp.names = false;
    let path = std::env::temp_dir().join("xv_ui_test.json");
    mgr.save(&path).expect("save json");

    let mut mgr2 = ConfigManager::new();
    mgr2.load(&path).expect("load json");
    assert!(!mgr2.config().esp.names);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn config_manager_unknown_extension() {
    let mgr = ConfigManager::new();
    assert!(mgr.save(std::path::Path::new("config.xyz")).is_err());
    let mut mgr2 = ConfigManager::new();
    // Write an actual file to test the load path
    let bad_path = std::env::temp_dir().join("xv_ui_test_bad.xyz");
    std::fs::write(&bad_path, b"data").unwrap();
    assert!(mgr2.load(&bad_path).is_err());
    let _ = std::fs::remove_file(&bad_path);
}

// ── Event dispatcher tests ───────────────────────────────────────────────────

#[test]
fn event_dispatcher_subscribes_and_emits() {
    let mut d = EventDispatcher::new();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let log_clone = Arc::clone(&log);

    d.subscribe(move |event| {
        log_clone.lock().unwrap().push(format!("{event:?}"));
    });

    d.emit(GameEvent::PlayerDetected(1));
    d.emit(GameEvent::PlayerLost(1));
    assert_eq!(log.lock().unwrap().len(), 2);
}

#[test]
fn event_dispatcher_unsubscribe_stops_events() {
    let mut d = EventDispatcher::new();
    let count = Arc::new(Mutex::new(0u32));
    let c = Arc::clone(&count);
    let id = d.subscribe(move |_| { *c.lock().unwrap() += 1; });

    d.emit(GameEvent::PlayerDetected(99));
    assert_eq!(*count.lock().unwrap(), 1);

    d.unsubscribe(id);
    d.emit(GameEvent::PlayerDetected(99));
    assert_eq!(*count.lock().unwrap(), 1); // no change
}

#[test]
fn event_dispatcher_clear_removes_all_subscribers() {
    let mut d = EventDispatcher::new();
    d.subscribe(|_| {});
    d.subscribe(|_| {});
    d.subscribe(|_| {});
    assert_eq!(d.subscriber_count(), 3);
    d.clear_subscribers();
    assert_eq!(d.subscriber_count(), 0);
}

#[test]
fn event_state_changed_carries_both_states() {
    let mut d = EventDispatcher::new();
    let result: Arc<Mutex<Option<(AppState, AppState)>>> = Arc::new(Mutex::new(None));
    let r = Arc::clone(&result);

    d.subscribe(move |event| {
        if let GameEvent::StateChanged { from, to } = event {
            *r.lock().unwrap() = Some((from.clone(), to.clone()));
        }
    });

    d.emit(GameEvent::StateChanged {
        from: AppState::Connected,
        to:   AppState::Running,
    });

    let guard = result.lock().unwrap();
    let (from, to) = guard.as_ref().unwrap();
    assert_eq!(from, &AppState::Connected);
    assert_eq!(to,   &AppState::Running);
}

#[test]
fn event_feature_toggled_carries_data() {
    let mut d = EventDispatcher::new();
    let saw: Arc<Mutex<Option<(Feature, bool)>>> = Arc::new(Mutex::new(None));
    let s = Arc::clone(&saw);

    d.subscribe(move |event| {
        if let GameEvent::FeatureToggled { feature, enabled } = event {
            *s.lock().unwrap() = Some((*feature, *enabled));
        }
    });

    d.emit(GameEvent::FeatureToggled { feature: Feature::Aimbot, enabled: true });
    assert_eq!(*saw.lock().unwrap(), Some((Feature::Aimbot, true)));
}

// ── Error handling tests ─────────────────────────────────────────────────────

#[test]
fn error_event_carries_message() {
    let mut d = EventDispatcher::new();
    let msg: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let m = Arc::clone(&msg);

    d.subscribe(move |event| {
        if let GameEvent::Error(s) = event {
            *m.lock().unwrap() = Some(s.clone());
        }
    });

    d.emit(GameEvent::Error("process disconnected".to_owned()));
    assert_eq!(msg.lock().unwrap().as_deref(), Some("process disconnected"));
}
