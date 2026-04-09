//! Persistent configuration management.
//!
//! [`Config`] bundles every user-facing setting.  [`ConfigManager`] handles
//! loading from TOML/JSON files and saving back to disk.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::features::FeatureFlags;

// ── Sub-configuration structs ─────────────────────────────────────────────────

/// Aimbot-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimbotConfig {
    /// Activation key code (e.g. right mouse button = 2).
    pub key: u32,
    /// Maximum angular distance (degrees) within which the aimbot activates.
    pub fov: f32,
    /// Smoothing factor; 1.0 = instant snap, < 1.0 = gradual movement.
    pub smoothing: f32,
    /// If `true`, prefer targeting the head; otherwise aim at centre-mass.
    pub head_only: bool,
    /// Require the target to be visible (not behind a wall).
    pub visible_only: bool,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            key:          2,
            fov:          5.0,
            smoothing:    0.5,
            head_only:    false,
            visible_only: true,
        }
    }
}

/// ESP (Extra Sensory Perception) visual overlay settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspConfig {
    /// Draw bounding boxes around players.
    pub boxes:         bool,
    /// Draw skeleton lines between bone joints.
    pub skeleton:      bool,
    /// Show player health bars.
    pub health_bars:   bool,
    /// Show player names above their heads.
    pub names:         bool,
    /// Show weapon names below the player.
    pub weapons:       bool,
    /// Show friendly-team players.
    pub show_friendly: bool,
    /// Maximum render distance in game units (0 = unlimited).
    pub max_distance:  f32,
}

impl Default for EspConfig {
    fn default() -> Self {
        Self {
            boxes:         true,
            skeleton:      true,
            health_bars:   true,
            names:         true,
            weapons:       false,
            show_friendly: false,
            max_distance:  0.0,
        }
    }
}

/// Miscellaneous boolean toggles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscConfig {
    /// Automatically accept Danger Zone / competitive match found screen.
    pub auto_accept:  bool,
    /// Reveal the bomb on the radar.
    pub radar_hack:   bool,
    /// Show how many players are spectating the local player.
    pub spectator_list: bool,
}

impl Default for MiscConfig {
    fn default() -> Self {
        Self {
            auto_accept:    false,
            radar_hack:     false,
            spectator_list: true,
        }
    }
}

/// UI display preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Overall UI opacity (0.0 transparent – 1.0 fully opaque).
    pub opacity: f32,
    /// Font size for on-screen text, in points.
    pub font_size: f32,
    /// Show the FPS / tick counter.
    pub show_fps: bool,
    /// Preferred config file format when saving: `"toml"` or `"json"`.
    pub format: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            opacity:   0.9,
            font_size: 13.0,
            show_fps:  false,
            format:    "toml".to_owned(),
        }
    }
}

// ── Top-level Config ──────────────────────────────────────────────────────────

/// Top-level configuration bundling every user-facing setting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub features: FeatureFlags,
    pub aimbot:   AimbotConfig,
    pub esp:      EspConfig,
    pub misc:     MiscConfig,
    pub display:  DisplayConfig,
}

// ── ConfigManager ─────────────────────────────────────────────────────────────

/// Loads and saves [`Config`] to disk in TOML or JSON format.
#[derive(Debug, Default)]
pub struct ConfigManager {
    config: Config,
}

/// Errors that can occur while loading or saving configuration.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    Json(serde_json::Error),
    UnknownFormat(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e)              => write!(f, "I/O error: {e}"),
            ConfigError::TomlDeserialize(e) => write!(f, "TOML parse error: {e}"),
            ConfigError::TomlSerialize(e)   => write!(f, "TOML serialize error: {e}"),
            ConfigError::Json(e)            => write!(f, "JSON error: {e}"),
            ConfigError::UnknownFormat(s)   => write!(f, "unknown config format: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self { ConfigError::Io(e) }
}

impl ConfigManager {
    /// Creates a `ConfigManager` with default settings.
    pub fn new() -> Self {
        Self { config: Config::default() }
    }

    /// Returns a reference to the current configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a mutable reference to the current configuration.
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Replaces the current configuration.
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Loads configuration from `path`.
    ///
    /// The format is inferred from the file extension (`.toml` → TOML,
    /// `.json` → JSON).
    pub fn load(&mut self, path: &Path) -> Result<(), ConfigError> {
        let raw = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        self.config = match ext.as_str() {
            "toml" => toml::from_str(&raw).map_err(ConfigError::TomlDeserialize)?,
            "json" => serde_json::from_str(&raw).map_err(ConfigError::Json)?,
            other  => return Err(ConfigError::UnknownFormat(other.to_owned())),
        };
        Ok(())
    }

    /// Saves the current configuration to `path`.
    ///
    /// The format is inferred from the file extension (`.toml` → TOML,
    /// `.json` → JSON).
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let content = match ext.as_str() {
            "toml" => toml::to_string_pretty(&self.config).map_err(ConfigError::TomlSerialize)?,
            "json" => serde_json::to_string_pretty(&self.config).map_err(ConfigError::Json)?,
            other  => return Err(ConfigError::UnknownFormat(other.to_owned())),
        };

        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let cfg = Config::default();
        assert!(cfg.esp.boxes);
        assert!(cfg.aimbot.visible_only);
        assert!((cfg.display.opacity - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn toml_round_trip() {
        let cfg = Config::default();
        let serialised = toml::to_string_pretty(&cfg).expect("serialise");
        let deserialised: Config = toml::from_str(&serialised).expect("deserialise");
        assert_eq!(deserialised.aimbot.fov, cfg.aimbot.fov);
        assert_eq!(deserialised.esp.boxes, cfg.esp.boxes);
    }

    #[test]
    fn json_round_trip() {
        let cfg = Config::default();
        let serialised = serde_json::to_string_pretty(&cfg).expect("serialise");
        let deserialised: Config = serde_json::from_str(&serialised).expect("deserialise");
        assert_eq!(deserialised.display.font_size, cfg.display.font_size);
    }

    #[test]
    fn save_and_load_toml() {
        let mut mgr = ConfigManager::new();
        mgr.config_mut().aimbot.fov = 12.5;

        let path = std::env::temp_dir().join("xv_test_config.toml");
        mgr.save(&path).expect("save toml");

        let mut mgr2 = ConfigManager::new();
        mgr2.load(&path).expect("load toml");
        assert!((mgr2.config().aimbot.fov - 12.5).abs() < f32::EPSILON);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_and_load_json() {
        let mut mgr = ConfigManager::new();
        mgr.config_mut().esp.max_distance = 1000.0;

        let path = std::env::temp_dir().join("xv_test_config.json");
        mgr.save(&path).expect("save json");

        let mut mgr2 = ConfigManager::new();
        mgr2.load(&path).expect("load json");
        assert!((mgr2.config().esp.max_distance - 1000.0).abs() < f32::EPSILON);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unknown_format_returns_error() {
        let mgr = ConfigManager::new();
        let result = mgr.save(Path::new("config.xyz"));
        assert!(matches!(result, Err(ConfigError::UnknownFormat(_))));
    }
}
