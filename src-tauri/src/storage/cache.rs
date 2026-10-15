use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

// ==================== CACHE ENTRY ====================

struct CacheEntry {
    data: String,
    cached_at: Instant,
    hits: u64,
}

impl CacheEntry {
    fn new(data: String) -> Self {
        Self {
            data,
            cached_at: Instant::now(),
            hits: 0,
        }
    }

    fn age_secs(&self) -> u64 {
        self.cached_at.elapsed().as_secs()
    }
}

// ==================== STORAGE CACHE ====================

pub struct StorageCache {
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl StorageCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// ✅ Get or load from file
    pub fn get_or_load<F>(&self, path: &str, loader: F) -> Result<String, String>
    where
        F: FnOnce() -> Result<String, String>,
    {
        // Check cache first
        {
            let mut cache = self.cache.write().map_err(|e| e.to_string())?;
            if let Some(entry) = cache.get_mut(path) {
                entry.hits += 1;
                // Cache TTL: 5 minutes
                if entry.age_secs() < 300 {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Load from disk
        let data = loader()?;

        // Insert into cache
        {
            let mut cache = self.cache.write().map_err(|e| e.to_string())?;
            cache.insert(path.to_string(), CacheEntry::new(data.clone()));
        }

        Ok(data)
    }

    /// ✅ Update cache entry
    pub fn update(&self, path: &str, data: String) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(path.to_string(), CacheEntry::new(data));
        }
    }

    /// ✅ Invalidate specific path
    pub fn invalidate(&self, path: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(path);
        }
    }

    /// ✅ Clear all cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// ✅ Cache statistics
    pub fn stats(&self) -> (usize, u64) {
        if let Ok(cache) = self.cache.read() {
            let total_hits: u64 = cache.values().map(|e| e.hits).sum();
            (cache.len(), total_hits)
        } else {
            (0, 0)
        }
    }
}