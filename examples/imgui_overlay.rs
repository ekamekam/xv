//! imgui_overlay example — demonstrates the Phase 4 ImGui backend.
//!
//! In production this example would create a real overlay window.
//! In this sandbox build it demonstrates the API without a display.
#![allow(unused)]

use xv::data::Data;
use xv::imgui_backend::{overlay::OverlayConfig, ImGuiBackend};
use xv::ui::{Config, Notification, NotificationLevel, UIBackend, UIEvent};

fn main() {
    println!("=== xv imgui overlay demo ===");

    // Build backend
    let overlay_cfg = OverlayConfig {
        width: 1920,
        height: 1080,
        title: "xv overlay".into(),
        click_through: true,
        always_on_top: true,
    };
    let mut backend = ImGuiBackend::new(overlay_cfg, Config::default());

    // Initialize (stub — no real window)
    backend.initialize().expect("backend init failed");
    assert!(backend.is_ready());

    // Push a welcome notification
    backend.push_notification(Notification {
        message: "xv overlay started".into(),
        level: NotificationLevel::Info,
        ttl_seconds: 5.0,
    });

    // Simulate input events
    backend.handle_input(UIEvent::MouseMove { x: 960.0, y: 540.0 });
    backend.handle_input(UIEvent::Resize { width: 1920, height: 1080 });

    // Simulate 3 render frames
    let data = Data::default();
    for i in 0..3 {
        backend.render(&data);
        println!("Frame {} rendered", i + 1);
    }

    println!("Demo complete.");
}
