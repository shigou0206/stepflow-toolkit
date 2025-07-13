//! Cache system implementation

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    created_at: Instant,
    ttl: Option<Duration>,
}

impl CacheEntry {
    fn new(value: String, ttl: Option<Duration>) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

/// Simple cache implementation
pub struct Cache {
    data: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
    max_size: usize,
}

impl Cache {
    /// Create a new cache
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            default_ttl,
            max_size,
        }
    }
    
    /// Get a value from cache
    pub async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        if let Some(entry) = data.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        None
    }
    
    /// Put a value into cache
    pub async fn put(&self, key: String, value: String) {
        let mut data = self.data.write().await;
        
        // Remove expired entries
        data.retain(|_, entry| !entry.is_expired());
        
        // Remove oldest entries if at capacity
        if data.len() >= self.max_size {
            if let Some(oldest_key) = data.keys().next().cloned() {
                data.remove(&oldest_key);
            }
        }
        
        data.insert(key, CacheEntry::new(value, Some(self.default_ttl)));
    }
    
    /// Remove a value from cache
    pub async fn remove(&self, key: &str) {
        let mut data = self.data.write().await;
        data.remove(key);
    }
    
    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }
    
    /// Get cache size
    pub async fn size(&self) -> usize {
        let data = self.data.read().await;
        data.len()
    }
    
    /// Check if cache contains key
    pub async fn contains(&self, key: &str) -> bool {
        let data = self.data.read().await;
        if let Some(entry) = data.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }
} 