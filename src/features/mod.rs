//! Feature flags — enable/disable major functionality modules at runtime.
//!
//! [`FeatureFlags`] acts as the single source of truth for which subsystems
//! are active.  It can be serialised to/from config so settings persist across
//! sessions.

use serde::{Deserialize, Serialize};

/// Identifiers for each major feature that can be toggled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    Aimbot,
    Esp,
    Triggerbot,
    Autowall,
    Backtrack,
}

/// Runtime toggles for every major feature module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub aimbot_enabled:     bool,
    pub esp_enabled:        bool,
    pub triggerbot_enabled: bool,
    pub autowall_enabled:   bool,
    pub backtrack_enabled:  bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            aimbot_enabled:     false,
            esp_enabled:        true,
            triggerbot_enabled: false,
            autowall_enabled:   false,
            backtrack_enabled:  false,
        }
    }
}

impl FeatureFlags {
    /// Creates a new `FeatureFlags` with all features disabled.
    pub fn all_disabled() -> Self {
        Self {
            aimbot_enabled:     false,
            esp_enabled:        false,
            triggerbot_enabled: false,
            autowall_enabled:   false,
            backtrack_enabled:  false,
        }
    }

    /// Enables or disables `feature`.
    pub fn set(&mut self, feature: Feature, enabled: bool) {
        match feature {
            Feature::Aimbot     => self.aimbot_enabled     = enabled,
            Feature::Esp        => self.esp_enabled        = enabled,
            Feature::Triggerbot => self.triggerbot_enabled = enabled,
            Feature::Autowall   => self.autowall_enabled   = enabled,
            Feature::Backtrack  => self.backtrack_enabled  = enabled,
        }
    }

    /// Returns `true` if `feature` is currently enabled.
    pub fn is_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::Aimbot     => self.aimbot_enabled,
            Feature::Esp        => self.esp_enabled,
            Feature::Triggerbot => self.triggerbot_enabled,
            Feature::Autowall   => self.autowall_enabled,
            Feature::Backtrack  => self.backtrack_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flags_have_only_esp_enabled() {
        let f = FeatureFlags::default();
        assert!(f.is_enabled(Feature::Esp));
        assert!(!f.is_enabled(Feature::Aimbot));
        assert!(!f.is_enabled(Feature::Triggerbot));
        assert!(!f.is_enabled(Feature::Autowall));
        assert!(!f.is_enabled(Feature::Backtrack));
    }

    #[test]
    fn toggle_on_and_off() {
        let mut f = FeatureFlags::all_disabled();
        f.set(Feature::Aimbot, true);
        assert!(f.is_enabled(Feature::Aimbot));
        f.set(Feature::Aimbot, false);
        assert!(!f.is_enabled(Feature::Aimbot));
    }

    #[test]
    fn all_disabled_creates_all_off() {
        let f = FeatureFlags::all_disabled();
        for feat in [Feature::Aimbot, Feature::Esp, Feature::Triggerbot, Feature::Autowall, Feature::Backtrack] {
            assert!(!f.is_enabled(feat));
        }
    }
}
