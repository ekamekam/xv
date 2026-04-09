pub mod cache;
pub mod offsets;

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
}
