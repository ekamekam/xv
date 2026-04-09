//! All CS2 weapon types plus helper methods.

use serde::{Deserialize, Serialize};

/// Every weapon / item available in CS2.
///
/// The `name()` method returns the in-game item key name; `is_grenade()` and
/// `is_sniper()` provide quick classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weapon {
    // ── Unknown / default ────────────────────────────────────────────────────
    #[default]
    Unknown,

    // ── Knives / melee ───────────────────────────────────────────────────────
    Knife,
    KnifeT,
    KnifeGhost,
    Bayonet,
    Bowie,
    Butterfly,
    Falchion,
    Flip,
    Gut,
    Huntsman,
    Karambit,
    M9Bayonet,
    Navaja,
    Nomad,
    Paracord,
    Shadow,
    Skeleton,
    Stiletto,
    Survival,
    Talon,
    Ursus,

    // ── Pistols ──────────────────────────────────────────────────────────────
    /// Glock-18 (T default).
    Glock,
    /// P2000 (CT default).
    P2000,
    /// USP-S.
    UspS,
    /// Dual Berettas.
    Elite,
    /// P250.
    P250,
    /// Tec-9.
    Tec9,
    /// Five-SeveN.
    FiveSeven,
    /// CZ75-Auto.
    Cz75,
    /// Desert Eagle.
    Deagle,
    /// R8 Revolver.
    Revolver,

    // ── SMGs ─────────────────────────────────────────────────────────────────
    /// MAC-10.
    Mac10,
    /// MP9.
    Mp9,
    /// MP5-SD.
    Mp5Sd,
    /// MP7.
    Mp7,
    /// UMP-45.
    Ump45,
    /// PP-Bizon.
    Bizon,
    /// P90.
    P90,

    // ── Heavy ────────────────────────────────────────────────────────────────
    /// Nova.
    Nova,
    /// XM1014.
    Xm1014,
    /// MAG-7.
    Mag7,
    /// Sawed-Off.
    SawedOff,
    /// M249.
    M249,
    /// Negev.
    Negev,

    // ── Rifles ───────────────────────────────────────────────────────────────
    /// FAMAS (CT).
    Famas,
    /// Galil AR (T).
    Galil,
    /// AK-47.
    Ak47,
    /// M4A4.
    M4A4,
    /// M4A1-S.
    M4A1S,
    /// SG 553.
    Sg553,
    /// AUG.
    Aug,
    /// SSG 08 (scout).
    Ssg08,
    /// AWP.
    Awp,
    /// G3SG1 (T auto-sniper).
    G3Sg1,
    /// SCAR-20 (CT auto-sniper).
    Scar20,

    // ── Grenades ─────────────────────────────────────────────────────────────
    /// High-Explosive grenade.
    HeGrenade,
    /// Flash-bang.
    Flashbang,
    /// Smoke grenade.
    SmokeGrenade,
    /// Molotov (T).
    Molotov,
    /// Incendiary grenade (CT).
    Incendiary,
    /// Decoy grenade.
    Decoy,

    // ── Other equipment ──────────────────────────────────────────────────────
    /// Bomb (C4).
    C4,
    /// Zeus x27 taser.
    Zeus,
    /// Breach charge.
    BreachCharge,
    /// Bump mine.
    BumpMine,
    /// Diversion device.
    Diversion,
    /// Frag grenade.
    FragGrenade,
    /// Tactical awareness grenade.
    Tagrenade,
    /// Shield.
    Shield,
}

impl Weapon {
    /// Returns `true` if this weapon is any kind of throwable grenade.
    #[inline]
    pub fn is_grenade(self) -> bool {
        matches!(
            self,
            Weapon::HeGrenade
                | Weapon::Flashbang
                | Weapon::SmokeGrenade
                | Weapon::Molotov
                | Weapon::Incendiary
                | Weapon::Decoy
                | Weapon::FragGrenade
                | Weapon::Tagrenade
        )
    }

    /// Returns `true` if this weapon is a bolt-action or semi-auto sniper rifle.
    #[inline]
    pub fn is_sniper(self) -> bool {
        matches!(
            self,
            Weapon::Awp | Weapon::Ssg08 | Weapon::G3Sg1 | Weapon::Scar20
        )
    }

