use serde::Serialize;

/// CS2 weapon identifiers.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub enum Weapon {
    #[default]
    Unknown,
    // Pistols
    Glock18,
    P2000,
    USP_S,
    Dualies,
    P250,
    Tec9,
    FiveSeven,
    CZ75,
    R8Revolver,
    Deagle,
    // SMGs
    Mac10,
    MP9,
    MP5SD,
    UMP45,
    PP_Bizon,
    P90,
    MP7,
    // Rifles
    FAMAS,
    Galil,
    AK47,
    M4A4,
    M4A1_S,
    SG553,
    AUG,
    SSG08,
    AWP,
    G3SG1,
    SCAR20,
    // Heavy
    Nova,
    XM1014,
    Sawedoff,
    MAG7,
    M249,
    Negev,
    // Grenades
    HEGrenade,
    Flashbang,
    SmokeGrenade,
    Molotov,
    IncGrenade,
    Decoy,
    // Equipment
    Knife,
    KnifeT,
    C4,
    Taser,
}

impl Weapon {
    /// Returns true if the weapon is a sniper rifle.
    pub fn is_sniper(&self) -> bool {
        matches!(self, Weapon::AWP | Weapon::SSG08 | Weapon::G3SG1 | Weapon::SCAR20)
    }

    /// Returns true if the weapon is a pistol.
    pub fn is_pistol(&self) -> bool {
        matches!(
            self,
            Weapon::Glock18
                | Weapon::P2000
                | Weapon::USP_S
                | Weapon::Dualies
                | Weapon::P250
                | Weapon::Tec9
                | Weapon::FiveSeven
                | Weapon::CZ75
                | Weapon::R8Revolver
                | Weapon::Deagle
        )
    }

    /// Returns true if the weapon is a grenade.
    pub fn is_grenade(&self) -> bool {
        matches!(
            self,
            Weapon::HEGrenade
                | Weapon::Flashbang
                | Weapon::SmokeGrenade
                | Weapon::Molotov
                | Weapon::IncGrenade
                | Weapon::Decoy
        )
    }

    /// Returns true if the weapon is a rifle.
    pub fn is_rifle(&self) -> bool {
        matches!(
            self,
            Weapon::FAMAS
                | Weapon::Galil
                | Weapon::AK47
                | Weapon::M4A4
                | Weapon::M4A1_S
                | Weapon::SG553
                | Weapon::AUG
                | Weapon::SSG08
                | Weapon::AWP
                | Weapon::G3SG1
                | Weapon::SCAR20
        )
    }

    /// Convert from CS2 item definition index.
    pub fn from_item_index(idx: u32) -> Self {
        match idx {
            1 => Weapon::Deagle,
            2 => Weapon::Dualies,
            3 => Weapon::FiveSeven,
            4 => Weapon::Glock18,
            7 => Weapon::AK47,
            8 => Weapon::AUG,
            9 => Weapon::AWP,
            10 => Weapon::FAMAS,
            11 => Weapon::G3SG1,
            13 => Weapon::Galil,
            14 => Weapon::M249,
            16 => Weapon::M4A4,
            17 => Weapon::Mac10,
            19 => Weapon::P90,
            23 => Weapon::MP5SD,
            24 => Weapon::UMP45,
            25 => Weapon::XM1014,
            26 => Weapon::PP_Bizon,
            27 => Weapon::MAG7,
            28 => Weapon::Negev,
            29 => Weapon::Sawedoff,
            30 => Weapon::Tec9,
            31 => Weapon::Taser,
            32 => Weapon::P2000,
            33 => Weapon::MP7,
            34 => Weapon::MP9,
            35 => Weapon::Nova,
            36 => Weapon::P250,
            38 => Weapon::SCAR20,
            39 => Weapon::SG553,
            40 => Weapon::SSG08,
            60 => Weapon::M4A1_S,
            61 => Weapon::USP_S,
            63 => Weapon::CZ75,
            64 => Weapon::R8Revolver,
            42 => Weapon::Knife,
            59 => Weapon::KnifeT,
            44 => Weapon::Flashbang,
            43 => Weapon::HEGrenade,
            45 => Weapon::SmokeGrenade,
            46 => Weapon::Molotov,
            47 => Weapon::Decoy,
            48 => Weapon::IncGrenade,
            49 => Weapon::C4,
            _ => Weapon::Unknown,
        }
    }
}
