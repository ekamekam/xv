use serde::Serialize;

/// Skeletal bone identifiers for CS2 player models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Bones {
    Head,
    Neck,
    Spine3,
    Spine2,
    Spine1,
    Spine0,
    Pelvis,
    UpperArmLeft,
    LowerArmLeft,
    HandLeft,
    UpperArmRight,
    LowerArmRight,
    HandRight,
    ThighLeft,
    KneeLeft,
    AnkleLeft,
    ThighRight,
    KneeRight,
    AnkleRight,
}

impl Bones {
    /// Returns all bones that are considered hitbox targets.
    pub fn hitbox_bones() -> &'static [Bones] {
        &[
            Bones::Head,
            Bones::Neck,
            Bones::Spine3,
            Bones::Spine2,
            Bones::Spine1,
            Bones::Spine0,
            Bones::Pelvis,
            Bones::UpperArmLeft,
            Bones::LowerArmLeft,
            Bones::UpperArmRight,
            Bones::LowerArmRight,
            Bones::ThighLeft,
            Bones::KneeLeft,
            Bones::ThighRight,
            Bones::KneeRight,
        ]
    }

    /// Returns the bone index used in the CS2 skeleton array.
    pub fn index(&self) -> usize {
        match self {
            Bones::Head => 6,
            Bones::Neck => 5,
            Bones::Spine3 => 4,
            Bones::Spine2 => 3,
            Bones::Spine1 => 2,
            Bones::Spine0 => 1,
            Bones::Pelvis => 0,
            Bones::UpperArmLeft => 8,
            Bones::LowerArmLeft => 9,
            Bones::HandLeft => 10,
            Bones::UpperArmRight => 13,
            Bones::LowerArmRight => 14,
            Bones::HandRight => 15,
            Bones::ThighLeft => 22,
            Bones::KneeLeft => 23,
            Bones::AnkleLeft => 24,
            Bones::ThighRight => 25,
            Bones::KneeRight => 26,
            Bones::AnkleRight => 27,
        }
    }
}
