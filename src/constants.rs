/// CS2 process/library names.
pub const CS2_PROCESS_NAME: &str = "cs2";
pub const CLIENT_LIB: &str = "libclient.so";
pub const ENGINE_LIB: &str = "libengine2.so";
pub const TIER0_LIB: &str = "libtier0.so";
pub const SCHEMA_SYSTEM_LIB: &str = "libschemasystem.so";
pub const MATCHMAKING_LIB: &str = "libmatchmaking.so";

/// CS2 team numbers.
pub const TEAM_UNASSIGNED: i32 = 0;
pub const TEAM_SPECTATOR: i32 = 1;
pub const TEAM_T: i32 = 2;
pub const TEAM_CT: i32 = 3;

/// CS2 game state flags.
pub const FL_ONGROUND: u32 = 1 << 0;
pub const FL_DUCKING: u32 = 1 << 1;

/// Maximum number of players in a CS2 match.
pub const MAX_PLAYERS: usize = 64;

/// CS2 entity class name strings used in schema lookups.
pub const CLASS_C_CS_PLAYER_PAWN: &str = "C_CSPlayerPawn";
pub const CLASS_C_CS_PLAYER_CONTROLLER: &str = "CCSPlayerController";
pub const CLASS_C_PLANTED_C4: &str = "C_PlantedC4";
pub const CLASS_C_SMOKE_GRENADE_PROJECTILE: &str = "C_SmokeGrenadeProjectile";
pub const CLASS_C_MOLOTOV_PROJECTILE: &str = "C_MolotovProjectile";
pub const CLASS_C_BASE_CS_GRENADE_PROJECTILE: &str = "C_BaseCSGrenadeProjectile";
pub const CLASS_C_INCENDIARY_GRENADE_PROJECTILE: &str = "C_IncendiaryGrenadeProjectile";
pub const CLASS_C_FLASHBANG_PROJECTILE: &str = "C_FlashbangProjectile";
pub const CLASS_C_DECOY_PROJECTILE: &str = "C_DecoyProjectile";
pub const CLASS_INFERNO: &str = "C_Inferno";

/// ELF section names used during offset resolution.
pub const ELF_SECTION_TEXT: &str = ".text";
pub const ELF_SECTION_DATA: &str = ".data";
pub const ELF_SECTION_RODATA: &str = ".rodata";
