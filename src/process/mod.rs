pub mod cache;
pub mod offsets;
pub mod offsets_discovery;

use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

use glam::Vec3;

/// Errors that can occur when interacting with a game process.
#[derive(Debug)]
pub enum ProcessError {
    /// The process could not be found or opened.
    ProcessNotFound(u32),
    /// A memory read failed at the given address.
    MemoryReadFailed { address: u64, size: usize, source: io::Error },
    /// Could not locate the requested module/library in the process map.
    ModuleNotFound(String),
    /// Generic I/O error.
    Io(io::Error),
    /// The process map file could not be parsed.
    MapParseError(String),
    /// Attempted to use a process that has not been opened.
    NotOpen,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::ProcessNotFound(pid) => write!(f, "process {pid} not found"),
            ProcessError::MemoryReadFailed { address, size, source } => {
                write!(f, "memory read failed at {address:#x} ({size} bytes): {source}")
            }
            ProcessError::ModuleNotFound(name) => write!(f, "module '{name}' not found"),
            ProcessError::Io(e) => write!(f, "I/O error: {e}"),
            ProcessError::MapParseError(msg) => write!(f, "map parse error: {msg}"),
            ProcessError::NotOpen => write!(f, "process is not open"),
        }
    }
}

impl std::error::Error for ProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProcessError::MemoryReadFailed { source, .. } => Some(source),
            ProcessError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ProcessError {
    fn from(e: io::Error) -> Self {
        ProcessError::Io(e)
    }
}

/// A loaded module (shared library) mapped in the process address space.
#[derive(Debug, Clone)]
pub struct Module {
    /// Library file name (e.g. `"libclient.so"`).
    pub name: String,
    /// Base virtual address of the first executable mapping.
    pub base: u64,
    /// Total size of all mappings belonging to this module.
    pub size: u64,
    /// Path on disk.
    pub path: PathBuf,
}

/// Platform-specific handle to an open process.
///
/// On Linux this is the PID and an open file handle to `/proc/<pid>/mem`.
pub struct ProcessHandle {
    pub pid: u32,
    mem_file: File,
}

/// Core process abstraction – holds a handle to the game process and provides
/// typed memory-read helpers built on top of raw byte reads.
pub struct Process {
    handle: Option<ProcessHandle>,
    pid: u32,
    modules: Vec<Module>,
}

impl Process {
    /// Creates a new, unopened `Process` for the given PID.
    pub fn new(pid: u32) -> Self {
        Self { handle: None, pid, modules: Vec::new() }
    }

