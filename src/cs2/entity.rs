//! Generic entity types that are not player pawns.

use serde::{Deserialize, Serialize};

/// High-level entity information for non-player objects in the game world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityInfo {
    /// Planted or carried bomb (C4).
    Bomb,
    /// Any throwable grenade projectile.
    Grenade(GrenadeType),
    /// Inferno created by a molotov or incendiary grenade.
    Inferno,
}

/// Classifies the specific type of a grenade entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrenadeType {
    /// Smoke grenade.
    Smoke,
    /// Flash-bang grenade.
    Flash,
    /// Molotov or incendiary grenade.
    Molotov,
    /// High-Explosive grenade.
    He,
    /// Decoy grenade.
    Decoy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_info_grenade_roundtrip() {
        let entity = EntityInfo::Grenade(GrenadeType::Smoke);
        let json = serde_json::to_string(&entity).expect("serialize");
        let back: EntityInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(entity, back);
    }
}
