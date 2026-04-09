//! Integration tests for Phase 4 imgui backend.

use xv::data::Data;
use xv::imgui_backend::{overlay::OverlayConfig, ImGuiBackend};
use xv::ui::{Config, Notification, NotificationLevel, UIBackend, UIEvent};

// --- Config tests ---

#[test]
fn config_default_values() {
    let cfg = Config::default();
    assert!(cfg.esp_enabled);
    assert!(!cfg.aimbot_enabled);
    assert_eq!(cfg.aimbot_fov, 10.0);
    assert_eq!(cfg.frame_cap_fps, 144);
}

#[test]
fn config_preset_conservative() {
    let cfg = Config::preset_conservative();
    assert!(cfg.esp_enabled);
    assert!(!cfg.aimbot_enabled);
    assert_eq!(cfg.aimbot_fov, 5.0);
}

#[test]
fn config_preset_balanced() {
    let cfg = Config::preset_balanced();
    assert!(cfg.aimbot_enabled);
    assert_eq!(cfg.aimbot_fov, 10.0);
}

#[test]
fn config_preset_aggressive() {
    let cfg = Config::preset_aggressive();
    assert!(cfg.aimbot_enabled);
    assert!(cfg.triggerbot_enabled);
    assert!(cfg.aimbot_aim_prediction);
    assert_eq!(cfg.aimbot_fov, 25.0);
}

#[test]
fn config_round_trip_json() {
    let cfg = Config::preset_aggressive();
    let path = "test_config_roundtrip.json";
    cfg.save(path).unwrap();
    let loaded = Config::load_or_default(path);
    assert_eq!(loaded.aimbot_fov, cfg.aimbot_fov);
    assert_eq!(loaded.triggerbot_enabled, cfg.triggerbot_enabled);
    std::fs::remove_file(path).ok();
}

#[test]
fn config_load_missing_file_gives_default() {
    let loaded = Config::load_or_default("/nonexistent/path/config.json");
    let def = Config::default();
    assert_eq!(loaded.esp_enabled, def.esp_enabled);
    assert_eq!(loaded.aimbot_fov, def.aimbot_fov);
}

// --- UIEvent tests ---

#[test]
fn ui_event_clone() {
    let ev = UIEvent::MouseMove { x: 1.0, y: 2.0 };
    let ev2 = ev.clone();
    if let UIEvent::MouseMove { x, y } = ev2 {
        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
    } else {
        panic!("wrong variant");
    }
}

#[test]
fn notification_creation() {
    let n = Notification {
        message: "test".into(),
        level: NotificationLevel::Warning,
        ttl_seconds: 3.0,
    };
    assert_eq!(n.level, NotificationLevel::Warning);
    assert_eq!(n.ttl_seconds, 3.0);
}

// --- ImGuiBackend tests ---

use std::sync::{LazyLock, Mutex};
// imgui only allows one active context per process; serialize all backend tests.
static IMGUI_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn make_backend() -> ImGuiBackend {
    ImGuiBackend::new(
        OverlayConfig { width: 800, height: 600, ..Default::default() },
        Config::default(),
    )
}

#[test]
fn backend_not_ready_before_init() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let b = make_backend();
    assert!(!b.is_ready());
}

#[test]
fn backend_ready_after_init() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let mut b = make_backend();
    b.initialize().unwrap();
    assert!(b.is_ready());
}

#[test]
fn backend_set_display_size() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let mut b = make_backend();
    b.initialize().unwrap();
    b.set_display_size(1280.0, 720.0);
    // just ensure no panic
}

#[test]
fn backend_handle_input_events() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let mut b = make_backend();
    b.initialize().unwrap();
    b.handle_input(UIEvent::MouseMove { x: 100.0, y: 200.0 });
    b.handle_input(UIEvent::MouseButton { button: 0, pressed: true, x: 100.0, y: 200.0 });
    b.handle_input(UIEvent::MouseScroll { delta: 1.0 });
    b.handle_input(UIEvent::KeyPress { key: 65, modifiers: 0 });
    b.handle_input(UIEvent::KeyRelease { key: 65, modifiers: 0 });
    b.handle_input(UIEvent::Resize { width: 1920, height: 1080 });
}

#[test]
fn backend_render_frame() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let mut b = make_backend();
    b.initialize().unwrap();
    let data = Data::default();
    b.render(&data);
    b.render(&data);
}

#[test]
fn backend_notification_push_and_expire() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let mut b = make_backend();
    b.initialize().unwrap();
    b.push_notification(Notification {
        message: "hello".into(),
        level: NotificationLevel::Info,
        ttl_seconds: 0.001, // expires almost immediately
    });
    b.push_notification(Notification {
        message: "long-lived".into(),
        level: NotificationLevel::Error,
        ttl_seconds: 3600.0,
    });
    // Sleep briefly so first notif expires
    std::thread::sleep(std::time::Duration::from_millis(5));
    let data = Data::default();
    b.render(&data); // tick_notifications removes expired
}

#[test]
fn backend_with_defaults() {
    let _guard = IMGUI_LOCK.lock().unwrap();
    let mut b = ImGuiBackend::with_defaults();
    b.initialize().unwrap();
    assert!(b.is_ready());
    b.render(&Data::default());
}

// --- Mock tests using mockall ---

use mockall::mock;

mock! {
    pub TestBackend {}
    impl UIBackend for TestBackend {
        fn initialize(&mut self) -> Result<(), String>;
        fn begin_frame(&mut self);
        fn end_frame(&mut self);
        fn handle_input(&mut self, event: UIEvent);
        fn set_display_size(&mut self, width: f32, height: f32);
        fn is_ready(&self) -> bool;
        fn render(&mut self, data: &Data);
        fn push_notification(&mut self, notification: Notification);
    }
}

#[test]
fn mock_backend_initialize_called_once() {
    let mut mock = MockTestBackend::new();
    mock.expect_initialize()
        .times(1)
        .returning(|| Ok(()));
    mock.expect_is_ready()
        .times(1)
        .returning(|| true);

    mock.initialize().unwrap();
    assert!(mock.is_ready());
}

#[test]
fn mock_backend_render_called() {
    let mut mock = MockTestBackend::new();
    mock.expect_initialize().times(1).returning(|| Ok(()));
    mock.expect_render().times(2).return_const(());

    mock.initialize().unwrap();
    let data = Data::default();
    mock.render(&data);
    mock.render(&data);
}
