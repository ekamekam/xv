//! Abstraction traits for UI frameworks.
//!
//! Any UI framework that wants to integrate with xv must implement
//! [`UIBackend`].  The optional [`Renderer`] and [`InputHandler`] traits
//! provide finer-grained extension points for frameworks that separate those
//! concerns.
//!
//! # Design goals
//! - Zero coupling to specific UI frameworks (egui, wgpu, winit, …).
//! - Swap implementations without changing core game-logic code.
//! - All trait objects can be stored in `Box<dyn …>`.

use crate::data::Data;

// ── InputEvent ────────────────────────────────────────────────────────────────

/// A normalised input event delivered to the UI backend.
///
/// Framework-specific events (e.g. `winit::event::WindowEvent`) should be
/// translated into this type before being passed to [`UIBackend::handle_input`].
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A keyboard key was pressed or released.
    KeyPress {
        /// Platform-specific key code.
        key_code: u32,
        pressed: bool,
    },
    /// A mouse button was pressed or released.
    MouseButton {
        /// 0 = left, 1 = right, 2 = middle.
        button: u32,
        pressed: bool,
    },
    /// The mouse cursor moved.
    MouseMove { x: f32, y: f32 },
    /// The mouse wheel was scrolled.
    MouseScroll { delta: f32 },
    /// The application window was resized.
    WindowResized { width: u32, height: u32 },
    /// The window focus changed.
    FocusChanged { focused: bool },
}

// ── UIBackend ─────────────────────────────────────────────────────────────────

/// Core interface that any UI framework must implement to integrate with xv.
///
/// # Example
///
/// ```rust,ignore
/// struct MyFramework { /* … */ }
///
/// impl UIBackend for MyFramework {
///     fn render_frame(&mut self, data: &Data) { /* draw overlays */ }
///     fn handle_input(&mut self, event: InputEvent) { /* handle keys */ }
///     fn show_notification(&mut self, msg: &str) { println!("[xv] {msg}"); }
///     fn is_window_focused(&self) -> bool { true }
/// }
/// ```
pub trait UIBackend {
    /// Draw the current overlay frame using the supplied game state snapshot.
    fn render_frame(&mut self, data: &Data);

    /// Process a single normalised input event.
    fn handle_input(&mut self, event: InputEvent);

    /// Display a short transient notification message to the user.
    fn show_notification(&mut self, msg: &str);

    /// Returns `true` if the overlay window currently has input focus.
    fn is_window_focused(&self) -> bool;
}

// ── Renderer ──────────────────────────────────────────────────────────────────

/// Optional advanced rendering interface.
///
/// Implement this in addition to [`UIBackend`] when your framework exposes a
/// draw-primitive API that xv can use for higher-quality overlays.
pub trait Renderer {
    /// Begin a new render pass.  Called once per frame before any draw calls.
    fn begin_frame(&mut self);

    /// Submit the completed frame to the display.  Called once per frame after
    /// all draw calls.
    fn end_frame(&mut self);

    /// Draw a 2D rectangle outline.
    fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [u8; 4]);

    /// Draw a filled 2D circle.
    fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: [u8; 4]);

    /// Draw a line segment between two 2D points.
    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [u8; 4]);

    /// Render a text string at the given position.
    fn draw_text(&mut self, x: f32, y: f32, text: &str, color: [u8; 4]);
}

// ── InputHandler ──────────────────────────────────────────────────────────────

/// Optional dedicated input-handling interface.
///
/// Implement this when the framework separates input polling from rendering.
pub trait InputHandler {
    /// Called once per frame to poll/flush pending input events.
    fn poll_events(&mut self) -> Vec<InputEvent>;

    /// Returns `true` if the given key code is currently held down.
    fn is_key_held(&self, key_code: u32) -> bool;

    /// Returns the current cursor position in screen-space pixels.
    fn cursor_position(&self) -> (f32, f32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Data;

    // ── Minimal mock backend ──────────────────────────────────────────────────

    #[derive(Default)]
    struct MockBackend {
        frames_rendered: u32,
        notifications:   Vec<String>,
        events_handled:  u32,
        focused:         bool,
    }

    impl UIBackend for MockBackend {
        fn render_frame(&mut self, _data: &Data) {
            self.frames_rendered += 1;
        }

        fn handle_input(&mut self, _event: InputEvent) {
            self.events_handled += 1;
        }

        fn show_notification(&mut self, msg: &str) {
            self.notifications.push(msg.to_owned());
        }

        fn is_window_focused(&self) -> bool {
            self.focused
        }
    }

    #[test]
    fn render_frame_increments_counter() {
        let mut backend = MockBackend::default();
        let data = Data::default();
        backend.render_frame(&data);
        backend.render_frame(&data);
        assert_eq!(backend.frames_rendered, 2);
    }

    #[test]
    fn handle_input_increments_counter() {
        let mut backend = MockBackend::default();
        backend.handle_input(InputEvent::KeyPress { key_code: 1, pressed: true });
        backend.handle_input(InputEvent::MouseButton { button: 0, pressed: false });
        assert_eq!(backend.events_handled, 2);
    }

    #[test]
    fn show_notification_stores_message() {
        let mut backend = MockBackend::default();
        backend.show_notification("Connected to CS2");
        assert_eq!(backend.notifications, vec!["Connected to CS2"]);
    }

    #[test]
    fn is_window_focused_reflects_state() {
        let mut backend = MockBackend { focused: true, ..Default::default() };
        assert!(backend.is_window_focused());
        backend.focused = false;
        assert!(!backend.is_window_focused());
    }

    // ── Minimal mock renderer ─────────────────────────────────────────────────

    #[derive(Default)]
    struct MockRenderer {
        began: u32,
        ended: u32,
        rects: u32,
    }

    impl Renderer for MockRenderer {
        fn begin_frame(&mut self) { self.began += 1; }
        fn end_frame(&mut self)   { self.ended += 1; }
        fn draw_rect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _c: [u8; 4]) { self.rects += 1; }
        fn draw_circle(&mut self, _cx: f32, _cy: f32, _r: f32, _c: [u8; 4]) {}
        fn draw_line(&mut self, _x0: f32, _y0: f32, _x1: f32, _y1: f32, _c: [u8; 4]) {}
        fn draw_text(&mut self, _x: f32, _y: f32, _t: &str, _c: [u8; 4]) {}
    }

    #[test]
    fn renderer_begin_end_symmetry() {
        let mut r = MockRenderer::default();
        r.begin_frame();
        r.draw_rect(0.0, 0.0, 100.0, 100.0, [255, 0, 0, 255]);
        r.end_frame();
        assert_eq!(r.began, 1);
        assert_eq!(r.ended, 1);
        assert_eq!(r.rects, 1);
    }
}
