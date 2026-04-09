//! Player skeleton bone identifiers used for hit-detection and ESP rendering.

use serde::{Deserialize, Serialize};

/// Identifies a specific bone in a player's skeleton.
///
/// Use [`Bones::all()`] to iterate every tracked bone and
/// [`Bones::hitboxes()`] to get only the bones relevant to hit detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bones {
    // ── Head / neck ──────────────────────────────────────────────────────────
    Head,
    Neck,

    // ── Spine ────────────────────────────────────────────────────────────────
    Spine0,
    Spine1,
    Spine2,
    Spine3,

    // ── Pelvis ───────────────────────────────────────────────────────────────
    Pelvis,

    // ── Left arm ─────────────────────────────────────────────────────────────
    LeftShoulder,
    LeftElbow,
    LeftWrist,
    LeftHand,

    // ── Right arm ────────────────────────────────────────────────────────────
    RightShoulder,
    RightElbow,
    RightWrist,
    RightHand,

    // ── Left leg ─────────────────────────────────────────────────────────────
    LeftHip,
    LeftKnee,
    LeftAnkle,
    LeftFoot,

    // ── Right leg ────────────────────────────────────────────────────────────
    RightHip,
    RightKnee,
    RightAnkle,
    RightFoot,
}

impl Bones {
    /// Returns every bone variant in a stable, deterministic order.
    pub fn all() -> &'static [Bones] {
        &[
            Bones::Head,
            Bones::Neck,
            Bones::Spine0,
            Bones::Spine1,
            Bones::Spine2,
            Bones::Spine3,
            Bones::Pelvis,
            Bones::LeftShoulder,
            Bones::LeftElbow,
            Bones::LeftWrist,
            Bones::LeftHand,
            Bones::RightShoulder,
            Bones::RightElbow,
            Bones::RightWrist,
            Bones::RightHand,
            Bones::LeftHip,
            Bones::LeftKnee,
            Bones::LeftAnkle,
            Bones::LeftFoot,
            Bones::RightHip,
            Bones::RightKnee,
            Bones::RightAnkle,
            Bones::RightFoot,
        ]
    }

    /// Returns the subset of bones that correspond to hittable hitboxes.
    ///
    /// These are the bones typically checked during aimbot / wallbang
    /// calculations.
    pub fn hitboxes() -> &'static [Bones] {
        &[
            Bones::Head,
            Bones::Neck,
            Bones::Spine3,
            Bones::Spine1,
            Bones::Pelvis,
        ]
    }

    /// Returns a human-readable name suitable for logging or debug UI.
    pub fn name(self) -> &'static str {
        match self {
            Bones::Head => "head",
            Bones::Neck => "neck",
            Bones::Spine0 => "spine_0",
            Bones::Spine1 => "spine_1",
            Bones::Spine2 => "spine_2",
            Bones::Spine3 => "spine_3",
            Bones::Pelvis => "pelvis",
            Bones::LeftShoulder => "left_shoulder",
            Bones::LeftElbow => "left_elbow",
            Bones::LeftWrist => "left_wrist",
            Bones::LeftHand => "left_hand",
            Bones::RightShoulder => "right_shoulder",
            Bones::RightElbow => "right_elbow",
            Bones::RightWrist => "right_wrist",
            Bones::RightHand => "right_hand",
            Bones::LeftHip => "left_hip",
            Bones::LeftKnee => "left_knee",
            Bones::LeftAnkle => "left_ankle",
            Bones::LeftFoot => "left_foot",
            Bones::RightHip => "right_hip",
            Bones::RightKnee => "right_knee",
            Bones::RightAnkle => "right_ankle",
            Bones::RightFoot => "right_foot",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_head() {
        assert!(Bones::all().contains(&Bones::Head));
    }

    #[test]
    fn hitboxes_subset_of_all() {
        let all: std::collections::HashSet<_> = Bones::all().iter().copied().collect();
        for bone in Bones::hitboxes() {
            assert!(all.contains(bone), "{:?} not in all()", bone);
        }
    }

    #[test]
    fn name_not_empty() {
        for bone in Bones::all() {
            assert!(!bone.name().is_empty(), "{:?} has empty name", bone);
        }
    }
}
