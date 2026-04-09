//! CS2 and ELF binary constants used throughout the library.

/// Counter-Strike 2 process and library names, team IDs, and entity/weapon
/// class name strings as they appear in the game's memory.
pub mod cs2 {
    /// Name of the CS2 process (without path).
    pub const PROCESS_NAME: &str = "cs2";

    /// Client-side shared library loaded by CS2.
    pub const CLIENT_LIB: &str = "libclient.so";

    /// Engine shared library.
    pub const ENGINE_LIB: &str = "libengine2.so";

    /// Tier-0 utility library.
    pub const TIER0_LIB: &str = "libtier0.so";

    // ── Teams ────────────────────────────────────────────────────────────────

    /// Spectator / unassigned team index.
    pub const TEAM_SPECTATOR: i32 = 1;

    /// Counter-Terrorist team index.
    pub const TEAM_CT: i32 = 3;

    /// Terrorist team index.
    pub const TEAM_T: i32 = 2;

    // ── Weapon / entity class name strings ───────────────────────────────────

    /// Prefix shared by all weapon entity class names.
    pub const WEAPON_CLASS_PREFIX: &str = "weapon_";

    /// Class name for the planted bomb entity.
    pub const PLANTED_C4_CLASS: &str = "C_PlantedC4";

    /// Class name for the hostage entity.
    pub const HOSTAGE_CLASS: &str = "C_Hostage";

    /// Class name for a generic grenade projectile.
    pub const GRENADE_PROJECTILE_CLASS: &str = "C_BaseCSGrenadeProjectile";

    /// Class name for an inferno (molotov / incendiary) entity.
    pub const INFERNO_CLASS: &str = "C_Inferno";

    /// Class name for a smoke grenade projectile.
    pub const SMOKE_GRENADE_CLASS: &str = "C_SmokeGrenadeProjectile";

    /// Class name for a flash-bang grenade projectile.
    pub const FLASH_GRENADE_CLASS: &str = "C_FlashbangProjectile";

    /// CS2 entity class identifiers.
    pub mod class {
        /// Base player pawn class name.
        pub const CS_PLAYER_PAWN: &str = "C_CSPlayerPawn";

        /// Controller (player state) class name.
        pub const CS_PLAYER_CONTROLLER: &str = "CCSPlayerController";

        /// Game rules singleton class name.
        pub const CS_GAME_RULES: &str = "C_CSGameRules";

        /// Weapon base class name.
        pub const BASE_WEAPON: &str = "C_CSWeaponBase";

        /// Bomb item (carried) class name.
        pub const BOMB_ITEM: &str = "C_C4";
    }
}

/// ELF binary format offsets used when parsing shared libraries from memory.
pub mod elf {
    /// Offset of the `e_ident` magic bytes field (0x7fELF).
    pub const E_IDENT_OFFSET: usize = 0x00;

    /// Length of the ELF identification array.
    pub const EI_NIDENT: usize = 16;

    /// Offset of the `e_type` field in a 64-bit ELF header.
    pub const E_TYPE_OFFSET: usize = 0x10;

    /// Offset of the `e_phoff` field (program header table file offset).
    pub const E_PHOFF_OFFSET: usize = 0x20;

    /// Offset of the `e_shoff` field (section header table file offset).
    pub const E_SHOFF_OFFSET: usize = 0x28;

    /// Offset of the `e_phentsize` field.
    pub const E_PHENTSIZE_OFFSET: usize = 0x36;

    /// Offset of the `e_phnum` field.
    pub const E_PHNUM_OFFSET: usize = 0x38;

    /// Size of a 64-bit ELF program header entry.
    pub const PHDR_SIZE: usize = 0x38;

    /// Offset of `p_type` in a program header entry.
    pub const PT_TYPE_OFFSET: usize = 0x00;

    /// Offset of `p_flags` in a 64-bit program header entry.
    pub const PT_FLAGS_OFFSET: usize = 0x04;

    /// Offset of `p_offset` (file offset) in a 64-bit program header entry.
    pub const PT_OFFSET_OFFSET: usize = 0x08;

    /// Offset of `p_vaddr` in a 64-bit program header entry.
    pub const PT_VADDR_OFFSET: usize = 0x10;

    /// Offset of `p_filesz` in a 64-bit program header entry.
    pub const PT_FILESZ_OFFSET: usize = 0x20;

    /// Offset of `p_memsz` in a 64-bit program header entry.
    pub const PT_MEMSZ_OFFSET: usize = 0x28;

    /// `PT_LOAD` program header type value.
    pub const PT_LOAD: u32 = 0x01;

    /// `PT_DYNAMIC` program header type value.
    pub const PT_DYNAMIC: u32 = 0x02;
}
