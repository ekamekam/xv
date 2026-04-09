//! Dynamic offset discovery for CS2.
//!
//! This module provides [`discover_offsets`], which automatically locates the
//! memory addresses that change every CS2 patch by:
//!
//! 1. Resolving library base addresses from `/proc/<pid>/maps`.
//! 2. Scanning `.text` segments for known byte-pattern signatures.
//! 3. Computing RIP-relative addresses from the matched instructions.
//! 4. Walking the CS2 interface registry to find `ICvar` and entity interfaces.
//!
//! If discovery fails for any field the hardcoded fallback from
//! [`super::offsets::Offsets::load`] is used instead, so the system degrades
//! gracefully across minor game updates.

use super::offsets::{
    ConvarOffsets, Direct, InterfaceOffsets, LibraryOffsets, Offsets,
};
use super::{Process, ProcessError};

// ── Pattern signatures ────────────────────────────────────────────────────────
//
// Each signature is a space-separated hex string; `?` matches any byte.
// These are the canonical patterns from the deadlocked project.

/// `dwLocalPlayerController` — pointer to the local player controller object.
const SIG_LOCAL_PLAYER: &str = "48 83 3D ? ? ? ? 00 0F 95 C0 C3";

/// `dwViewMatrix` — 4×4 view/projection matrix used for world-to-screen.
const SIG_VIEW_MATRIX: &str = "C6 83 ? ? 00 00 01 4C 8D 05";

/// `dwPlantedC4` — pointer to the planted bomb entity.
const SIG_PLANTED_C4: &str =
    "48 8D 35 ? ? ? ? 66 0F EF C0 C6 05 ? ? ? ? 01 48 8D 3D";

/// `dwGlobalVars` — pointer to the server-side `CGlobalVarsBase` struct.
const SIG_GLOBAL_VARS: &str = "48 8D 05 ? ? ? ? 48 8B 00 8B 48 ? E9";

// ── Scan window ───────────────────────────────────────────────────────────────

/// Maximum bytes to scan per module when searching for signatures.
/// 32 MiB is enough to cover the entire `.text` of `libclient.so`.
const MAX_SCAN_BYTES: usize = 32 * 1024 * 1024;

// ── Public entry point ────────────────────────────────────────────────────────

/// Discovers all offsets dynamically from the running CS2 process.
///
/// On success returns a fully populated [`Offsets`] struct.
/// On partial failure individual fields fall back to the values from
/// [`Offsets::load`].
pub fn discover_offsets(process: &mut Process) -> Result<Offsets, ProcessError> {
    let fallback = Offsets::load();

    // ── Library bases ─────────────────────────────────────────────────────────
    let libs = discover_libraries(process);

    let client = if libs.client != 0 { libs.client } else {
        return Err(ProcessError::ModuleNotFound("libclient.so".into()));
    };

    // ── Direct offsets via pattern scanning ───────────────────────────────────
    let local_player_controller = scan_local_player(process, client)
        .unwrap_or(fallback.direct.local_player_controller);

    let view_matrix = scan_view_matrix(process, client)
        .unwrap_or(fallback.direct.view_matrix);

    let planted_c4 = scan_planted_c4(process, client)
        .unwrap_or(fallback.direct.planted_c4);

    let global_vars = scan_global_vars(process, client)
        .unwrap_or(0);

    let direct = Direct {
        entity_list: fallback.direct.entity_list,
        local_player_controller,
        local_player_pawn: fallback.direct.local_player_pawn,
        view_matrix,
        planted_c4,
        game_rules: fallback.direct.game_rules,
        global_vars,
    };

    // ── Interface resolution ──────────────────────────────────────────────────
    let interfaces = discover_interfaces(process, &libs);

    // ── ConVar resolution ─────────────────────────────────────────────────────
    let convars = discover_convars(process, interfaces.cvar);

    Ok(Offsets {
        direct,
        iface: fallback.iface,
        libs,
        interfaces,
        convars,
    })
}

// ── Library discovery ─────────────────────────────────────────────────────────

fn discover_libraries(process: &mut Process) -> LibraryOffsets {
    LibraryOffsets {
        client: process.get_module("libclient.so").unwrap_or(0),
        engine: process.get_module("libengine2.so").unwrap_or(0),
        tier0: process.get_module("libtier0.so").unwrap_or(0),
        input: process.get_module("libinputsystem.so").unwrap_or(0),
        sdl: process.get_module("libSDL3.so.0").unwrap_or(
            process.get_module("libSDL3.so").unwrap_or(0)
        ),
        schema: process.get_module("libschemasystem.so").unwrap_or(0),
    }
}

// ── Pattern-based offset scanners ─────────────────────────────────────────────

