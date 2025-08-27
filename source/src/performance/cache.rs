//! Advanced validation caching system
//!
//! This module provides a sophisticated caching layer for validation results with:
//! - TTL (Time To Live) for cache entries
//! - Size-based eviction policies (LRU)
//! - Persistent cache storage
//! - Cache optimization and statistics

use anyhow::{Result, anyhow};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use blake3::Hasher;
use std::fs::{self, File};
use std::io::Read;
use serde::{Serialize, Deserialize};

/// Configuration for the validation cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries to keep in memory
    pub max_entries: usize,
    
    /// TTL for cache entries in seconds
    pub ttl_seconds: u64,
    
    /// Maximum cache file size in MB
    pub max_file_size_mb: usize,
    
    /// Enable persistent cache storage
    pub persistent: bool,
    
    /// Cache directory path
    pub cache_dir: Option<PathBuf>,
    
    /// Enable cache compression
    pub compress: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            ttl_seconds: 3600, // 1 hour
            max_file_size_mb: 50,
            persistent: true,
            cache_dir: None, // Will use default cache directory
            compress: true,
        }
    }
}

/// A single cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// File content hash
    pub hash: String,
    
    /// Validation result
    pub is_valid: bool,
    
    /// Timestamp when entry was created
    pub timestamp: u64,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// Validation duration in milliseconds
    pub validation_duration_ms: u64,
    
    /// Access count for LRU eviction
    pub access_count: u64,
    
    /// Last access timestamp
    pub last_accessed: u64,
}

impl CacheEntry {
    pub fn new(hash: String, is_valid: bool, file_size: u64, validation_duration: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            hash,
            is_valid,
            timestamp: now,
            file_size,
            validation_duration_ms: validation_duration.as_millis() as u64,
            access_count: 1,
            last_accessed: now,
        }
    }
    
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - self.timestamp > ttl_seconds
    }
    
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expired_entries: u64,
    pub total_memory_mb: f64,
    pub average_validation_time_ms: f64,
    pub hit_ratio: f64,
}

