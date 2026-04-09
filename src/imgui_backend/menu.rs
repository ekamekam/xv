//! Main menu bar.
use crate::ui::Config;
use crate::imgui_backend::panels::{AimbotPanel, ConfigPanel, ESPPanel, MetricsPanel};

pub struct MainMenu;

impl MainMenu {
    pub fn new() -> Self {
        Self
    }

    /// Render the menu bar, toggling panel visibility as needed.
    pub fn render(
        &mut self,
        ui: &imgui::Ui,
        config: &mut Config,
        esp: &mut ESPPanel,
        aimbot: &mut AimbotPanel,
        cfg_panel: &mut ConfigPanel,
        metrics: &mut MetricsPanel,
    ) {
        ui.window("xv Menu")
            .size([600.0, 30.0], imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .build(|| {
                ui.checkbox("ESP", &mut config.esp_enabled);
                ui.same_line();
                ui.checkbox("Aimbot", &mut config.aimbot_enabled);
                ui.same_line();
                ui.checkbox("Triggerbot", &mut config.triggerbot_enabled);
                ui.same_line();
                ui.separator();
                ui.same_line();
                if ui.button("ESP Panel") {
                    esp.open = !esp.open;
                }
                ui.same_line();
                if ui.button("Aimbot Panel") {
                    aimbot.open = !aimbot.open;
                }
                ui.same_line();
                if ui.button("Config") {
                    cfg_panel.open = !cfg_panel.open;
                }
                ui.same_line();
                if ui.button("Metrics") {
                    metrics.open = !metrics.open;
                }
            });
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}
