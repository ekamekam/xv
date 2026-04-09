//! Performance metrics panel.
use crate::data::Data;

pub struct MetricsPanel {
    pub open: bool,
    frame_count: u64,
}

impl MetricsPanel {
    pub fn new() -> Self {
        Self { open: false, frame_count: 0 }
    }

    pub fn render(&mut self, ui: &imgui::Ui, data: &Data) {
        self.frame_count += 1;
        if !self.open {
            return;
        }
        ui.window("Metrics")
            .size([250.0, 160.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Frame: {}", self.frame_count));
                ui.text(format!("In-game: {}", data.in_game));
                ui.text(format!("Players: {}", data.players.len()));
                ui.text(format!("Ping: {}ms", data.ping));
                ui.text(format!("Map: {}", data.map_name));
            });
    }
}

impl Default for MetricsPanel {
    fn default() -> Self {
        Self::new()
    }
}
