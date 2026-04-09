/// Memory offsets for CS2 structures.
///
/// These offsets change with every CS2 patch and should be updated accordingly.
/// Offsets are relative to the base address of the containing library.

/// Offsets inside `libclient.so` / direct memory addresses.
#[derive(Debug, Clone)]
pub struct Direct {
    /// `dwEntityList` — pointer to the client entity list.
    pub entity_list: u64,
    /// `dwLocalPlayerController` — pointer to local player controller.
    pub local_player_controller: u64,
    /// `dwLocalPlayerPawn` — pointer to local player pawn.
    pub local_player_pawn: u64,
    /// `dwViewMatrix` — 4×4 view/projection matrix.
    pub view_matrix: u64,
    /// `dwPlantedC4` — pointer to the planted bomb entity.
    pub planted_c4: u64,
    /// `dwGameRules` — pointer to game rules.
    pub game_rules: u64,
}

/// Offsets of fields within CS2 classes.
///
/// All values are byte offsets from the start of the containing object.
#[derive(Debug, Clone)]
pub struct Interface {
    // ── CCSPlayerController ─────────────────────────────────────────────────
    /// `m_steamID` — SteamID64.
    pub controller_steam_id: u64,
    /// `m_iszPlayerName` — player name string (up to 128 bytes).
    pub controller_player_name: u64,
    /// `m_hPlayerPawn` — handle to the player pawn.
    pub controller_pawn_handle: u64,
    /// `m_iTeamNum` — team number (2 = T, 3 = CT).
    pub controller_team_num: u64,

    // ── C_CSPlayerPawn ───────────────────────────────────────────────────────
    /// `m_iHealth` — player health.
    pub pawn_health: u64,
    /// `m_ArmorValue` — armour value.
    pub pawn_armor: u64,
    /// `m_vecAbsOrigin` — world-space position.
    pub pawn_origin: u64,
    /// `m_vecViewOffset` — eye offset relative to origin.
    pub pawn_view_offset: u64,
    /// `m_angEyeAngles` — eye angles (pitch, yaw).
    pub pawn_eye_angles: u64,
    /// `m_pGameSceneNode` — pointer to game scene node (for bones).
    pub pawn_game_scene_node: u64,
    /// `m_vecVelocity` — movement velocity.
    pub pawn_velocity: u64,
    /// `m_bHasDefuser` — defuser kit flag.
    pub pawn_has_defuser: u64,
    /// `m_bHasHelmet` — helmet flag.
    pub pawn_has_helmet: u64,

    // ── CGameSceneNode ───────────────────────────────────────────────────────
    /// `m_modelState` — model state (contains bone data pointer).
    pub scene_node_model_state: u64,

    // ── C_PlantedC4 ──────────────────────────────────────────────────────────
    /// `m_flC4Blow` — time when bomb explodes.
    pub c4_blow_time: u64,
    /// `m_bBombDefused` — defused flag.
    pub c4_defused: u64,
    /// `m_flDefuseCountDown` — defuse countdown timer.
    pub c4_defuse_countdown: u64,
    /// `m_vecAbsOrigin` (on planted C4).
    pub c4_origin: u64,
    /// `m_hBombDefuser` — entity handle of the player defusing.
    pub c4_defuser: u64,

    // ── CBaseCSGrenadeProjectile ─────────────────────────────────────────────
    /// `m_vecAbsOrigin` on a grenade projectile.
    pub grenade_origin: u64,

    // ── CGameRules ───────────────────────────────────────────────────────────
    /// `m_bFreezePeriod` — true during freeze time.
    pub game_rules_freeze_period: u64,
}

/// Top-level container for all game offsets.
#[derive(Debug, Clone)]
pub struct Offsets {
    pub direct: Direct,
    pub iface: Interface,
}

impl Offsets {
    /// Returns default offsets that match a recent CS2 build.
    ///
    /// These values are based on publicly documented offset dumps and will need
    /// updating when Valve patches the game.
    pub fn load() -> Self {
        Self {
            direct: Direct {
                entity_list: 0x18E1_A48,
                local_player_controller: 0x1856_8B8,
                local_player_pawn: 0x173F_D20,
                view_matrix: 0x18D_D5E0,
                planted_c4: 0x18E_0F28,
                game_rules: 0x18D_7A40,
            },
            iface: Interface {
                // CCSPlayerController
                controller_steam_id: 0x7E0,
                controller_player_name: 0x640,
                controller_pawn_handle: 0x7E4,
                controller_team_num: 0x3BF,

                // C_CSPlayerPawn
                pawn_health: 0x344,
                pawn_armor: 0xDE4,
                pawn_origin: 0xC8,
                pawn_view_offset: 0xC84,
                pawn_eye_angles: 0x1510,
                pawn_game_scene_node: 0x328,
                pawn_velocity: 0x3F0,
                pawn_has_defuser: 0xDF0,
                pawn_has_helmet: 0xDF1,

                // CGameSceneNode
                scene_node_model_state: 0x170,

                // C_PlantedC4
                c4_blow_time: 0xB10,
                c4_defused: 0xB6C,
                c4_defuse_countdown: 0xB74,
                c4_origin: 0xC8,
                c4_defuser: 0xB64,

                // CBaseCSGrenadeProjectile
                grenade_origin: 0xC8,

                // CGameRules
                game_rules_freeze_period: 0xA0,
            },
        }
    }

    /// Attempt to resolve offsets by scanning ELF sections of the loaded
    /// library at `base_address`.
    ///
    /// This is a stub that returns a clone of `self` — real signature scanning
    /// would parse the ELF header and walk `.text` for known byte patterns.
    /// Implementing full pattern scanning is beyond the scope of this layer,
    /// but the interface is kept here so callers can swap in a real
    /// implementation without changing downstream code.
    pub fn resolve_from_binary(&self, _base_address: u64, _data: &[u8]) -> Self {
        // In a full implementation, this would:
        //  1. Parse the ELF header from `data`
        //  2. Walk `.text` / `.rodata` for known byte signatures
        //  3. Return updated offsets
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_returns_non_zero_offsets() {
        let offsets = Offsets::load();
        assert_ne!(offsets.direct.entity_list, 0);
        assert_ne!(offsets.iface.pawn_health, 0);
    }

    #[test]
    fn test_resolve_from_binary_is_stable() {
        let offsets = Offsets::load();
        let resolved = offsets.resolve_from_binary(0, &[]);
        assert_eq!(resolved.direct.entity_list, offsets.direct.entity_list);
    }
}
