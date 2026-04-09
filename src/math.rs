//! Framework-agnostic math utilities for CS2 game logic.
//!
//! All functions work with primitive Rust/glam types — there are no dependencies
//! on any UI framework (egui, winit, etc.).

use glam::{Mat4, Vec3};

/// Projects a 3D world-space position onto a 2D screen.
///
/// Returns `Some((x, y))` in pixel coordinates if the point is in front of the
/// camera, or `None` if it is behind the view plane.
///
/// # Parameters
/// * `world_pos`     – Position in world space.
/// * `view_matrix`   – Combined view-projection matrix (VP or MVP).
/// * `window_pos`    – Top-left corner of the game window in screen space.
/// * `window_size`   – Width and height of the game window in pixels.
pub fn world_to_screen(
    world_pos: Vec3,
    view_matrix: Mat4,
    window_pos: (f32, f32),
    window_size: (f32, f32),
) -> Option<(f32, f32)> {
    let clip = view_matrix * world_pos.extend(1.0);

    // Behind the near plane — do not project.
    if clip.w < 0.0001 {
        return None;
    }

    // Normalised device coordinates.
    let ndc_x = clip.x / clip.w;
    let ndc_y = clip.y / clip.w;

    let x = window_pos.0 + (1.0 + ndc_x) * 0.5 * window_size.0;
    let y = window_pos.1 + (1.0 - ndc_y) * 0.5 * window_size.1;

    Some((x, y))
}

/// Converts a normalised forward direction vector into `(pitch, yaw)` in
/// **degrees**.
///
/// * `pitch` is the vertical angle: positive = looking down, negative = up.
/// * `yaw`   is the horizontal angle in the range `[-180, 180]`.
pub fn angles_from_vector(forward: Vec3) -> (f32, f32) {
    let pitch = -forward.z.asin().to_degrees();
    let yaw = forward.y.atan2(forward.x).to_degrees();
    (pitch, yaw)
}

/// Calculates the angular distance (FOV) between two `(pitch, yaw)` pairs
/// (in degrees).
///
/// The result is the absolute angular separation and is always ≥ 0.
pub fn angles_to_fov(from: (f32, f32), to: (f32, f32)) -> f32 {
    let d_pitch = (to.0 - from.0).to_radians();
    let d_yaw = delta_yaw(from.1, to.1).to_radians();
    (d_pitch * d_pitch + d_yaw * d_yaw).sqrt().to_degrees()
}

/// Clamps a `(pitch, yaw)` pair to the ranges CS2 considers valid:
/// * `pitch` ∈ `[-89, 89]`
/// * `yaw`   is normalised to `(-180, 180]`
pub fn vec2_clamp(angles: (f32, f32)) -> (f32, f32) {
    let pitch = angles.0.clamp(-89.0, 89.0);
    let yaw = normalise_yaw(angles.1);
    (pitch, yaw)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Normalises a yaw value to the range `(-180, 180]`.
fn normalise_yaw(yaw: f32) -> f32 {
    let mut y = yaw;
    while y > 180.0 {
        y -= 360.0;
    }
    while y <= -180.0 {
        y += 360.0;
    }
    y
}

/// Signed shortest angular difference between two yaw values (degrees).
fn delta_yaw(from: f32, to: f32) -> f32 {
    normalise_yaw(to - from)
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Mat4;

    const EPS: f32 = 1e-4;

    // ── world_to_screen ───────────────────────────────────────────────────────

    #[test]
    fn world_to_screen_behind_camera_is_none() {
        // A clip.w < 0 means the point is behind the near plane.
        // Build a trivial matrix that pushes w negative for the test point.
        let m = Mat4::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, -1.0, // w = -z
        ]);
        // Point at z=1 → w = -1 → behind plane.
        let result = world_to_screen(Vec3::new(0.0, 0.0, 1.0), m, (0.0, 0.0), (1920.0, 1080.0));
        assert!(result.is_none());
    }

    #[test]
    fn world_to_screen_centre_maps_to_centre() {
        // Identity-like matrix where w stays 1 and x,y stay 0 for origin.
        let m = Mat4::IDENTITY;
        // Vec3::ZERO → clip = (0, 0, 0, 1) → ndc (0, 0) → window centre.
        let result = world_to_screen(Vec3::ZERO, m, (0.0, 0.0), (1920.0, 1080.0));
        assert!(result.is_some());
        let (x, y) = result.unwrap();
        assert!((x - 960.0).abs() < EPS, "x={x}");
        assert!((y - 540.0).abs() < EPS, "y={y}");
    }

    // ── angles_from_vector ────────────────────────────────────────────────────

    #[test]
    fn angles_from_vector_forward_is_zero() {
        let (pitch, yaw) = angles_from_vector(Vec3::new(1.0, 0.0, 0.0));
        assert!(pitch.abs() < EPS, "pitch={pitch}");
        assert!(yaw.abs() < EPS, "yaw={yaw}");
    }

    #[test]
    fn angles_from_vector_up_gives_negative_pitch() {
        // Looking straight up: forward = +Z.
        let (pitch, _yaw) = angles_from_vector(Vec3::new(0.0, 0.0, 1.0));
        assert!(pitch < 0.0, "expected negative pitch for upward vector, got {pitch}");
    }

    #[test]
    fn angles_from_vector_right_gives_90_yaw() {
        // Looking right: forward = +Y in CS2 convention.
        let (pitch, yaw) = angles_from_vector(Vec3::new(0.0, 1.0, 0.0));
        assert!(pitch.abs() < EPS);
        assert!((yaw - 90.0).abs() < EPS, "yaw={yaw}");
    }
    let sx = (1.0 + clip.x) * 0.5 * width;
    let sy = (1.0 - clip.y) * 0.5 * height;
    Some((sx, sy))
}

    // ── angles_to_fov ────────────────────────────────────────────────────────

    #[test]
    fn angles_to_fov_identity_is_zero() {
        let fov = angles_to_fov((0.0, 0.0), (0.0, 0.0));
        assert!(fov.abs() < EPS, "fov={fov}");
    }

    #[test]
    fn angles_to_fov_pure_pitch() {
        // 45° pitch difference should yield ~45° FOV.
        let fov = angles_to_fov((0.0, 0.0), (45.0, 0.0));
        assert!((fov - 45.0).abs() < EPS, "fov={fov}");
    }

    #[test]
    fn angles_to_fov_yaw_wraps() {
        // 179° and -179° are only 2° apart.
        let fov = angles_to_fov((0.0, 179.0), (0.0, -179.0));
        assert!(fov < 3.0, "expected small fov for near-antipodal yaw, got {fov}");
    }

    // ── vec2_clamp ────────────────────────────────────────────────────────────

    #[test]
    fn vec2_clamp_limits_pitch() {
        let (pitch, _) = vec2_clamp((120.0, 0.0));
        assert!((pitch - 89.0).abs() < EPS);
    }

    #[test]
    fn vec2_clamp_normalises_yaw() {
        let (_, yaw) = vec2_clamp((0.0, 270.0));
        assert!((yaw - (-90.0)).abs() < EPS, "yaw={yaw}");
    }

    #[test]
    fn vec2_clamp_identity() {
        let (pitch, yaw) = vec2_clamp((45.0, 90.0));
        assert!((pitch - 45.0).abs() < EPS);
        assert!((yaw - 90.0).abs() < EPS);
    }
}
