//! xv — framework-agnostic CS2 game logic library (Phase 1–3).
//!
//! This crate provides core data structures, math utilities, CS2-specific
//! constants/enums, memory reading, and a UI integration layer — all
//! independent of any specific UI or OS framework.

pub mod constants;
pub mod cs2;
pub mod data;
pub mod math;

// Phase 2 — memory reading
pub mod process;
pub mod reader;
pub mod schema;

// Phase 3 — UI integration layer
pub mod config;
pub mod events;
pub mod features;
pub mod state;
pub mod ui;