    /// Opens the process with the given PID for memory reading.
    ///
    /// This opens `/proc/<pid>/mem` and reads the memory map.
    pub fn open(pid: u32) -> Result<Self, ProcessError> {
        let mem_path = format!("/proc/{pid}/mem");
        let mem_file = File::open(&mem_path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                ProcessError::ProcessNotFound(pid)
            } else {
                ProcessError::Io(e)
            }
        })?;

        let handle = ProcessHandle { pid, mem_file };
        let mut proc = Self { handle: Some(handle), pid, modules: Vec::new() };
        proc.refresh_modules()?;
        Ok(proc)
    }

    /// Returns the PID of the process.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Returns the list of loaded modules.
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    /// Reads `size` raw bytes from the process at `address`.
    pub fn read_bytes(&mut self, address: u64, size: usize) -> Result<Vec<u8>, ProcessError> {
        let handle = self.handle.as_mut().ok_or(ProcessError::NotOpen)?;
        let mut buf = vec![0u8; size];
        handle
            .mem_file
            .seek(SeekFrom::Start(address))
            .map_err(|e| ProcessError::MemoryReadFailed { address, size, source: e })?;
        handle
            .mem_file
            .read_exact(&mut buf)
            .map_err(|e| ProcessError::MemoryReadFailed { address, size, source: e })?;
        Ok(buf)
    }

    /// Reads a `u32` from the process at `address`.
    pub fn read_u32(&mut self, address: u64) -> Result<u32, ProcessError> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Reads a `u64` from the process at `address`.
    pub fn read_u64(&mut self, address: u64) -> Result<u64, ProcessError> {
        let bytes = self.read_bytes(address, 8)?;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Reads an `f32` from the process at `address`.
    pub fn read_f32(&mut self, address: u64) -> Result<f32, ProcessError> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(f32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Reads a [`glam::Vec3`] (three consecutive `f32` values) from `address`.
    pub fn read_vec3(&mut self, address: u64) -> Result<Vec3, ProcessError> {
        let x = self.read_f32(address)?;
        let y = self.read_f32(address + 4)?;
        let z = self.read_f32(address + 8)?;
        Ok(Vec3::new(x, y, z))
    }

    /// Returns the base address of the module with the given name, or an error
    /// if the module is not loaded.
    pub fn get_module(&self, name: &str) -> Result<u64, ProcessError> {
        self.modules
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.base)
            .ok_or_else(|| ProcessError::ModuleNotFound(name.to_owned()))
    }

    /// Re-reads `/proc/<pid>/maps` to update the cached module list.
    pub fn refresh_modules(&mut self) -> Result<(), ProcessError> {
        let maps_path = format!("/proc/{}/maps", self.pid);
        let content = std::fs::read_to_string(&maps_path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                ProcessError::ProcessNotFound(self.pid)
            } else {
                ProcessError::Io(e)
            }
        })?;

        self.modules = parse_proc_maps(&content)?;
        Ok(())
    }

    // ── Pattern scanning ─────────────────────────────────────────────────────

    /// Parses a byte pattern string with `?` wildcards into bytes and a mask.
    ///
    /// Input format: space-separated hex bytes where `?` means "any byte".
    ///
    /// Returns `(bytes, mask)` where `mask[i]` is `true` if byte `i` must match.
    pub fn parse_pattern(pattern: &str) -> (Vec<u8>, Vec<bool>) {
        let mut bytes = Vec::new();
        let mut mask = Vec::new();
        for token in pattern.split_whitespace() {
            if token == "?" || token == "??" {
                bytes.push(0u8);
                mask.push(false);
            } else {
                let byte = u8::from_str_radix(token, 16).unwrap_or(0);
                bytes.push(byte);
                mask.push(true);
            }
        }
        (bytes, mask)
    }

    /// Scans `size` bytes of process memory starting at `base` for the given
    /// byte pattern (with `?` wildcards).
    ///
    /// Returns the **absolute** address of the first match, or `None`.
    ///
    /// Pattern format: space-separated hex bytes, e.g. `"48 83 3D ? ? ? ? 00"`.
    pub fn scan(
        &mut self,
        pattern: &str,
        base: u64,
        size: usize,
    ) -> Option<u64> {
        let (pat_bytes, pat_mask) = Self::parse_pattern(pattern);
        if pat_bytes.is_empty() || pat_bytes.len() > size {
            return None;
        }

        let data = self.read_bytes(base, size).ok()?;
        let pat_len = pat_bytes.len();

        'outer: for i in 0..=data.len().saturating_sub(pat_len) {
            for j in 0..pat_len {
                if pat_mask[j] && data[i + j] != pat_bytes[j] {
                    continue 'outer;
                }
            }
            return Some(base + i as u64);
        }
        None
    }

    /// Resolves a RIP-relative address embedded inside an instruction.
    ///
    /// * `instruction` — absolute address of the instruction start.
    /// * `offset`      — byte offset from instruction start to the 4-byte
    ///                   relative operand (e.g. 3 for `48 83 3D [rip+rel]`).
    /// * `size`        — total size of the instruction (used to compute RIP).
    ///
    /// Returns the absolute target address: `instruction + size + rel32`.
    pub fn get_relative_address(
        &mut self,
        instruction: u64,
        offset: usize,
        size: usize,
    ) -> Result<u64, ProcessError> {
        let rel_bytes = self.read_bytes(instruction + offset as u64, 4)?;
        let rel32 = i32::from_le_bytes(rel_bytes.try_into().unwrap());
        let rip = instruction + size as u64;
        Ok(rip.wrapping_add(rel32 as i64 as u64))
    }

    /// Searches for a CS2 interface factory entry by name in `libXxx.so`.
    ///
    /// CS2 shared libraries expose a linked list of `InterfaceReg` structs
    /// through `CreateInterface`.  Each node contains a pointer to the
    /// instance-creator function, a name C-string, and a next pointer.
    ///
    /// * `base` — base address of the module (from `get_module`).
    /// * `name` — partial or exact interface name to search for.
    ///
    /// Returns the address of the matching `InterfaceReg::m_CreateFn`, or
    /// `None` if not found.
    pub fn get_interface_offset(
        &mut self,
        base: u64,
        name: &str,
    ) -> Option<u64> {
        // Look up the `CreateInterface` export to find the head of the list.
        let create_iface = self.get_module_export(base, "CreateInterface")?;

        // `CreateInterface` contains a JMP into `CreateInterfaceInternal`,
        // which holds the list-head pointer.  Read the 4-byte relative offset
        // at byte 1 of the first JMP instruction.
        let internal = self.get_relative_address(create_iface, 1, 5).ok()?;
        // The list-head pointer is a RIP-relative load inside the function;
        // offset 3, instruction size 7 is a common pattern.
        let list_head_ptr = self.get_relative_address(internal, 3, 7).ok()?;
        let mut node = self.read_u64(list_head_ptr).ok()?;

        for _ in 0..1024 {
            if node == 0 {
                break;
            }
            // InterfaceReg layout: [fn_ptr: u64, name_ptr: u64, next: u64]
            let name_ptr = self.read_u64(node + 8).ok()?;
            let raw = self.read_bytes(name_ptr, 128).ok()?;
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            let entry_name = std::str::from_utf8(&raw[..end]).unwrap_or("");
            if entry_name.starts_with(name) {
                return Some(node);
            }
            node = self.read_u64(node + 16).ok()?;
        }
        None
    }

    /// Parses the ELF `.dynsym` / `.dynstr` sections of a module to find an
    /// exported symbol by name.
    ///
    /// * `base` — base address of the mapped module.
    /// * `name` — symbol name to look up.
    ///
    /// Returns the absolute virtual address of the symbol, or `None`.
    pub fn get_module_export(
        &mut self,
        base: u64,
        name: &str,
    ) -> Option<u64> {
        // Read the ELF64 header (64 bytes).
        let ehdr = self.read_bytes(base, 64).ok()?;
        // Verify ELF magic.
        if &ehdr[0..4] != b"\x7fELF" {
            return None;
        }
        let e_phoff = u64::from_le_bytes(ehdr[32..40].try_into().ok()?);
        let e_phentsize = u16::from_le_bytes(ehdr[54..56].try_into().ok()?) as u64;
        let e_phnum = u16::from_le_bytes(ehdr[56..58].try_into().ok()?) as u64;

        // Walk program headers to find PT_DYNAMIC (type == 2).
        let (dyn_addr, dyn_size) =
            self.get_segment_from_pht(base, e_phoff, e_phentsize, e_phnum, 2)?;

        // Parse DYNAMIC section for DT_SYMTAB (6), DT_STRTAB (5), DT_STRSZ (10).
        let sym_addr = self.get_address_from_dynamic_section(base, dyn_addr, dyn_size, 6)?;
        let str_addr = self.get_address_from_dynamic_section(base, dyn_addr, dyn_size, 5)?;
        let str_size = self.get_address_from_dynamic_section(base, dyn_addr, dyn_size, 10)
            .unwrap_or(4096);

        // Read string table.
        let str_table = self.read_bytes(base + str_addr, str_size as usize).ok()?;

        // Walk symbol table (each Elf64_Sym is 24 bytes).
        let sym_size = 24usize;
        for i in 0..4096usize {
            let sym = self.read_bytes(base + sym_addr + (i * sym_size) as u64, sym_size).ok()?;
            let st_name = u32::from_le_bytes(sym[0..4].try_into().ok()?) as usize;
            let st_value = u64::from_le_bytes(sym[8..16].try_into().ok()?);
            if st_value == 0 {
                continue;
            }
            let end = str_table[st_name..].iter().position(|&b| b == 0).unwrap_or(0);
            let sym_name = std::str::from_utf8(&str_table[st_name..st_name + end]).unwrap_or("");
            if sym_name == name {
                return Some(base + st_value);
            }
        }
        None
    }

    /// Reads `DT_NEEDED`-style entries from a module's `.dynamic` segment.
    ///
    /// Returns the value associated with the dynamic tag `tag`, or `None`.
    pub fn get_address_from_dynamic_section(
        &mut self,
        _base: u64,
        dyn_addr: u64,
        dyn_size: u64,
        tag: i64,
    ) -> Option<u64> {
        // Each Elf64_Dyn entry is 16 bytes: [d_tag: i64, d_val/d_ptr: u64].
        let count = (dyn_size / 16) as usize;
        for i in 0..count {
            let entry = self.read_bytes(dyn_addr + (i * 16) as u64, 16).ok()?;
            let d_tag = i64::from_le_bytes(entry[0..8].try_into().ok()?);
            let d_val = u64::from_le_bytes(entry[8..16].try_into().ok()?);
            if d_tag == tag {
                return Some(d_val);
            }
            if d_tag == 0 {
                // DT_NULL terminates the array.
                break;
            }
        }
        None
    }

    /// Returns the `(vaddr, filesz)` of the program-header segment with type
    /// `pt_type`, or `None` if not present.
    pub fn get_segment_from_pht(
        &mut self,
        base: u64,
        phoff: u64,
        phentsize: u64,
        phnum: u64,
        pt_type: u32,
    ) -> Option<(u64, u64)> {
        // ELF64 program header: 56 bytes.
        // p_type at offset 0 (u32), p_vaddr at offset 16 (u64), p_filesz at 32 (u64).
        for i in 0..phnum {
            let phdr = self.read_bytes(base + phoff + i * phentsize, 56).ok()?;
            let p_type = u32::from_le_bytes(phdr[0..4].try_into().ok()?);
            if p_type == pt_type {
                let p_vaddr = u64::from_le_bytes(phdr[16..24].try_into().ok()?);
                let p_filesz = u64::from_le_bytes(phdr[32..40].try_into().ok()?);
                return Some((p_vaddr, p_filesz));
            }
        }
        None
    }

    /// Looks up a `ConVar` by name through the `ICvar` interface.
    ///
    /// * `interface` — pointer to the `ICvar` vtable instance.
    /// * `name`      — console variable name (e.g. `"mp_teammates_are_enemies"`).
    ///
    /// Returns the address of the `ConVar` object, or `None`.
    pub fn get_convar(
        &mut self,
        interface: u64,
        name: &str,
    ) -> Option<u64> {
        if interface == 0 {
            return None;
        }
        // ICvar::FindVar is typically vtable slot 16 (offset 0x80 for 64-bit).
        // We can't call the function directly from a read-only process, so we
        // walk the cvar linked list at ICvar + 0x40 instead.
        let _vtable = self.read_u64(interface).ok()?;

        // Walk the flat cvar linked list stored at interface + 0x40.
        let list_head_offset: u64 = 0x40;
        let mut node = self.read_u64(interface + list_head_offset).ok()?;

        for _ in 0..8192 {
            if node == 0 {
                break;
            }
            // ConVar name pointer is at offset 0 of the ConVar object.
            let name_ptr = self.read_u64(node).ok()?;
            if name_ptr == 0 {
                // Advance via next pointer at offset 0x18.
                node = self.read_u64(node + 0x18).ok()?;
                continue;
            }
            let raw = self.read_bytes(name_ptr, 256).ok()?;
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            let cvar_name = std::str::from_utf8(&raw[..end]).unwrap_or("");
            if cvar_name == name {
                return Some(node);
            }
            node = self.read_u64(node + 0x18).ok()?;
        }
        None
    }
}

