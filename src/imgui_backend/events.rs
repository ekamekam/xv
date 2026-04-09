//! Event handling for the imgui backend.
use crate::ui::{Notification, NotificationLevel, UIEvent};
use std::time::Instant;

pub struct EventHandler {
    pub quit_requested: bool,
}

impl EventHandler {
    pub fn new() -> Self {
        Self { quit_requested: false }
    }

    pub fn handle(&mut self, io: &mut imgui::Io, event: UIEvent) {
        match event {
            UIEvent::MouseMove { x, y } => {
                io.mouse_pos = [x, y];
            }
            UIEvent::MouseButton { button, pressed, .. } => {
                if (button as usize) < io.mouse_down.len() {
                    io.mouse_down[button as usize] = pressed;
                }
            }
            UIEvent::MouseScroll { delta } => {
                io.mouse_wheel = delta;
            }
            UIEvent::KeyPress { key, .. } => {
                if (key as usize) < io.keys_down.len() {
                    io.keys_down[key as usize] = true;
                }
            }
            UIEvent::KeyRelease { key, .. } => {
                if (key as usize) < io.keys_down.len() {
                    io.keys_down[key as usize] = false;
                }
            }
            UIEvent::Resize { .. } => {
                // display_size is updated via set_display_size
            }
            UIEvent::Close => {
                self.quit_requested = true;
            }
        }
    }

    pub fn render_notifications(
        &self,
        ui: &imgui::Ui,
        notifications: &[(Notification, Instant)],
    ) {
        if notifications.is_empty() {
            return;
        }
        ui.window("##notifications")
            .size(
                [320.0, 20.0 * notifications.len() as f32 + 10.0],
                imgui::Condition::Always,
            )
            .position([10.0, 40.0], imgui::Condition::Always)
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .bg_alpha(0.75)
            .build(|| {
                for (n, _) in notifications {
                    let color = match n.level {
                        NotificationLevel::Info => [0.6, 0.9, 0.6, 1.0],
                        NotificationLevel::Warning => [1.0, 0.8, 0.2, 1.0],
                        NotificationLevel::Error => [1.0, 0.3, 0.3, 1.0],
                    };
                    ui.text_colored(color, &n.message);
                }
            });
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
