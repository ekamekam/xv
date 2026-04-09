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

// ── Pattern signatures ───────────────────────────────────────────────────────

/// `dwLocalPlayerController` — pointer to the local player controller object.
const SIG_LOCAL_PLAYER: &str = "48 83 3D ? ? ? ? 00 0F 95 C0 C3";

/// `dwViewMatrix` — 4×4 view/projection matrix used for world-to-screen.
const SIG_VIEW_MATRIX: &str = "C6 83 ? ? 00 00 01 4C 8D 05";

/// `dwPlantedC4` — pointer to the planted bomb entity.
const SIG_PLANTED_C4: &str =
    "48 8D 35 ? ? ? ? 66 0F EF C0 C6 05 ? ? ? ? 01 48 8D 3D";

/// `dwGlobalVars` — pointer to the server-side `CGlobalVarsBase` struct.
const SIG_GLOBAL_VARS: &str = "48 8D 05 ? ? ? ? 48 8B 00 8B 48 ? E9";

/// `EntityListOffset` — Entity system list offset (Osiris pattern)
const SIG_ENTITY_LIST_LINUX: &str = "4C 8D 6F ? 41 54 53 48 89 FB 48 83 EC ? 48 89 07 48";

/// `OffsetToBasePawnHandle` — Pawn handle offset within controller (Osiris pattern)
const SIG_PAWN_HANDLE: &str = "84 C0 75 ? 8B 8F ? ? ? ?";

// ── Scan window ──────────────────────────────────────────────────────────

/// Maximum bytes to scan per module when searching for signatures.
/// 32 MiB is enough to cover the entire `.text` of `libclient.so`.
const MAX_SCAN_BYTES: usize = 32 * 1024 * 1024;

// ── Public entry point ───────────────────────────────────────────────────────

/// Discovers all offsets dynamically from the running CS2 process.
///
/// On success returns a fully populated [`Offsets`] struct.
/// On partial failure individual fields fall back to the values from
/// [`Offsets::load`].
pub fn discover_offsets(process: &mut Process) -> Result<Offsets, ProcessError> {
    let fallback = Offsets::load();

    // ── Library bases ────────────────────────────────────────────────────────
    let libs = discover_libraries(process);

    let client = if libs.client != 0 { libs.client } else {
        return Err(ProcessError::ModuleNotFound("libclient.so".into()));
    };

    // Get the actual module base for offset calculations
    let module_base = libs.client;
    
    // ── Direct offsets via pattern scanning ───────────────────────────────────
    let local_player_controller = scan_local_player(process, client, module_base)
        .unwrap_or(fallback.direct.local_player_controller);

    let view_matrix = scan_view_matrix(process, client, module_base)
        .unwrap_or(fallback.direct.view_matrix);

    let planted_c4 = scan_planted_c4(process, client, module_base)
        .unwrap_or(fallback.direct.planted_c4);

    let global_vars = scan_global_vars(process, client, module_base)
        .unwrap_or(0);

    let local_player_pawn = scan_local_player_pawn(process, client, module_base)
        .unwrap_or(fallback.direct.local_player_pawn);

    let entity_list = scan_entity_list(process, client, module_base)
        .unwrap_or(fallback.direct.entity_list);

    // Use fallback for controller_pawn_handle - scanning is unreliable
    let controller_pawn_handle = fallback.direct.controller_pawn_handle;
    eprintln!("[OFFSET] Using controller_pawn_handle: {:#x}", controller_pawn_handle);

    let direct = Direct {
        entity_list,
        local_player_controller,
        local_player_pawn,
        view_matrix,
        planted_c4,
        game_rules: fallback.direct.game_rules,
        global_vars,
        controller_pawn_handle,
    };
    eprintln!("[OFFSET] Direct struct controller_pawn_handle: {:#x}", direct.controller_pawn_handle);

    // ── Interface resolution ──────────────────────────────────────────────────
    let interfaces = discover_interfaces(process, &libs);

    // ── ConVar resolution ─────────���───────────────────────────────────────────
    let convars = discover_convars(process, interfaces.cvar);

    Ok(Offsets {
        direct,
        iface: fallback.iface,
        libs,
        interfaces,
        convars,
    })
}

// ── Library discovery ────────────────────────────────────────────────────────

