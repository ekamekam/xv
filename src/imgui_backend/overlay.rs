//! Overlay / window management stub.
//!
//! In production this would create a transparent, click-through top-level
//! window using platform-specific APIs (e.g. `SetWindowLong` on Windows,
//! `_NET_WM_WINDOW_TYPE_DOCK` on X11).  In this sandbox build it is a no-op
//! stub that documents the intended interface.

/// Configuration for the overlay window.
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    /// Width of the overlay in pixels.
    pub width: u32,
    /// Height of the overlay in pixels.
    pub height: u32,
    /// Window title (may be used by the compositor).
    pub title: String,
    /// Whether the overlay should be click-through.
    pub click_through: bool,
    /// Whether the window should always stay on top.
    pub always_on_top: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            title: "xv overlay".to_string(),
            click_through: true,
            always_on_top: true,
        }
    }
}

/// Manages the native overlay window lifetime.
///
/// **Stub implementation** — replace with real windowing code (e.g. winit +
/// raw-window-handle) when system libraries are available.
pub struct OverlayManager {
    pub config: OverlayConfig,
    initialized: bool,
}

impl OverlayManager {
    pub fn new(config: OverlayConfig) -> Self {
        Self { config, initialized: false }
    }

    /// Create and show the overlay window.
    ///
    /// Returns `Err` if the platform is not supported or the window cannot be
    /// created.  The stub always returns `Ok(())`.
    pub fn initialize(&mut self) -> Result<(), String> {
        // TODO: create a transparent overlay window using platform APIs.
        self.initialized = true;
        Ok(())
    }

    /// Returns `true` after `initialize` has been called successfully.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Resize the overlay to match a new game window size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
    }

    /// Destroy the overlay window and free platform resources.
    pub fn shutdown(&mut self) {
        // TODO: destroy window handle, release GL context, etc.
        self.initialized = false;
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        if self.initialized {
            self.shutdown();
        }
    }
}
