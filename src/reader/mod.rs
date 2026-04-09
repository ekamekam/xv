//! Game state reader — translates raw memory reads into typed [`Data`].
//!
//! The main entry point is [`GameReader`], which wraps a [`Process`] handle
//! and decodes the CS2 memory layout defined in [`crate::process::offsets`].

use glam::{Mat4, Vec3};

use crate::data::{Data, PlayerData};
use crate::process::offsets::Offsets;
use crate::process::{Process, ProcessError};

/// Error type returned by [`GameReader`] operations.
#[derive(Debug)]
pub enum ReaderError {
    /// Underlying memory read error.
    Memory(ProcessError),
    /// A required module was not found in the process map.
    ModuleNotFound(String),
    /// Attempted to read from a null/invalid pointer.
    NullPointer(u64),
}

impl std::fmt::Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReaderError::Memory(e) => write!(f, "memory error: {e}"),
            ReaderError::ModuleNotFound(name) => write!(f, "module not found: {name}"),
            ReaderError::NullPointer(addr) => write!(f, "null pointer at {addr:#x}"),
        }
    }
}

impl std::error::Error for ReaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReaderError::Memory(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ProcessError> for ReaderError {
    fn from(e: ProcessError) -> Self {
        ReaderError::Memory(e)
    }
}

/// Reads typed game data from a live CS2 process.
pub struct GameReader {
    process: Process,
    offsets: Offsets,
    /// Base address of `libclient.so`.
    client_base: u64,
}

impl GameReader {
    /// Creates a new `GameReader` attached to the given process.
    ///
    /// Resolves the `libclient.so` base address at construction time.
    /// Offsets can be auto-discovered or supplied directly.
    pub fn new(process: Process, offsets: Offsets) -> Result<Self, ReaderError> {
        let client_base = process
            .get_module("libclient.so")
            .map_err(|_| ReaderError::ModuleNotFound("libclient.so".into()))?;
        Ok(Self { process, offsets, client_base })
    }

    /// Creates a `GameReader` with dynamically discovered offsets.
    ///
    /// Calls [`crate::process::offsets_discovery::discover_offsets`] to scan
    /// the game binary for up-to-date offsets before constructing the reader.
    pub fn new_with_discovery(mut process: Process) -> Result<Self, ReaderError> {
        let offsets = crate::process::offsets_discovery::discover_offsets(&mut process)
            .unwrap_or_else(|_| Offsets::load());
        let client_base = process
            .get_module("libclient.so")
            .map_err(|_| ReaderError::ModuleNotFound("libclient.so".into()))?;
        Ok(Self { process, offsets, client_base })
    }

    fn read_u32(&mut self, addr: u64) -> Result<u32, ReaderError> {
        Ok(self.process.read_u32(addr)?)
    }

    fn read_u64(&mut self, addr: u64) -> Result<u64, ReaderError> {
        Ok(self.process.read_u64(addr)?)
    }

    fn read_f32(&mut self, addr: u64) -> Result<f32, ReaderError> {
        Ok(self.process.read_f32(addr)?)
    }

    fn read_vec3(&mut self, addr: u64) -> Result<Vec3, ReaderError> {
        Ok(self.process.read_vec3(addr)?)
    }

    /// Reads a null-terminated UTF-8 string (max `max_len` bytes) from `addr`.
    fn read_string(&mut self, addr: u64, max_len: usize) -> Result<String, ReaderError> {
        let bytes = self.process.read_bytes(addr, max_len)?;
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
    }

    // ── Game-state readers ────────────────────────────────────────────────────

    /// Reads fields from a player pawn into `out`.
    fn read_pawn(&mut self, pawn: u64, out: &mut PlayerData) -> Result<(), ReaderError> {
        // Clone scalar offsets up-front so we don't hold a borrow on `self`
        // while calling `self.read_*` methods.
        let off_health = self.offsets.iface.pawn_health;
        let off_armor = self.offsets.iface.pawn_armor;
        let off_origin = self.offsets.iface.pawn_origin;
        let off_view_offset = self.offsets.iface.pawn_view_offset;
        let off_velocity = self.offsets.iface.pawn_velocity;
        let off_has_defuser = self.offsets.iface.pawn_has_defuser;
        let off_has_helmet = self.offsets.iface.pawn_has_helmet;

        out.pawn = pawn;
        out.health = self.read_u32(pawn + off_health)? as i32;
        out.armor = self.read_u32(pawn + off_armor)? as i32;

        let origin = self.read_vec3(pawn + off_origin)?;
        out.position = origin;

        let view_offset = self.read_vec3(pawn + off_view_offset)?;
        out.eye_pos = origin + view_offset;

        out.velocity = self.read_vec3(pawn + off_velocity)?;

        let has_defuser = self.process.read_bytes(pawn + off_has_defuser, 1)?;
        out.has_defuser = has_defuser.first().copied().unwrap_or(0) != 0;

        let has_helmet = self.process.read_bytes(pawn + off_has_helmet, 1)?;
        out.has_helmet = has_helmet.first().copied().unwrap_or(0) != 0;

        Ok(())
    }

    /// Populates `data` with the current game state.
    pub fn update_game_data(&mut self, data: &mut Data) -> Result<(), ReaderError> {
        let client = self.client_base;
        let off = self.offsets.clone();

        // ── View matrix ──────────────────────────────────────────────────────
        let vm_addr = client + off.direct.view_matrix;
        let mut vm_floats = [0f32; 16];
        for (i, val) in vm_floats.iter_mut().enumerate() {
            *val = self.read_f32(vm_addr + i as u64 * 4)?;
        }
        data.view_matrix = Mat4::from_cols_array(&vm_floats);

        // ── Local player controller ───────────────────────────────────────────
        let lpc_ptr = client + off.direct.local_player_controller;
        let local_controller = self.read_u64(lpc_ptr)?;
        if local_controller == 0 {
            data.in_game = false;
            return Ok(());
        }
        data.in_game = true;

        // ── Local player name ─────────────────────────────────────────────────
        let name_addr = local_controller + off.iface.controller_player_name;
        data.local_player.name = self.read_string(name_addr, 128).unwrap_or_default();

        // ── Local player pawn ─────────────────────────────────────────────────
        let lpp_ptr = client + off.direct.local_player_pawn;
        let local_pawn = self.read_u64(lpp_ptr)?;
        if local_pawn != 0 {
            // line 173: origin is used within read_pawn via pawn_origin offset
            let _origin = self.read_vec3(local_pawn + off.iface.pawn_origin)?;
            self.read_pawn(local_pawn, &mut data.local_player)?;
        }

        // ── Entity list — collect other players ───────────────────────────────
        let entity_list_ptr = client + off.direct.entity_list;
        let entity_list = self.read_u64(entity_list_ptr).unwrap_or(0);
        if entity_list != 0 {
            data.players.clear();
            // Walk up to 64 controller slots (indices 1–64; 0 is local).
            for i in 1u64..=64 {
                let controller_addr = entity_list + i * 0x78;
                let controller = match self.read_u64(controller_addr) {
                    Ok(v) if v != 0 => v,
                    _ => continue,
                };

                let pawn_handle_addr = controller + off.iface.controller_pawn_handle;
                let pawn_handle = self.read_u32(pawn_handle_addr).unwrap_or(0);
                if pawn_handle == 0xFFFF_FFFF {
                    continue;
                }

                // Resolve pawn via entity list (handle → pawn ptr).
                let list_entry = entity_list + (((pawn_handle & 0x7FFF) >> 9) as u64 + 1) * 8;
                let list_ptr = self.read_u64(list_entry).unwrap_or(0);
                if list_ptr == 0 {
                    continue;
                }
                let pawn = self.read_u64(list_ptr + (pawn_handle & 0x1FF) as u64 * 0x78)
                    .unwrap_or(0);
                if pawn == 0 {
                    continue;
                }

                let mut player = PlayerData::default();
                let name_addr = controller + off.iface.controller_player_name;
                player.name = self.read_string(name_addr, 128).unwrap_or_default();
                player.steam_id =
                    self.read_u64(controller + off.iface.controller_steam_id).unwrap_or(0);

                if self.read_pawn(pawn, &mut player).is_ok() && player.health > 0 {
                    data.players.push(player);
                }
            }
        }

        // ── Planted C4 ────────────────────────────────────────────────────────
        let c4_list_ptr = client + off.direct.planted_c4;
        let c4_entry = self.read_u64(c4_list_ptr).unwrap_or(0);
        if c4_entry != 0 {
            let c4_ptr = self.read_u64(c4_entry).unwrap_or(0);
            if c4_ptr != 0 {
                data.bomb.planted = true;
                data.bomb.timer =
                    self.read_f32(c4_ptr + off.iface.c4_blow_time).unwrap_or(0.0);
                data.bomb.position =
                    self.read_vec3(c4_ptr + off.iface.c4_origin).unwrap_or_default();
                let defuse_byte = self
                    .process
                    .read_bytes(c4_ptr + off.iface.c4_defused, 1)
                    .unwrap_or_default();
                data.bomb.being_defused =
                    defuse_byte.first().copied().unwrap_or(0) != 0;
                data.bomb.defuse_remain_time =
                    self.read_f32(c4_ptr + off.iface.c4_defuse_countdown).unwrap_or(0.0);
            }
        }

        // ── Game rules ────────────────────────────────────────────────────────
        let rules_addr_ptr = client + off.direct.game_rules;
        let rules_ptr = self.read_u64(rules_addr_ptr).unwrap_or(0);
        if rules_ptr != 0 {
            let freeze_byte = self
                .process
                .read_bytes(rules_ptr + off.iface.game_rules_freeze_period, 1)
                .unwrap_or_default();
            // freeze_byte[0] != 0 means freeze period is active (round start)
            let _ = freeze_byte;
        }

        Ok(())
    }
}

/// Mock memory backend for unit tests.
pub mod mock {
    use std::collections::HashMap;

    /// Simple in-process memory store used in tests.
    pub struct MockMemory {
        data: HashMap<u64, u8>,
    }

    impl MockMemory {
        /// Creates an empty mock memory store.
        pub fn new() -> Self {
            Self { data: HashMap::new() }
        }

        /// Writes a `u64` value at `address` (little-endian).
        pub fn write_u64(&mut self, address: u64, value: u64) {
            for (i, byte) in value.to_le_bytes().iter().enumerate() {
                self.data.insert(address + i as u64, *byte);
            }
        }

        /// Writes a `u32` value at `address` (little-endian).
        pub fn write_u32(&mut self, address: u64, value: u32) {
            for (i, byte) in value.to_le_bytes().iter().enumerate() {
                self.data.insert(address + i as u64, *byte);
            }
        }

        /// Writes an `f32` value at `address` (little-endian).
        pub fn write_f32(&mut self, address: u64, value: f32) {
            for (i, byte) in value.to_bits().to_le_bytes().iter().enumerate() {
                self.data.insert(address + i as u64, *byte);
            }
        }

        /// Reads `size` bytes starting at `address`. Returns zeros for unwritten bytes.
        pub fn read(&self, address: u64, size: usize) -> Vec<u8> {
            (0..size as u64)
                .map(|i| *self.data.get(&(address + i)).unwrap_or(&0))
                .collect()
        }
    }

    impl Default for MockMemory {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockMemory;

    #[test]
    fn test_mock_write_read_u64() {
        let mut mem = MockMemory::new();
        mem.write_u64(0x1000, 0xDEAD_BEEF_CAFE_BABE);
        let bytes = mem.read(0x1000, 8);
        let val = u64::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(val, 0xDEAD_BEEF_CAFE_BABE);
    }

    #[test]
    fn test_mock_write_read_f32() {
        let mut mem = MockMemory::new();
        mem.write_f32(0x2000, std::f32::consts::PI);
        let bytes = mem.read(0x2000, 4);
        let val = f32::from_le_bytes(bytes.try_into().unwrap());
        assert!((val - std::f32::consts::PI).abs() < 1e-7);
    }

    #[test]
    fn test_mock_unwritten_is_zero() {
        let mem = MockMemory::new();
        let bytes = mem.read(0x9999, 4);
        assert_eq!(bytes, vec![0u8; 4]);
    }
}