/// Parses `/proc/<pid>/maps` content into a list of `Module` entries.
///
/// Each unique library path gets one entry with the lowest base address found.
pub fn parse_proc_maps(maps: &str) -> Result<Vec<Module>, ProcessError> {
    let mut modules: std::collections::HashMap<String, Module> = std::collections::HashMap::new();

    for line in maps.lines() {
        // Format: address perms offset dev inode pathname
        // e.g.:  7f1234000000-7f1234001000 r-xp 00000000 08:01 12345  /usr/lib/libfoo.so.1
        let parts: Vec<&str> = line.splitn(6, ' ').collect();
        if parts.len() < 6 {
            continue;
        }
        let pathname = parts[5].trim();
        if pathname.is_empty() || !pathname.starts_with('/') {
            continue;
        }

        let addr_range = parts[0];
        let dash = addr_range
            .find('-')
            .ok_or_else(|| ProcessError::MapParseError(format!("bad address range: {addr_range}")))?;
        let start = u64::from_str_radix(&addr_range[..dash], 16)
            .map_err(|_| ProcessError::MapParseError(format!("bad start address: {addr_range}")))?;
        let end = u64::from_str_radix(&addr_range[dash + 1..], 16)
            .map_err(|_| ProcessError::MapParseError(format!("bad end address: {addr_range}")))?;

        let size = end.saturating_sub(start);
        let path = PathBuf::from(pathname);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(pathname)
            .to_owned();

        modules
            .entry(name.clone())
            .and_modify(|m| {
                if start < m.base {
                    m.base = start;
                }
                m.size += size;
            })
            .or_insert(Module { name, base: start, size, path });
    }

    Ok(modules.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MAPS: &str = "\
7f1234000000-7f1234001000 r-xp 00000000 08:01 12345  /usr/lib/libfoo.so.1
7f1234001000-7f1234002000 r--p 00001000 08:01 12345  /usr/lib/libfoo.so.1
7f2000000000-7f2000010000 r-xp 00000000 08:01 99999  /usr/lib/libbar.so.2
";

    #[test]
    fn test_parse_proc_maps() {
        let modules = parse_proc_maps(SAMPLE_MAPS).unwrap();
        let foo = modules.iter().find(|m| m.name == "libfoo.so.1");
        assert!(foo.is_some(), "libfoo.so.1 should be found");
        let foo = foo.unwrap();
        assert_eq!(foo.base, 0x7f1234000000);
        assert_eq!(foo.size, 0x2000);

        let bar = modules.iter().find(|m| m.name == "libbar.so.2");
        assert!(bar.is_some(), "libbar.so.2 should be found");
        assert_eq!(bar.unwrap().base, 0x7f2000000000);
    }

    #[test]
    fn test_parse_proc_maps_skips_anonymous() {
        let maps = "7fff00000000-7fff00010000 rwxp 00000000 00:00 0\n";
        let modules = parse_proc_maps(maps).unwrap();
        assert!(modules.is_empty(), "anonymous mappings should be skipped");
    }

    #[test]
    fn test_module_not_found() {
        let proc = Process { handle: None, pid: 0, modules: vec![] };
        let result = proc.get_module("libclient.so");
        assert!(matches!(result, Err(ProcessError::ModuleNotFound(_))));
    }

    #[test]
    fn test_parse_pattern_basic() {
        let (bytes, mask) = Process::parse_pattern("48 83 3D ? ? 00 FF");
        assert_eq!(bytes, vec![0x48, 0x83, 0x3D, 0x00, 0x00, 0x00, 0xFF]);
        assert_eq!(mask, vec![true, true, true, false, false, true, true]);
    }

    #[test]
    fn test_parse_pattern_all_wildcards() {
        let (bytes, mask) = Process::parse_pattern("? ? ?");
        assert_eq!(bytes.len(), 3);
        assert!(mask.iter().all(|&m| !m));
    }

    #[test]
    fn test_parse_pattern_empty() {
        let (bytes, mask) = Process::parse_pattern("");
        assert!(bytes.is_empty());
        assert!(mask.is_empty());
    }
}
