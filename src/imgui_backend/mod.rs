//! Phase 4: imgui-rs backend implementing the `UIBackend` trait.
//!
//! This module wires together the imgui `Context`, all UI panels, the menu bar,
//! and the event handler into a single [`ImGuiBackend`] that can be used from
//! the main render loop.
//!
//! # Window management
//! Native window creation (transparent overlay, click-through, always-on-top)
//! is handled by [`overlay::OverlayManager`] which is a **stub** in this build.
//! Swap that struct for a real implementation backed by `winit` + `raw-window-handle`
//! when the system libraries become available.

pub mod events;
pub mod menu;
pub mod overlay;
pub mod panels;

use std::time::Instant;

use crate::data::Data;
use crate::ui::{Config, Notification, UIBackend, UIEvent};

use events::EventHandler;
use menu::MainMenu;
use overlay::{OverlayConfig, OverlayManager};
use panels::{AimbotPanel, ConfigPanel, ESPPanel, MetricsPanel};

/// imgui-rs backend that implements [`UIBackend`].
pub struct ImGuiBackend {
    context: imgui::Context,
    overlay: OverlayManager,
    menu: MainMenu,
    esp_panel: ESPPanel,
    aimbot_panel: AimbotPanel,
    config_panel: ConfigPanel,
    metrics_panel: MetricsPanel,
    event_handler: EventHandler,
    notifications: Vec<(Notification, Instant)>,
    config: Config,
    display_size: [f32; 2],
    last_frame: Instant,
    initialized: bool,
}

impl ImGuiBackend {
    /// Create a new backend with the given overlay config and initial config.
    pub fn new(overlay_cfg: OverlayConfig, config: Config) -> Self {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        context.set_log_filename(None);

        let w = overlay_cfg.width as f32;
        let h = overlay_cfg.height as f32;
        {
            let io = context.io_mut();
            io.display_size = [w, h];
            io.delta_time = 1.0 / 60.0;
        }
        // Build the font atlas so that new_frame() sanity checks pass.
        context.fonts().build_rgba32_texture();

        Self {
            context,
            overlay: OverlayManager::new(overlay_cfg),
            menu: MainMenu::new(),
            esp_panel: ESPPanel::new(),
            aimbot_panel: AimbotPanel::new(),
            config_panel: ConfigPanel::new(),
            metrics_panel: MetricsPanel::new(),
            event_handler: EventHandler::new(),
            notifications: Vec::new(),
            config,
            display_size: [w, h],
            last_frame: Instant::now(),
            initialized: false,
        }
    }

    /// Convenience constructor with defaults.
    pub fn with_defaults() -> Self {
        Self::new(OverlayConfig::default(), Config::default())
    }

    /// Tick notifications — removes expired entries.
    fn tick_notifications(&mut self) {
        self.notifications.retain(|(n, start)| {
            start.elapsed().as_secs_f32() < n.ttl_seconds
        });
    }
}

impl UIBackend for ImGuiBackend {
    fn initialize(&mut self) -> Result<(), String> {
        self.overlay.initialize()?;
        self.initialized = true;
        Ok(())
    }

    fn begin_frame(&mut self) {
        let delta = self.last_frame.elapsed().as_secs_f32().max(1e-6);
        self.last_frame = Instant::now();
        let io = self.context.io_mut();
        io.delta_time = delta;
        io.display_size = self.display_size;
    }

    fn end_frame(&mut self) {
        // draw_data would be consumed by a renderer here; frame is managed by render()
    }

    fn handle_input(&mut self, event: UIEvent) {
        let io = self.context.io_mut();
        self.event_handler.handle(io, event);
    }

    fn set_display_size(&mut self, width: f32, height: f32) {
        self.display_size = [width, height];
        self.context.io_mut().display_size = [width, height];
        self.overlay.resize(width as u32, height as u32);
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    fn render(&mut self, data: &Data) {
        self.tick_notifications();

        // Update IO timing.
        let delta = self.last_frame.elapsed().as_secs_f32().max(1e-6);
        self.last_frame = Instant::now();
        self.context.io_mut().delta_time = delta;
        self.context.io_mut().display_size = self.display_size;

        // Clone config so we can mutate it inside the frame while `context`
        // is mutably borrowed through the `Ui` token.
        let mut cfg = self.config.clone();

        // Use a nested block so that all destructured borrows are dropped
        // before we write `self.config = cfg` below.
        {
            // Destructure to give the borrow checker visibility into disjoint
            // fields. `context.new_frame()` borrows only `context`; all other
            // panel fields are independent bindings.
            let ImGuiBackend {
                ref mut context,
                ref mut menu,
                ref mut esp_panel,
                ref mut aimbot_panel,
                ref mut config_panel,
                ref mut metrics_panel,
                ref mut event_handler,
                ref notifications,
                ..
            } = *self;

            {
                let ui = context.new_frame();

                menu.render(ui, &mut cfg, esp_panel, aimbot_panel, config_panel, metrics_panel);
                esp_panel.render(ui, data, &cfg);
                aimbot_panel.render(ui, &mut cfg);
                config_panel.render(ui, &mut cfg);
                metrics_panel.render(ui, data);
                event_handler.render_notifications(ui, notifications);
            }

            // Complete the frame (draw data would be sent to a renderer here).
            let _ = context.render();
        }

        // Apply any config mutations made inside the frame.
        self.config = cfg;
    }

    fn push_notification(&mut self, notification: Notification) {
        self.notifications.push((notification, Instant::now()));
    }
}