    /// Returns the canonical in-game item key name for this weapon.
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Unknown => "unknown",
            Weapon::Knife => "weapon_knife",
            Weapon::KnifeT => "weapon_knife_t",
            Weapon::KnifeGhost => "weapon_knife_ghost",
            Weapon::Bayonet => "weapon_bayonet",
            Weapon::Bowie => "weapon_knife_survival_bowie",
            Weapon::Butterfly => "weapon_knife_butterfly",
            Weapon::Falchion => "weapon_knife_falchion",
            Weapon::Flip => "weapon_knife_flip",
            Weapon::Gut => "weapon_knife_gut",
            Weapon::Huntsman => "weapon_knife_tactical",
            Weapon::Karambit => "weapon_knife_karambit",
            Weapon::M9Bayonet => "weapon_knife_m9_bayonet",
            Weapon::Navaja => "weapon_knife_gypsy_jackknife",
            Weapon::Nomad => "weapon_knife_outdoor",
            Weapon::Paracord => "weapon_knife_cord",
            Weapon::Shadow => "weapon_knife_push",
            Weapon::Skeleton => "weapon_knife_skeleton",
            Weapon::Stiletto => "weapon_knife_stiletto",
            Weapon::Survival => "weapon_knife_canis",
            Weapon::Talon => "weapon_knife_widowmaker",
            Weapon::Ursus => "weapon_knife_ursus",
            Weapon::Glock => "weapon_glock",
            Weapon::P2000 => "weapon_hkp2000",
            Weapon::UspS => "weapon_usp_silencer",
            Weapon::Elite => "weapon_elite",
            Weapon::P250 => "weapon_p250",
            Weapon::Tec9 => "weapon_tec9",
            Weapon::FiveSeven => "weapon_fiveseven",
            Weapon::Cz75 => "weapon_cz75a",
            Weapon::Deagle => "weapon_deagle",
            Weapon::Revolver => "weapon_revolver",
            Weapon::Mac10 => "weapon_mac10",
            Weapon::Mp9 => "weapon_mp9",
            Weapon::Mp5Sd => "weapon_mp5sd",
            Weapon::Mp7 => "weapon_mp7",
            Weapon::Ump45 => "weapon_ump45",
            Weapon::Bizon => "weapon_bizon",
            Weapon::P90 => "weapon_p90",
            Weapon::Nova => "weapon_nova",
            Weapon::Xm1014 => "weapon_xm1014",
            Weapon::Mag7 => "weapon_mag7",
            Weapon::SawedOff => "weapon_sawedoff",
            Weapon::M249 => "weapon_m249",
            Weapon::Negev => "weapon_negev",
            Weapon::Famas => "weapon_famas",
            Weapon::Galil => "weapon_galilar",
            Weapon::Ak47 => "weapon_ak47",
            Weapon::M4A4 => "weapon_m4a1",
            Weapon::M4A1S => "weapon_m4a1_silencer",
            Weapon::Sg553 => "weapon_sg556",
            Weapon::Aug => "weapon_aug",
            Weapon::Ssg08 => "weapon_ssg08",
            Weapon::Awp => "weapon_awp",
            Weapon::G3Sg1 => "weapon_g3sg1",
            Weapon::Scar20 => "weapon_scar20",
            Weapon::HeGrenade => "weapon_hegrenade",
            Weapon::Flashbang => "weapon_flashbang",
            Weapon::SmokeGrenade => "weapon_smokegrenade",
            Weapon::Molotov => "weapon_molotov",
            Weapon::Incendiary => "weapon_incgrenade",
            Weapon::Decoy => "weapon_decoy",
            Weapon::C4 => "weapon_c4",
            Weapon::Zeus => "weapon_taser",
            Weapon::BreachCharge => "weapon_breachcharge",
            Weapon::BumpMine => "weapon_bumpmine",
            Weapon::Diversion => "weapon_diversion",
            Weapon::FragGrenade => "weapon_frag_grenade",
            Weapon::Tagrenade => "weapon_tagrenade",
            Weapon::Shield => "weapon_shield",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unknown() {
        assert_eq!(Weapon::default(), Weapon::Unknown);
    }

    #[test]
    fn grenade_classification() {
        assert!(Weapon::HeGrenade.is_grenade());
        assert!(Weapon::Flashbang.is_grenade());
        assert!(Weapon::Molotov.is_grenade());
        assert!(!Weapon::Ak47.is_grenade());
        assert!(!Weapon::Awp.is_grenade());
    }

    #[test]
    fn sniper_classification() {
        assert!(Weapon::Awp.is_sniper());
        assert!(Weapon::Ssg08.is_sniper());
        assert!(Weapon::G3Sg1.is_sniper());
        assert!(Weapon::Scar20.is_sniper());
        assert!(!Weapon::Ak47.is_sniper());
        assert!(!Weapon::Deagle.is_sniper());
    }

    #[test]
    fn name_not_empty() {
        assert!(!Weapon::Awp.name().is_empty());
        assert!(!Weapon::Unknown.name().is_empty());
    }
}
