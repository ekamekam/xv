//! Config/settings panel.
use crate::ui::Config;

pub struct ConfigPanel {
    pub open: bool,
}

impl ConfigPanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn render(&mut self, ui: &imgui::Ui, config: &mut Config) {
        if !self.open {
            return;
        }
        ui.window("Config")
            .size([320.0, 280.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text("Presets:");
                if ui.button("Conservative") {
                    *config = Config::preset_conservative();
                }
                ui.same_line();
                if ui.button("Balanced") {
                    *config = Config::preset_balanced();
                }
                ui.same_line();
                if ui.button("Aggressive") {
                    *config = Config::preset_aggressive();
                }
                ui.separator();
                ui.text("ESP Settings:");
                ui.checkbox("Show Health", &mut config.esp_show_health);
                ui.checkbox("Show Name", &mut config.esp_show_name);
                ui.checkbox("Show Weapon", &mut config.esp_show_weapon);
                ui.checkbox("Show Distance", &mut config.esp_show_distance);
                ui.checkbox("Show Skeleton", &mut config.esp_show_skeleton);
                ui.checkbox("Bounding Box", &mut config.esp_box);
                ui.separator();
                ui.slider("Opacity", 0.1_f32, 1.0_f32, &mut config.overlay_opacity);
            });
    }
}

impl Default for ConfigPanel {
    fn default() -> Self {
        Self::new()
    }
}
