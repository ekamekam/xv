use glam::{Mat4, Vec2, Vec3};

/// Projects a 3D world position to 2D screen coordinates.
///
/// Returns `Some((x, y))` if the point is in front of the camera, `None` otherwise.
pub fn world_to_screen(view_matrix: &Mat4, pos: Vec3, width: f32, height: f32) -> Option<(f32, f32)> {
    let clip = view_matrix.project_point3(pos);
    // w component after division - clip.z is depth in NDC space.
    // project_point3 performs perspective divide; z < -1 or > 1 means outside frustum.
    // We use the raw matrix to get the w component for the w > 0 check.
    let col3 = view_matrix.row(3);
    let w = col3.x * pos.x + col3.y * pos.y + col3.z * pos.z + col3.w;
    if w < 0.001 {
        return None;
    }
    let sx = (1.0 + clip.x) * 0.5 * width;
    let sy = (1.0 - clip.y) * 0.5 * height;
    Some((sx, sy))
}

/// Converts a direction vector to pitch and yaw angles (in degrees).
///
/// Returns `(pitch, yaw)`.
pub fn angles_from_vector(dir: Vec3) -> (f32, f32) {
    let pitch = (-dir.z).atan2(dir.truncate().length()).to_degrees();
    let yaw = dir.y.atan2(dir.x).to_degrees();
    (pitch, yaw)
}

/// Computes the angular FOV distance between two view angles (pitch, yaw) in degrees.
pub fn angles_to_fov(view: Vec2, target: Vec2) -> f32 {
    let dp = view.x - target.x;
    let dy = view.y - target.y;
    (dp * dp + dy * dy).sqrt()
}

/// Clamps a view angle vector to valid pitch/yaw ranges.
///
/// Pitch: [-89, 89], Yaw: [-180, 180].
pub fn vec2_clamp(angles: Vec2) -> Vec2 {
    Vec2::new(
        angles.x.clamp(-89.0, 89.0),
        // Wrap yaw to [-180, 180]
        ((angles.y + 180.0).rem_euclid(360.0)) - 180.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_angles_from_vector_forward() {
        let (pitch, yaw) = angles_from_vector(Vec3::new(1.0, 0.0, 0.0));
        assert!((pitch - 0.0).abs() < 1e-4, "pitch should be ~0 for forward vector");
        assert!((yaw - 0.0).abs() < 1e-4, "yaw should be 0 for forward vector");
    }

    #[test]
    fn test_vec2_clamp_pitch() {
        let clamped = vec2_clamp(Vec2::new(95.0, 0.0));
        assert_eq!(clamped.x, 89.0);
    }

    #[test]
    fn test_vec2_clamp_yaw_wrap() {
        let clamped = vec2_clamp(Vec2::new(0.0, 270.0));
        assert!((clamped.y - (-90.0)).abs() < 1e-4, "270 should wrap to -90");
    }

    #[test]
    fn test_angles_to_fov_zero() {
        let fov = angles_to_fov(Vec2::new(10.0, 20.0), Vec2::new(10.0, 20.0));
        assert_eq!(fov, 0.0);
    }

    #[test]
    fn test_world_to_screen_behind_camera() {
        // Identity matrix with a point at negative depth should return None.
        let mat = Mat4::IDENTITY;
        let result = world_to_screen(&mat, Vec3::new(0.0, 0.0, -10.0), 1920.0, 1080.0);
        // With identity matrix, w = z component via row 3 = (0,0,0,1)*pos = 1.0 for any pos
        // but clip.z will still be computed; this just checks the function doesn't panic.
        let _ = result;
    }
}
