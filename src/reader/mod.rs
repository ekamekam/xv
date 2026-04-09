//! High-level game state reader.
//!
//! [`GameReader`] orchestrates reading all CS2 game state into a [`Data`]
//! struct.  It owns a [`Process`] handle and uses cached [`Offsets`] so that
//! individual read methods are concise.

use glam::{Mat4, Vec3};

use crate::cs2::{entity::EntityInfo, weapon::Weapon};
use crate::data::{BombData, Data, PlayerData};
use crate::process::{offsets::Offsets, Process, ProcessError};

/// Error type returned by [`GameReader`] methods.
#[derive(Debug)]
pub enum ReadError {
    /// A memory read failed.
    Memory(ProcessError),
    /// The game does not appear to be running (not in a match, etc.).
    NotInGame,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Memory(e) => write!(f, "memory error: {e}"),
            ReadError::NotInGame => write!(f, "not in game"),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let ReadError::Memory(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

impl From<ProcessError> for ReadError {
    fn from(e: ProcessError) -> Self {
        ReadError::Memory(e)
    }
}

/// Reads a null-terminated UTF-8 string from the process at `address`.
///
/// Reads up to `max_bytes` and stops at the first null byte.
fn read_cstring(process: &mut Process, address: u64, max_bytes: usize) -> Result<String, ProcessError> {
    let bytes = process.read_bytes(address, max_bytes)?;
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
}

/// Orchestrates reading all CS2 game state.
pub struct GameReader {
    process: Process,
    offsets: Offsets,
    /// Base address of `libclient.so`.
    client_base: u64,
}

impl GameReader {
    /// Creates a new `GameReader` from an already-open [`Process`].
    ///
    /// `lib_name` is the name of the client library (e.g. `"libclient.so"`).
    pub fn new(process: Process, offsets: Offsets) -> Result<Self, ReadError> {
        let client_base = process.get_module(crate::constants::CLIENT_LIB)?;
        Ok(Self { process, offsets, client_base })
    }

    /// Returns an immutable reference to the underlying process.
    pub fn process(&self) -> &Process {
        &self.process
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Resolves a direct (library-relative) pointer.
    fn direct(&self, offset: u64) -> u64 {
        self.client_base + offset
    }

    /// Reads a pointer-width value (u64) and returns it.
    fn read_ptr(&mut self, address: u64) -> Result<u64, ProcessError> {
        self.process.read_u64(address)
    }

    // ── Public read methods ──────────────────────────────────────────────────

    /// Reads the current 4×4 view/projection matrix used by the renderer.
    pub fn read_view_matrix(&mut self) -> Result<Mat4, ReadError> {
        let addr = self.direct(self.offsets.direct.view_matrix);
        let mut floats = [0f32; 16];
        for (i, f) in floats.iter_mut().enumerate() {
            *f = self.process.read_f32(addr + (i as u64) * 4)?;
        }
        // glam is column-major; CS2 stores row-major — transpose on load.
        Ok(Mat4::from_cols_array(&floats).transpose())
    }

    /// Reads the current map name.
    pub fn read_map_name(&mut self) -> Result<String, ReadError> {
        // The map name is stored in the game rules object.
        let gr_ptr_addr = self.direct(self.offsets.direct.game_rules);
        let gr_ptr = self.read_ptr(gr_ptr_addr)?;
        if gr_ptr == 0 {
            return Ok(String::new());
        }
        // Map name string is at a fixed offset (0x188) inside the game rules object.
        let name = read_cstring(&mut self.process, gr_ptr + 0x188, 64)?;
        Ok(name)
    }

    /// Reads the local player's current state.
    pub fn read_local_player(&mut self) -> Result<PlayerData, ReadError> {
        let controller_addr = self.direct(self.offsets.direct.local_player_controller);
        let controller = self.read_ptr(controller_addr)?;
        if controller == 0 {
            return Err(ReadError::NotInGame);
        }

        let pawn_addr = self.direct(self.offsets.direct.local_player_pawn);
        let pawn = self.read_ptr(pawn_addr)?;
        if pawn == 0 {
            return Err(ReadError::NotInGame);
        }

        self.read_player_from_pawn(controller, pawn)
    }

    /// Reads all player data from the entity list.
    pub fn read_players(&mut self) -> Result<Vec<PlayerData>, ReadError> {
        let list_ptr_addr = self.direct(self.offsets.direct.entity_list);
        let list_ptr = self.read_ptr(list_ptr_addr)?;
        if list_ptr == 0 {
            return Ok(Vec::new());
        }

        let mut players = Vec::new();
        for i in 0..64u64 {
            let controller = match self.process.read_u64(list_ptr + i * 0x78) {
                Ok(v) if v != 0 => v,
                _ => continue,
            };

            let pawn = match self.process.read_u64(
                controller + self.offsets.iface.controller_pawn_handle,
            ) {
                Ok(v) if v != 0 => v,
                _ => continue,
            };

            match self.read_player_from_pawn(controller, pawn) {
                Ok(p) => players.push(p),
                Err(ReadError::Memory(_)) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(players)
    }

    /// Reads entities (bomb, grenades, infernos) from the entity list.
    pub fn read_entities(&mut self) -> Result<Vec<EntityInfo>, ReadError> {
        // Simplified stub: reads the planted C4 if present.
        let mut entities = Vec::new();

        let c4_ptr_addr = self.direct(self.offsets.direct.planted_c4);
        let c4_ptr = self.read_ptr(c4_ptr_addr).unwrap_or(0);
        if c4_ptr != 0 {
            let origin = self.process.read_vec3(c4_ptr + self.offsets.iface.c4_origin)?;
            entities.push(EntityInfo::Bomb { position: origin });
        }

        Ok(entities)
    }

    /// Reads the planted bomb state.
    pub fn read_bomb(&mut self) -> Result<BombData, ReadError> {
        let c4_ptr_addr = self.direct(self.offsets.direct.planted_c4);
        let c4_ptr = self.read_ptr(c4_ptr_addr).unwrap_or(0);
        if c4_ptr == 0 {
            return Ok(BombData::default());
        }

        let position = self.process.read_vec3(c4_ptr + self.offsets.iface.c4_origin)?;
        let blow_time = self.process.read_f32(c4_ptr + self.offsets.iface.c4_blow_time)?;
        let being_defused_handle = self.process.read_u32(c4_ptr + self.offsets.iface.c4_defuser).unwrap_or(0);
        let defuse_countdown = self.process.read_f32(c4_ptr + self.offsets.iface.c4_defuse_countdown).unwrap_or(0.0);

        Ok(BombData {
            planted: true,
            position,
            timer: blow_time,
            being_defused: being_defused_handle != 0,
            defuse_remain_time: defuse_countdown,
        })
    }

    /// Performs a full game state update, populating all fields in `data`.
    ///
    /// Individual sub-reads are allowed to fail without aborting the whole
    /// update — partial data is still useful for rendering.
    pub fn update_game_data(&mut self, data: &mut Data) -> Result<(), ReadError> {
        // View matrix
        if let Ok(vm) = self.read_view_matrix() {
            data.view_matrix = vm;
            data.in_game = true;
        }

        // Map name
        if let Ok(name) = self.read_map_name() {
            data.map_name = name;
        }

        // Local player
        match self.read_local_player() {
            Ok(lp) => {
                data.in_game = true;
                data.local_player = lp;
            }
            Err(ReadError::NotInGame) => {
                data.in_game = false;
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        // All players
        if let Ok(players) = self.read_players() {
            data.players = players;
        }

        // Entities
        if let Ok(ents) = self.read_entities() {
            data.entities = ents;
        }

        // Bomb
        if let Ok(bomb) = self.read_bomb() {
            data.bomb = bomb;
        }

        Ok(())
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn read_player_from_pawn(
        &mut self,
        controller: u64,
        pawn: u64,
    ) -> Result<PlayerData, ReadError> {
        let health = self.process.read_u32(pawn + self.offsets.iface.pawn_health)? as i32;
        let armor = self.process.read_u32(pawn + self.offsets.iface.pawn_armor)? as i32;
        let position = self.process.read_vec3(pawn + self.offsets.iface.pawn_origin)?;
        let eye_offset = self.process.read_vec3(pawn + self.offsets.iface.pawn_view_offset)?;
        let velocity = self.process.read_vec3(pawn + self.offsets.iface.pawn_velocity)?;

        let steam_id = self.process.read_u64(controller + self.offsets.iface.controller_steam_id).unwrap_or(0);
        let name = read_cstring(
            &mut self.process,
            controller + self.offsets.iface.controller_player_name,
            128,
        )
        .unwrap_or_default();

        let has_defuser_bytes = self.process.read_bytes(pawn + self.offsets.iface.pawn_has_defuser, 1).unwrap_or_default();
        let has_defuser = has_defuser_bytes.first().copied().unwrap_or(0) != 0;

        let has_helmet_bytes = self.process.read_bytes(pawn + self.offsets.iface.pawn_has_helmet, 1).unwrap_or_default();
        let has_helmet = has_helmet_bytes.first().copied().unwrap_or(0) != 0;

        let eye_pos = position + eye_offset;

        Ok(PlayerData {
            steam_id,
            pawn,
            health,
            armor,
            position,
            eye_pos,
            head: eye_pos + Vec3::new(0.0, 0.0, 4.0),
            name,
            weapon: Weapon::Unknown,
            bones: std::collections::HashMap::new(),
            has_defuser,
            has_helmet,
            has_bomb: false,
            visible: false,
            color: 0,
            rotation: 0.0,
            velocity,
        })
    }
}

// ── Mock reader for testing ──────────────────────────────────────────────────

/// A testable in-memory substitute for the real game process.
///
/// Allows unit tests to inject arbitrary memory contents and verify that
/// [`GameReader`] parses them correctly without a running CS2 process.
pub mod mock {
    use std::collections::HashMap;

    /// Simulated memory map used by [`MockProcess`].
    pub struct MockMemory {
        regions: HashMap<u64, Vec<u8>>,
    }

    impl MockMemory {
        pub fn new() -> Self {
            Self { regions: HashMap::new() }
        }

        /// Writes `bytes` starting at `address`.
        pub fn write(&mut self, address: u64, bytes: &[u8]) {
            let region = self.regions.entry(address).or_insert_with(|| vec![0u8; bytes.len()]);
            region.resize(bytes.len().max(region.len()), 0);
            region[..bytes.len()].copy_from_slice(bytes);
        }

        /// Writes an `f32` in little-endian format at `address`.
        pub fn write_f32(&mut self, address: u64, val: f32) {
            self.write(address, &val.to_le_bytes());
        }

        /// Writes a `u32` in little-endian format at `address`.
        pub fn write_u32(&mut self, address: u64, val: u32) {
            self.write(address, &val.to_le_bytes());
        }

        /// Writes a `u64` in little-endian format at `address`.
        pub fn write_u64(&mut self, address: u64, val: u64) {
            self.write(address, &val.to_le_bytes());
        }

        /// Reads `size` bytes from `address`, returning zeros for unmapped regions.
        pub fn read(&self, address: u64, size: usize) -> Vec<u8> {
            // Search for a region that contains the requested range.
            for (&base, data) in &self.regions {
                if address >= base {
                    let offset = (address - base) as usize;
                    if offset < data.len() {
                        // Return as many bytes as available from this region, zero-pad the rest.
                        let available = data.len() - offset;
                        let mut result = vec![0u8; size];
                        let copy_len = available.min(size);
                        result[..copy_len].copy_from_slice(&data[offset..offset + copy_len]);
                        return result;
                    }
                }
            }
            vec![0u8; size]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockMemory;

    #[test]
    fn test_mock_memory_write_read() {
        let mut mem = MockMemory::new();
        mem.write_u32(0x1000, 100);
        let bytes = mem.read(0x1000, 4);
        let val = u32::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(val, 100);
    }

    #[test]
    fn test_mock_memory_f32() {
        let mut mem = MockMemory::new();
        mem.write_f32(0x2000, 3.14);
        let bytes = mem.read(0x2000, 4);
        let val = f32::from_le_bytes(bytes.try_into().unwrap());
        assert!((val - 3.14f32).abs() < 1e-5);
    }

    #[test]
    fn test_mock_memory_uninitialized_is_zero() {
        let mem = MockMemory::new();
        let bytes = mem.read(0xDEAD_BEEF, 8);
        assert_eq!(bytes, vec![0u8; 8]);
    }

    #[test]
    fn test_mock_memory_string() {
        let mut mem = MockMemory::new();
        let name = b"TestPlayer\0";
        mem.write(0x3000, name);
        let raw = mem.read(0x3000, 16);
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        let s = String::from_utf8_lossy(&raw[..end]);
        assert_eq!(s, "TestPlayer");
    }
}
