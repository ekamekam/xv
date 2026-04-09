//! Example: reading game state from a running CS2 process.
//!
//! Run this with:
//!
//! ```sh
//! # (requires root / ptrace_scope=0)
//! sudo cargo run --example read_game_state -- <CS2_PID>
//! ```
//!
//! Without a real CS2 process, the example shows how to wire everything
//! together using the library's public API.

use std::process as std_process;

use xv::data::Data;
use xv::process::{offsets::Offsets, Process};
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

    let process = match Process::open(pid) {
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

    // List loaded modules.
    for module in process.modules() {
        println!("  {:#018x}  +{:#010x}  {}", module.base, module.size, module.name);
    }

    let offsets = Offsets::load();
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
    println!(
        "bomb:       planted={}  timer={:.1}s",
        data.bomb.planted, data.bomb.timer
    );
    println!("entities:   {}", data.entities.len());
}