/// Scans for `dwLocalPlayerController` using [`SIG_LOCAL_PLAYER`].
///
/// The signature `48 83 3D [rip+rel: 4 bytes] 00 ...` encodes a CMP against a
/// global pointer; the RIP-relative operand (offset=3, insn_size=8) points to
/// the `dwLocalPlayerController` global.
fn scan_local_player(process: &mut Process, client: u64) -> Option<u64> {
    let hit = process.scan(SIG_LOCAL_PLAYER, client, MAX_SCAN_BYTES)?;
    // CMP qword ptr [rip+rel], 0  =>  48 83 3D [rel32] 00
    // offset=3, total instruction size=8
    let abs = process.get_relative_address(hit, 3, 8).ok()?;
    Some(abs - client)
}

/// Scans for `dwViewMatrix` using [`SIG_VIEW_MATRIX`].
///
/// The signature `C6 83 ?? ?? 00 00 01 4C 8D 05 [rip+rel]` — the MOV at the
/// end contains a RIP-relative reference to the view-matrix global.
/// offset=10, instruction size=7 (LEA r8, [rip+rel]).
fn scan_view_matrix(process: &mut Process, client: u64) -> Option<u64> {
    let hit = process.scan(SIG_VIEW_MATRIX, client, MAX_SCAN_BYTES)?;
    // LEA r8, [rip + rel32] at hit+7: 4C 8D 05 [rel32]
    // offset = 3 within the LEA (starts at hit+7), insn_size = 7
    let lea_addr = hit + 7;
    let abs = process.get_relative_address(lea_addr, 3, 7).ok()?;
    Some(abs - client)
}

/// Scans for `dwPlantedC4` using [`SIG_PLANTED_C4`].
///
/// The signature begins with `48 8D 35 [rip+rel]` — a LEA rsi,[rip+rel]
/// that points at the planted-C4 list pointer.
/// offset=3, instruction size=7.
fn scan_planted_c4(process: &mut Process, client: u64) -> Option<u64> {
    let hit = process.scan(SIG_PLANTED_C4, client, MAX_SCAN_BYTES)?;
    let abs = process.get_relative_address(hit, 3, 7).ok()?;
    Some(abs - client)
}

/// Scans for `dwGlobalVars` using [`SIG_GLOBAL_VARS`].
///
/// `48 8D 05 [rip+rel]` — LEA rax,[rip+rel], offset=3, size=7.
fn scan_global_vars(process: &mut Process, client: u64) -> Option<u64> {
    let hit = process.scan(SIG_GLOBAL_VARS, client, MAX_SCAN_BYTES)?;
    let abs = process.get_relative_address(hit, 3, 7).ok()?;
    Some(abs - client)
}

// ── Interface discovery ───────────────────────────────────────────────────────

fn discover_interfaces(process: &mut Process, libs: &LibraryOffsets) -> InterfaceOffsets {
    let resource = if libs.engine != 0 {
        process
            .get_interface_offset(libs.engine, "GameResourceServiceClientV")
            .unwrap_or(0)
    } else {
        0
    };

    let cvar = if libs.tier0 != 0 {
        process
            .get_interface_offset(libs.tier0, "VEngineCvar")
            .unwrap_or(0)
    } else {
        0
    };

    let input = if libs.input != 0 {
        process
            .get_interface_offset(libs.input, "InputSystemVersion")
            .unwrap_or(0)
    } else {
        0
    };

    // IEntitySystem is accessed through GameResourceService; skip for now.
    InterfaceOffsets { resource, entity: 0, cvar, input }
}

// ── ConVar discovery ──────────────────────────────────────────────────────────

fn discover_convars(process: &mut Process, cvar_interface: u64) -> ConvarOffsets {
    if cvar_interface == 0 {
        return ConvarOffsets::default();
    }
    let ffa = process
        .get_convar(cvar_interface, "mp_teammates_are_enemies")
        .unwrap_or(0);
    let sensitivity = process
        .get_convar(cvar_interface, "sensitivity")
        .unwrap_or(0);
    ConvarOffsets { ffa, sensitivity }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::Process;

    #[test]
    fn parse_pattern_basic() {
        let (bytes, mask) = Process::parse_pattern("48 83 3D ? ? 00");
        assert_eq!(bytes, vec![0x48, 0x83, 0x3D, 0x00, 0x00, 0x00]);
        assert_eq!(mask, vec![true, true, true, false, false, true]);
    }

    #[test]
    fn parse_pattern_double_wildcard() {
        let (bytes, mask) = Process::parse_pattern("48 ?? 3D");
        assert_eq!(bytes, vec![0x48, 0x00, 0x3D]);
        assert_eq!(mask, vec![true, false, true]);
    }

    #[test]
    fn parse_pattern_empty() {
        let (bytes, mask) = Process::parse_pattern("");
        assert!(bytes.is_empty());
        assert!(mask.is_empty());
    }

    #[test]
    fn discover_libraries_no_crash_on_missing_modules() {
        // A fresh Process with no open handle still has modules = [].
        let mut proc = Process::new(0);
        let libs = discover_libraries(&mut proc);
        assert_eq!(libs.client, 0);
        assert_eq!(libs.engine, 0);
        assert_eq!(libs.tier0, 0);
    }
}
