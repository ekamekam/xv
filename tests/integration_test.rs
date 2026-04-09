//! Integration tests for the memory reading layer.
//!
//! These tests use the `MockMemory` helper from `xv::reader::mock` and
//! validate the full parsing pipeline without requiring a real CS2 process.

use xv::process::offsets::Offsets;
use xv::process::cache::MemoryCache;
use xv::schema::Schema;

// ── Schema tests ─────────────────────────────────────────────────────────────

#[test]
fn test_schema_default_loads_player_pawn() {
    let mut schema = Schema::new();
    schema.load_defaults();
    let cls = schema.get_class("C_CSPlayerPawn");
    assert!(cls.is_some(), "default schema must contain C_CSPlayerPawn");
}

#[test]
fn test_schema_field_offsets_match_offsets_struct() {
    let offsets = Offsets::load();
    let mut schema = Schema::new();
    schema.load_defaults();

    // Health offset in both sources must agree.
    let schema_health = schema.get_field_offset("C_CSPlayerPawn", "m_iHealth");
    assert_eq!(
        schema_health,
        Some(offsets.iface.pawn_health),
        "health offset should be consistent between Schema and Offsets"
    );
}

// ── Cache tests ───────────────────────────────────────────────────────────────

#[test]
fn test_cache_full_round_trip() {
    use std::time::Duration;
    let mut cache = MemoryCache::new(Duration::from_secs(5), 256);
    cache.insert(0xABCD_EF00, 4, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let got = cache.get(0xABCD_EF00, 4).unwrap();
    assert_eq!(got, &[0xDEu8, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn test_cache_capacity_eviction() {
    use std::time::Duration;
    let max = 4;
    let mut cache = MemoryCache::new(Duration::from_secs(60), max);
    for i in 0..10u64 {
        cache.insert(i * 0x1000, 4, vec![i as u8; 4]);
    }
    // After overflow the old entries are gone; the cache should have <= max entries.
    assert!(cache.stats().entries <= max);
}

// ── Offsets tests ─────────────────────────────────────────────────────────────

#[test]
fn test_offsets_direct_entity_list_nonzero() {
    let o = Offsets::load();
    assert_ne!(o.direct.entity_list, 0);
}

#[test]
fn test_offsets_resolve_from_binary_is_stable() {
    let o = Offsets::load();
    let o2 = o.resolve_from_binary(0x7f0000000000, &[]);
    assert_eq!(o.direct.view_matrix, o2.direct.view_matrix);
}

// ── Mock memory reader ────────────────────────────────────────────────────────

#[test]
fn test_mock_memory_read_write_cycle() {
    use xv::reader::mock::MockMemory;

    let mut mem = MockMemory::new();
    mem.write_u64(0x1000, 0xCAFE_BABE_DEAD_BEEF);
    let bytes = mem.read(0x1000, 8);
    let val = u64::from_le_bytes(bytes.try_into().unwrap());
    assert_eq!(val, 0xCAFE_BABE_DEAD_BEEF);
}

#[test]
fn test_mock_memory_vec3() {
    use xv::reader::mock::MockMemory;

    let mut mem = MockMemory::new();
    let base = 0x5000u64;
    mem.write_f32(base, 1.0);
    mem.write_f32(base + 4, 2.0);
    mem.write_f32(base + 8, 3.0);

    let x = f32::from_le_bytes(mem.read(base, 4).try_into().unwrap());
    let y = f32::from_le_bytes(mem.read(base + 4, 4).try_into().unwrap());
    let z = f32::from_le_bytes(mem.read(base + 8, 4).try_into().unwrap());
    assert_eq!((x, y, z), (1.0f32, 2.0, 3.0));
}
