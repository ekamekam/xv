use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A single entry in the memory cache.
struct CacheEntry {
    data: Vec<u8>,
    inserted_at: Instant,
}

/// Cache statistics snapshot.
#[derive(Debug, Default, Clone, Copy)]
pub struct CacheStats {
    /// Total number of cache lookups.
    pub lookups: u64,
    /// Number of lookups that found a valid (non-expired) entry.
    pub hits: u64,
    /// Number of lookups that missed (entry absent or expired).
    pub misses: u64,
    /// Current number of entries stored in the cache.
    pub entries: usize,
}

impl CacheStats {
    /// Hit rate as a value in `[0.0, 1.0]`. Returns `0.0` when there are no lookups.
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            self.hits as f64 / self.lookups as f64
        }
    }
}

/// LRU-style memory cache that reduces repeated `/proc/<pid>/mem` reads.
///
/// Entries expire after `ttl` duration and are evicted lazily on the next
/// lookup or on an explicit `invalidate()` call.
pub struct MemoryCache {
    entries: HashMap<(u64, usize), CacheEntry>,
    ttl: Duration,
    stats: CacheStats,
    max_entries: usize,
}

impl MemoryCache {
    /// Creates a new cache with the given time-to-live per entry and capacity.
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            stats: CacheStats::default(),
            max_entries,
        }
    }

    /// Creates a cache with a 16 ms TTL and space for 1 024 entries — reasonable
    /// defaults for a ~60 Hz read loop.
    pub fn default_config() -> Self {
        Self::new(Duration::from_millis(16), 1024)
    }

    /// Returns cached bytes for `(address, size)` if the entry exists and has
    /// not expired; otherwise returns `None`.
    pub fn get(&mut self, address: u64, size: usize) -> Option<&[u8]> {
        self.stats.lookups += 1;
        let ttl = self.ttl;

        if let Some(entry) = self.entries.get(&(address, size)) {
            if entry.inserted_at.elapsed() < ttl {
                self.stats.hits += 1;
                // Re-borrow immutably for the return value.
                return self.entries.get(&(address, size)).map(|e| e.data.as_slice());
            }
            // Expired — fall through to remove and return None.
            self.entries.remove(&(address, size));
        }
        self.stats.misses += 1;
        None
    }

    /// Inserts or updates a cache entry.
    ///
    /// If the cache is at capacity the entire cache is cleared to make room
    /// (simple eviction policy).
    pub fn insert(&mut self, address: u64, size: usize, data: Vec<u8>) {
        if self.entries.len() >= self.max_entries {
            self.entries.clear();
        }
        self.entries.insert(
            (address, size),
            CacheEntry { data, inserted_at: Instant::now() },
        );
        self.stats.entries = self.entries.len();
    }

    /// Removes all cached entries.
    pub fn invalidate(&mut self) {
        self.entries.clear();
        self.stats.entries = 0;
    }

    /// Returns a snapshot of current cache statistics.
    pub fn stats(&self) -> CacheStats {
        let mut s = self.stats;
        s.entries = self.entries.len();
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_miss_on_empty() {
        let mut cache = MemoryCache::new(Duration::from_secs(10), 100);
        assert!(cache.get(0xDEAD_BEEF, 4).is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_cache_hit_after_insert() {
        let mut cache = MemoryCache::new(Duration::from_secs(10), 100);
        cache.insert(0x1000, 4, vec![1, 2, 3, 4]);
        let result = cache.get(0x1000, 4);
        assert_eq!(result, Some(&[1u8, 2, 3, 4][..]));
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_cache_expiry() {
        let mut cache = MemoryCache::new(Duration::from_nanos(1), 100);
        cache.insert(0x1000, 4, vec![0xAA; 4]);
        // Tiny TTL should have elapsed by the time we read it.
        std::thread::sleep(Duration::from_millis(1));
        assert!(cache.get(0x1000, 4).is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_invalidate_clears_all() {
        let mut cache = MemoryCache::new(Duration::from_secs(10), 100);
        cache.insert(0x1000, 4, vec![0; 4]);
        cache.insert(0x2000, 8, vec![0; 8]);
        cache.invalidate();
        assert_eq!(cache.stats().entries, 0);
        assert!(cache.get(0x1000, 4).is_none());
    }

    #[test]
    fn test_hit_rate() {
        let mut cache = MemoryCache::new(Duration::from_secs(10), 100);
        cache.insert(0x1000, 4, vec![0; 4]);
        cache.get(0x1000, 4); // hit
        cache.get(0x2000, 4); // miss
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.5).abs() < f64::EPSILON);
    }
}