fn discover_libraries(process: &mut Process) -> LibraryOffsets {
    let client_base = process.get_module("libclient.so").unwrap_or(0);
    eprintln!("[LIBS] libclient.so base: {:#x}", client_base);
    LibraryOffsets {
        client: client_base,
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
fn scan_local_player(process: &mut Process, client: u64, module_base: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for local_player");
    match process.scan(SIG_LOCAL_PLAYER, client, MAX_SCAN_BYTES) {
        Some(hit) => {
            match process.get_relative_address(hit, 3, 8) {
                Ok(abs) => {
                    let offset = abs - module_base;
                    eprintln!("[SCAN] Found local_player at offset: {:#x}", offset);
                    
                    // Sanity check: offset should be < 10MB
                    if offset > 0xA00000 {
                        eprintln!("[SCAN] WARNING: offset {:#x} seems too large", offset);
                        return None;
                    }
                    Some(offset)
                }
                Err(e) => {
                    eprintln!("[SCAN] get_relative_address failed: {}", e);
                    None
                }
            }
        }
        None => {
            eprintln!("[SCAN] local_player pattern not found");
            None
        }
    }
}

/// Scans for `dwViewMatrix` using [`SIG_VIEW_MATRIX`].
fn scan_view_matrix(process: &mut Process, client: u64, module_base: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for view_matrix");
    let hit = process.scan(SIG_VIEW_MATRIX, client, MAX_SCAN_BYTES)?;
    // Pattern: C6 83 ? ? 00 00 01 4C 8D 05 [rel32]
    // The LEA r8, [rip+rel] is at offset 7
    let lea_addr = hit + 7;
    let abs = process.get_relative_address(lea_addr, 3, 7).ok()?;
    eprintln!("[SCAN] Found view_matrix at offset: {:#x}", abs - client);
    Some(abs - client)
}

/// Scans for `dwPlantedC4` using [`SIG_PLANTED_C4`].
fn scan_planted_c4(process: &mut Process, client: u64, module_base: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for planted_c4");
    let hit = process.scan(SIG_PLANTED_C4, client, MAX_SCAN_BYTES)?;
    let abs = process.get_relative_address(hit, 3, 7).ok()?;
    eprintln!("[SCAN] Found planted_c4 at offset: {:#x}", abs - client);
    Some(abs - client)
}

/// Scans for `dwGlobalVars` using [`SIG_GLOBAL_VARS`].
fn scan_global_vars(process: &mut Process, client: u64, module_base: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for global_vars");
    let hit = process.scan(SIG_GLOBAL_VARS, client, MAX_SCAN_BYTES)?;
    let abs = process.get_relative_address(hit, 3, 7).ok()?;
    eprintln!("[SCAN] Found global_vars at offset: {:#x}", abs - client);
    Some(abs - client)
}

/// Scans for `dwLocalPlayerPawn` pointer
/// Pattern: `48 8B 05 ? ? ? ? 48 8B 80 ? ? 00 00` (MOV rax, [rip+rel]; MOV rax, [rax+offset])
const SIG_LOCAL_PLAYER_PAWN: &str = "48 8B 05 ? ? ? ? 48 8B 80 ? ? 00 00";

fn scan_local_player_pawn(process: &mut Process, client: u64, module_base: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for local_player_pawn");
    let hit = process.scan(SIG_LOCAL_PLAYER_PAWN, client, MAX_SCAN_BYTES)?;
    let abs = process.get_relative_address(hit, 3, 7).ok()?;
    eprintln!("[SCAN] Found local_player_pawn at offset: {:#x}", abs - client);
    Some(abs - client)
}

/// Pattern for entity list pointer reference - just the LEA instruction
const SIG_ENTITY_LIST: &str = "48 8D 3D ? ? ? ?";

fn scan_entity_list(process: &mut Process, client: u64, module_base: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for entity_list");
    let hit = process.scan(SIG_ENTITY_LIST, client, MAX_SCAN_BYTES)?;
    // LEA rdi, [rip+rel32] - the relative address is at offset 3, size 4
    let abs = process.get_relative_address(hit, 3, 7).ok()?;
    let offset = abs - client;
    eprintln!("[SCAN] Found entity_list pattern at: {:#x}, calculated offset: {:#x}", abs, offset);
    Some(offset)
}

/// Scans for pawn handle offset using Osiris pattern
/// Pattern: `84 C0 75 ? 8B 8F ? ? ? ?` (TEST + JNZ + MOV r9, [rdi+offset])
/// Extracts the u32 offset at bytes 6-9
fn scan_pawn_handle_offset(process: &mut Process, client: u64) -> Option<u64> {
    eprintln!("[SCAN] Scanning for pawn_handle offset");
    let hit = process.scan(SIG_PAWN_HANDLE, client, MAX_SCAN_BYTES)?;
    eprintln!("[SCAN] Found pawn_handle pattern at: {:#x}", hit);
    // Pattern: 84 C0 75 ? 8B 8F [offset32]
    // Bytes:    0  1  2  3  4  5  6  7  8  9
    // MOV r9, [rdi + offset] is: 8B 8F offset32
    // So offset is at bytes 7-10 (skip 0x8B 0x8F)
    let offset_bytes = process.read_bytes(hit + 7, 4).ok()?;
    let offset = u32::from_le_bytes(offset_bytes.try_into().ok()?) as u64;
    eprintln!("[SCAN] Extracted pawn_handle offset: {:#x}", offset);
    
    // Sanity check: should be around 0x7E4
    if offset < 0x100 || offset > 0x2000 {
        eprintln!("[SCAN] WARNING: offset {:#x} seems wrong, expected ~0x7E4", offset);
    }
    Some(offset)
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

    InterfaceOffsets { resource, entity: 0, cvar, input }
}

// ── ConVar discovery ────────────────────────────────────────────────────────

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
        let mut proc = Process::new(0);
        let libs = discover_libraries(&mut proc);
        assert_eq!(libs.client, 0);
        assert_eq!(libs.engine, 0);
        assert_eq!(libs.tier0, 0);
    }
}