impl CacheStats {
    pub fn update_hit_ratio(&mut self) {
        let total = self.hits + self.misses;
        self.hit_ratio = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Advanced validation cache with TTL, LRU eviction, and persistence
pub struct ValidationCache {
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    access_order: Arc<Mutex<VecDeque<PathBuf>>>,
    stats: Arc<RwLock<CacheStats>>,
    cache_file: PathBuf,
}

impl ValidationCache {
    /// Create a new validation cache with the given configuration
    pub fn new(config: CacheConfig) -> Result<Self> {
        let cache_dir = config.cache_dir.clone()
            .or_else(|| dirs::cache_dir().map(|d| d.join("synx")))
            .unwrap_or_else(|| PathBuf::from(".cache/synx"));
        
        fs::create_dir_all(&cache_dir)?;
        let cache_file = cache_dir.join("validation_cache.json");
        
        let entries = if config.persistent && cache_file.exists() {
            Self::load_cache(&cache_file)?
        } else {
            HashMap::new()
        };
        
        let mut stats = CacheStats::default();
        stats.total_entries = entries.len();
        
        Ok(Self {
            config,
            entries: Arc::new(RwLock::new(entries)),
            access_order: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(stats)),
            cache_file,
        })
    }
    
    /// Check if a file's validation result is cached and still valid
    pub fn get(&self, path: &Path) -> Option<bool> {
        let file_hash = Self::compute_file_hash(path)?;
        
        let mut entries = self.entries.write().ok()?;
        let mut stats = self.stats.write().ok()?;
        
        if let Some(entry) = entries.get_mut(path) {
            // Check if entry is expired
            if entry.is_expired(self.config.ttl_seconds) {
                entries.remove(path);
                stats.expired_entries += 1;
                stats.misses += 1;
                return None;
            }
            
            // Check if file content has changed
            if entry.hash != file_hash {
                entries.remove(path);
                stats.misses += 1;
                return None;
            }
            
            // Update access statistics
            entry.access();
            let _ = self.update_access_order(path); // Ignore access order update failures
            
            stats.hits += 1;
            stats.update_hit_ratio();
            
            Some(entry.is_valid)
        } else {
            stats.misses += 1;
            stats.update_hit_ratio();
            None
        }
    }
    
    /// Cache a validation result for a file
    pub fn put(&self, path: &Path, is_valid: bool, validation_duration: Duration) -> Result<()> {
        let file_hash = Self::compute_file_hash(path)
            .ok_or_else(|| anyhow!("Failed to compute file hash"))?;
        
        let file_size = fs::metadata(path)?.len();
        let entry = CacheEntry::new(file_hash, is_valid, file_size, validation_duration);
        
        {
            let mut entries = self.entries.write().map_err(|_| anyhow!("Failed to lock entries"))?;
            let mut stats = self.stats.write().map_err(|_| anyhow!("Failed to lock stats"))?;
            
            // Check if we need to evict entries
            if entries.len() >= self.config.max_entries {
                self.evict_lru_entries(&mut entries, &mut stats)?;
            }
            
            entries.insert(path.to_path_buf(), entry);
            stats.total_entries = entries.len();
        }
        
        let _ = self.update_access_order(path); // Ignore access order update failures
        
        Ok(())
    }
    
    /// Clear all cache entries
    pub fn clear(&self) -> Result<()> {
        {
            let mut entries = self.entries.write().map_err(|_| anyhow!("Failed to lock entries"))?;
            entries.clear();
        }
        
        {
            let mut access_order = self.access_order.lock().map_err(|_| anyhow!("Failed to lock access order"))?;
            access_order.clear();
        }
        
        {
            let mut stats = self.stats.write().map_err(|_| anyhow!("Failed to lock stats"))?;
            *stats = CacheStats::default();
        }
        
        if self.config.persistent {
            fs::remove_file(&self.cache_file).ok();
        }
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let stats = match self.stats.read() {
            Ok(stats) => stats,
            Err(_) => return CacheStats::default(),
        };
        let mut stats_clone = stats.clone();
        
        // Update memory usage estimate
        if let Ok(entries) = self.entries.read() {
            stats_clone.total_memory_mb = self.estimate_memory_usage(&entries);
            
            // Update average validation time
            if !entries.is_empty() {
                let total_time: u64 = entries.values()
                    .map(|e| e.validation_duration_ms)
                    .sum();
                stats_clone.average_validation_time_ms = total_time as f64 / entries.len() as f64;
            }
        }
        
        stats_clone
    }
    
    /// Optimize cache by removing expired entries and updating statistics
    pub fn optimize(&self) -> Result<()> {
        let expired_keys: Vec<PathBuf>;
        
        {
            let mut entries = self.entries.write().map_err(|_| anyhow!("Failed to lock entries"))?;
            let mut stats = self.stats.write().map_err(|_| anyhow!("Failed to lock stats"))?;
            let mut access_order = self.access_order.lock().map_err(|_| anyhow!("Failed to lock access order"))?;
            
            // Remove expired entries
            expired_keys = entries.iter()
                .filter(|(_, entry)| entry.is_expired(self.config.ttl_seconds))
                .map(|(path, _)| path.clone())
                .collect();
                
            for key in &expired_keys {
                entries.remove(key);
                access_order.retain(|p| p != key);
            }
            
            stats.expired_entries += expired_keys.len() as u64;
            stats.total_entries = entries.len();
            
            // Save cache if persistent
            if self.config.persistent {
                self.save_cache(&entries)?
            }
        }
        
        Ok(())
    }
    
    /// Estimate memory usage of cache entries
    pub fn estimated_memory_usage(&self) -> usize {
        match self.entries.read() {
            Ok(entries) => (self.estimate_memory_usage(&entries) as usize).max(1),
            Err(_) => 1, // Return minimum value on lock failure
        }
    }
    
    fn estimate_memory_usage(&self, entries: &HashMap<PathBuf, CacheEntry>) -> f64 {
        let entry_size = std::mem::size_of::<CacheEntry>() + 
                        std::mem::size_of::<PathBuf>() + 
                        64; // Estimated hash and path string sizes
        
        (entries.len() * entry_size) as f64 / (1024.0 * 1024.0) // Convert to MB
    }
    
    fn compute_file_hash(path: &Path) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut hasher = Hasher::new();
        let mut buffer = [0; 8192];
        
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(_) => return None,
            }
        }
        
        Some(hasher.finalize().to_hex().to_string())
    }
    
    fn update_access_order(&self, path: &Path) -> Result<()> {
        let mut access_order = self.access_order.lock()
            .map_err(|_| anyhow!("Failed to lock access order for update"))?;
        
        // Remove existing entry if present
        access_order.retain(|p| p != path);
        // Add to front (most recently used)
        access_order.push_front(path.to_path_buf());
        
        // Keep access order list reasonable in size
        while access_order.len() > self.config.max_entries {
            access_order.pop_back();
        }
        
        Ok(())
    }
    
    fn evict_lru_entries(&self, entries: &mut HashMap<PathBuf, CacheEntry>, stats: &mut CacheStats) -> Result<()> {
        let target_size = (self.config.max_entries * 9) / 10; // Evict to 90% capacity
        let mut access_order = self.access_order.lock().map_err(|_| anyhow!("Failed to lock access order"))?;
        
        while entries.len() > target_size {
            if let Some(lru_path) = access_order.pop_back() {
                entries.remove(&lru_path);
                stats.evictions += 1;
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    fn load_cache(cache_file: &Path) -> Result<HashMap<PathBuf, CacheEntry>> {
        if !cache_file.exists() {
            return Ok(HashMap::new());
        }
        
        match fs::read_to_string(cache_file) {
            Ok(content) => {
                match serde_json::from_str::<HashMap<PathBuf, CacheEntry>>(&content) {
                    Ok(entries) => Ok(entries),
                    Err(e) => {
                        eprintln!("Warning: Failed to deserialize cache file ({}), starting with fresh cache", e);
                        // Remove corrupted cache file
                        let _ = fs::remove_file(cache_file);
                        Ok(HashMap::new())
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read cache file ({}), starting with fresh cache", e);
                Ok(HashMap::new())
            }
        }
    }
    
    fn save_cache(&self, entries: &HashMap<PathBuf, CacheEntry>) -> Result<()> {
        let json = serde_json::to_string_pretty(entries)?;
        fs::write(&self.cache_file, json)?;
        Ok(())
    }
}

impl Drop for ValidationCache {
    fn drop(&mut self) {
        if self.config.persistent {
            if let Ok(entries) = self.entries.read() {
                let _ = self.save_cache(&entries);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::io::Write;
    
    #[test]
    fn test_cache_creation() {
        let config = CacheConfig::default();
        let cache = ValidationCache::new(config);
        assert!(cache.is_ok());
    }
    
    #[test]
    fn test_cache_put_get() {
        let config = CacheConfig {
            persistent: false,
            ..Default::default()
        };
        let cache = ValidationCache::new(config).unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        temp_file.as_file().write_all(b"test content").unwrap();
        
        let path = temp_file.path();
        let duration = Duration::from_millis(100);
        
        cache.put(path, true, duration).unwrap();
        let result = cache.get(path);
        assert_eq!(result, Some(true));
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }
    
    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig {
            ttl_seconds: 0, // Immediate expiration
            persistent: false,
            ..Default::default()
        };
        let cache = ValidationCache::new(config).unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        temp_file.as_file().write_all(b"test content").unwrap();
        
        let path = temp_file.path();
        let duration = Duration::from_millis(100);
        
        cache.put(path, true, duration).unwrap();
        
        // Wait a bit to ensure expiration
        std::thread::sleep(Duration::from_millis(10));
        
        let result = cache.get(path);
        assert_eq!(result, None);
        
        let stats = cache.get_stats();
        assert_eq!(stats.expired_entries, 1);
    }
    
    #[test]
    fn test_cache_clear() {
        let config = CacheConfig {
            persistent: false,
            ..Default::default()
        };
        let cache = ValidationCache::new(config).unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        temp_file.as_file().write_all(b"test content").unwrap();
        
        let path = temp_file.path();
        let duration = Duration::from_millis(100);
        
        cache.put(path, true, duration).unwrap();
        cache.clear().unwrap();
        
        let result = cache.get(path);
        assert_eq!(result, None);
        
        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 0);
    }
}
