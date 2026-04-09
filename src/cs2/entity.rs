use glam::Vec3;
use serde::Serialize;

/// Types of generic game entities tracked by the reader.
#[derive(Debug, Clone, Serialize)]
pub enum EntityInfo {
    Bomb { position: Vec3 },
    Grenade { grenade_type: GrenadeType, position: Vec3 },
    Inferno { position: Vec3 },
}

/// Grenade sub-types.
#[derive(Debug, Clone, Serialize)]
pub enum GrenadeType {
    Smoke,
    Flash,
    HE,
    Molotov,
    Incendiary,
    Decoy,
}