//! Phase 4: UI Integration Layer.
//!
//! Defines the `UIBackend` trait and supporting types that decouple the
//! game logic from any specific UI framework implementation.

use crate::data::Data;

/// An input event from the user (keyboard or mouse).
#[derive(Debug, Clone)]
pub enum UIEvent {
    KeyPress { key: u32, modifiers: u32 },
    KeyRelease { key: u32, modifiers: u32 },
    MouseMove { x: f32, y: f32 },
    MouseButton { button: u8, pressed: bool, x: f32, y: f32 },
    MouseScroll { delta: f32 },
    Resize { width: u32, height: u32 },
    Close,
}

/// A notification message to display in the UI.
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub ttl_seconds: f32,
}

/// Severity of a notification.
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

/// Abstract UI backend that any framework implementation must satisfy.
pub trait UIBackend {
    fn initialize(&mut self) -> Result<(), String>;
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn handle_input(&mut self, event: UIEvent);
    fn set_display_size(&mut self, width: f32, height: f32);
    fn is_ready(&self) -> bool;
    fn render(&mut self, data: &Data);
    fn push_notification(&mut self, notification: Notification);
}

/// Configuration used by all UI panels.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub esp_enabled: bool,
    pub aimbot_enabled: bool,
    pub triggerbot_enabled: bool,
    pub autowall_enabled: bool,
    pub aimbot_fov: f32,
    pub aimbot_smooth: f32,
    pub aimbot_aim_prediction: bool,
    pub aimbot_lock_distance: f32,
    pub esp_show_health: bool,
    pub esp_show_name: bool,
    pub esp_show_weapon: bool,
    pub esp_show_distance: bool,
    pub esp_show_skeleton: bool,
    pub esp_box: bool,
    pub overlay_opacity: f32,
    pub frame_cap_fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            esp_enabled: true,
            aimbot_enabled: false,
            triggerbot_enabled: false,
            autowall_enabled: false,
            aimbot_fov: 10.0,
            aimbot_smooth: 5.0,
            aimbot_aim_prediction: false,
            aimbot_lock_distance: 100.0,
            esp_show_health: true,
            esp_show_name: true,
            esp_show_weapon: true,
            esp_show_distance: true,
            esp_show_skeleton: false,
            esp_box: true,
            overlay_opacity: 0.9,
            frame_cap_fps: 144,
        }
    }
}

impl Config {
    /// Load from a JSON file, falling back to defaults on any error.
    pub fn load_or_default(path: &str) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist this config to a JSON file.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("write: {e}"))
    }

    pub fn preset_conservative() -> Self {
        Self {
            esp_enabled: true,
            aimbot_enabled: false,
            aimbot_fov: 5.0,
            aimbot_smooth: 10.0,
            ..Default::default()
        }
    }

    pub fn preset_balanced() -> Self {
        Self {
            esp_enabled: true,
            aimbot_enabled: true,
            aimbot_fov: 10.0,
            aimbot_smooth: 5.0,
            ..Default::default()
        }
    }

    pub fn preset_aggressive() -> Self {
        Self {
            esp_enabled: true,
            aimbot_enabled: true,
            triggerbot_enabled: true,
            aimbot_fov: 25.0,
            aimbot_smooth: 1.0,
            aimbot_aim_prediction: true,
            ..Default::default()
        }
    }
}
