//! Aimbot configuration panel.
use crate::ui::Config;

pub struct AimbotPanel {
    pub open: bool,
}

impl AimbotPanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn render(&mut self, ui: &imgui::Ui, config: &mut Config) {
        if !self.open {
            return;
        }
        ui.window("Aimbot")
            .size([300.0, 220.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.checkbox("Enable Aimbot", &mut config.aimbot_enabled);
                ui.separator();
                ui.slider("FOV", 1.0_f32, 90.0_f32, &mut config.aimbot_fov);
                ui.slider("Smooth", 1.0_f32, 20.0_f32, &mut config.aimbot_smooth);
                ui.slider("Lock Distance", 10.0_f32, 500.0_f32, &mut config.aimbot_lock_distance);
                ui.checkbox("Aim Prediction", &mut config.aimbot_aim_prediction);
            });
    }
}

impl Default for AimbotPanel {
    fn default() -> Self {
        Self::new()
    }
}
