//! Example: using Phase 3 UI Integration Layer with a custom UI framework.
//!
//! This example demonstrates how to implement [`UIBackend`] for a custom
//! framework and integrate it with [`GameStateManager`].
//!
//! Run with:
//! ```sh
//! cargo run --example ui_integration
//! ```
//!
//! No real CS2 process is required — the example uses the mock reader and
//! shows the wiring without actually reading memory.

use xv::config::ConfigManager;
use xv::data::Data;
use xv::events::GameEvent;
use xv::features::{Feature, FeatureFlags};
use xv::state::{AppState, StateMachine};
use xv::ui::traits::{InputEvent, Renderer, UIBackend};

// ── Mock UI backend ───────────────────────────────────────────────────────────

/// A minimal UI backend that logs calls to stdout — suitable for testing or
/// as a starting template for a real implementation.
struct ConsoleBackend {
    frame_count: u32,
    focused: bool,
    notifications: Vec<String>,
}

impl ConsoleBackend {
    fn new() -> Self {
        Self { frame_count: 0, focused: true, notifications: Vec::new() }
    }
}

impl UIBackend for ConsoleBackend {
    fn render_frame(&mut self, data: &Data) {
        self.frame_count += 1;
        println!(
            "[frame {:>4}] in_game={} players={} bomb={}",
            self.frame_count,
            data.in_game,
            data.players.len(),
            data.bomb.planted,
        );
    }

    fn handle_input(&mut self, event: InputEvent) {
        println!("[input] {event:?}");
    }

    fn show_notification(&mut self, msg: &str) {
        println!("[notification] {msg}");
        self.notifications.push(msg.to_owned());
    }

    fn is_window_focused(&self) -> bool {
        self.focused
    }
}

// ── Mock renderer ─────────────────────────────────────────────────────────────

struct ConsoleRenderer;

impl Renderer for ConsoleRenderer {
    fn begin_frame(&mut self) { println!("[renderer] begin_frame"); }
    fn end_frame(&mut self)   { println!("[renderer] end_frame"); }

    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, _color: [u8; 4]) {
        println!("[renderer] rect ({x},{y}) {w}×{h}");
    }

    fn draw_circle(&mut self, cx: f32, cy: f32, r: f32, _color: [u8; 4]) {
        println!("[renderer] circle ({cx},{cy}) r={r}");
    }

    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, _color: [u8; 4]) {
        println!("[renderer] line ({x0},{y0})→({x1},{y1})");
    }

    fn draw_text(&mut self, x: f32, y: f32, text: &str, _color: [u8; 4]) {
        println!("[renderer] text @ ({x},{y}): {text}");
    }
}

// ── Event handling example ────────────────────────────────────────────────────

fn demo_events() {
    use xv::events::EventDispatcher;

    println!("\n=== Event system demo ===");

    let mut dispatcher = EventDispatcher::new();
    dispatcher.subscribe(|event| {
        println!("[event] {event:?}");
    });

    dispatcher.emit(GameEvent::PlayerDetected(42));
    dispatcher.emit(GameEvent::FeatureToggled { feature: Feature::Esp, enabled: true });
    dispatcher.emit(GameEvent::Error("demo error".to_owned()));
    dispatcher.clear_subscribers();
    println!("Subscribers after clear: {}", dispatcher.subscriber_count());
}

// ── Config loading example ────────────────────────────────────────────────────

fn demo_config() {
    println!("\n=== Config demo ===");

    let mut mgr = ConfigManager::new();
    println!("Default FOV: {}", mgr.config().aimbot.fov);
    mgr.config_mut().aimbot.fov = 8.0;
    println!("Updated FOV: {}", mgr.config().aimbot.fov);

    let path = std::env::temp_dir().join("xv_example_config.toml");
    if let Err(e) = mgr.save(&path) {
        println!("Config save failed (expected outside real env): {e}");
    } else {
        println!("Config saved to {}", path.display());
        let _ = std::fs::remove_file(&path);
    }
}

// ── State machine example ─────────────────────────────────────────────────────

fn demo_state_machine() {
    println!("\n=== State machine demo ===");

    let mut sm = StateMachine::new();
    println!("Initial state: {}", sm.state());

    for next in [AppState::Connected, AppState::Running] {
        match sm.transition(next.clone()) {
            Ok(()) => println!("→ {}", sm.state()),
            Err(e) => println!("Transition error: {e}"),
        }
    }
    println!("is_running: {}", sm.is_running());

    sm.transition(AppState::Paused).unwrap();
    println!("Paused. is_running: {}", sm.is_running());
}

// ── Feature flags example ─────────────────────────────────────────────────────

fn demo_features() {
    println!("\n=== Feature flags demo ===");

    let mut flags = FeatureFlags::default();
    println!("ESP enabled (default): {}", flags.is_enabled(Feature::Esp));
    println!("Aimbot enabled (default): {}", flags.is_enabled(Feature::Aimbot));

    flags.set(Feature::Aimbot, true);
    println!("Aimbot enabled (after toggle): {}", flags.is_enabled(Feature::Aimbot));
}

// ── Simulated game loop ───────────────────────────────────────────────────────

fn demo_game_loop() {
    println!("\n=== Simulated game loop (5 frames) ===");

    let mut backend = ConsoleBackend::new();
    let mut renderer = ConsoleRenderer;
    let data = Data::default();

    backend.show_notification("xv overlay started");

    for _ in 0..5 {
        renderer.begin_frame();
        backend.render_frame(&data);
        // Simulate a key press event
        backend.handle_input(InputEvent::KeyPress { key_code: 0x49 /*Insert*/, pressed: true });
        renderer.draw_rect(10.0, 10.0, 200.0, 40.0, [255, 255, 255, 200]);
        renderer.draw_text(15.0, 20.0, "xv overlay", [255, 255, 255, 255]);
        renderer.end_frame();
    }

    println!("Total frames rendered: {}", backend.frame_count);
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    println!("=== xv Phase 3: UI Integration Layer demo ===\n");

    demo_state_machine();
    demo_features();
    demo_config();
    demo_events();
    demo_game_loop();

    println!("\nDemo complete. Integrate UIBackend with your framework to go live.");
}
