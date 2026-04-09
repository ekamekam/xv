//! Example: reading game state from a running CS2 process.
//!
//! Run this with:
//!
//! ```sh
//! # (requires root / ptrace_scope=0)
//! sudo cargo run --example read_game_state -- <CS2_PID>
//! ```

use std::process as std_process;

use xv::data::Data;
use xv::process::{offsets_discovery::discover_offsets, Process};
use xv::reader::GameReader;

fn main() {
    let pid: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("Usage: read_game_state <PID>");
            eprintln!("Tip: find the CS2 PID with: pgrep cs2");
            std_process::exit(1);
        });

    println!("[xv] Attaching to process {pid} …");

    let mut process = match Process::open(pid) {
        Ok(p) => {
            println!("[xv] Opened process {} ({} modules loaded)", p.pid(), p.modules().len());
            p
        }
        Err(e) => {
            eprintln!("[xv] Failed to open process: {e}");
            eprintln!("     Make sure you have permission to read /proc/{pid}/mem");
            std_process::exit(1);
        }
    };

    println!("[xv] Discovering offsets dynamically …");
    let offsets = match xv::process::offsets_discovery::discover_offsets(&mut process) {
        Ok(o) => {
            println!("[xv] ✓ Offsets discovered successfully");
            o
        }
        Err(e) => {
            eprintln!("[xv] Offset discovery failed: {e}");
            eprintln!("[xv] Falling back to hardcoded offsets");
            xv::process::offsets::Offsets::load()
        }
    };

    let mut reader = match GameReader::new(process, offsets) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[xv] Failed to create reader: {e}");
            std_process::exit(1);
        }
    };

    println!("[xv] Reading game state …");

    let mut data = Data::default();
    match reader.update_game_data(&mut data) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[xv] update_game_data error: {e}");
        }
    }

    println!("in_game:    {}", data.in_game);
    println!("map:        {}", data.map_name);
    println!("players:    {}", data.players.len());
    println!(
        "local hp:   {}  name: {}",
        data.local_player.health, data.local_player.name
    );
    println!("entities:   {}", data.entities.len());
}
