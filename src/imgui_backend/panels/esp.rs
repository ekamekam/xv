//! ESP panel — renders per-player overlays.
use crate::data::Data;
use crate::ui::Config;

pub struct ESPPanel {
    pub open: bool,
}

impl ESPPanel {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn render(&mut self, ui: &imgui::Ui, data: &Data, config: &Config) {
        if !self.open || !config.esp_enabled {
            return;
        }
        ui.window("ESP")
            .size([340.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Players visible: {}", data.players.len()));
                ui.separator();
                for player in &data.players {
                    if config.esp_show_health {
                        ui.text(format!("  {} | HP: {}", player.name, player.health));
                    } else {
                        ui.text(&player.name);
                    }
                    if config.esp_show_weapon {
                        ui.text(format!("    Weapon: {:?}", player.weapon));
                    }
                }
            });
    }
}

impl Default for ESPPanel {
    fn default() -> Self {
        Self::new()
    }
}
