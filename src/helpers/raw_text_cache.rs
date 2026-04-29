//! Raw text caching for AST nodes
//!
//! Ported from TypeSpec compiler/src/core/helpers/raw-text-cache.ts
//!
//! In the TS version, this caches the raw text of AST nodes using a WeakMap.
//! In Rust, we use a HashMap since we don't have weak references in the same way.

use std::collections::HashMap;

/// Cache for raw text of AST nodes
pub struct RawTextCache {
    cache: HashMap<u32, String>,
}

impl RawTextCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Cache the raw text for a node
    pub fn cache_raw_text(&mut self, node_id: u32, raw_text: String) {
        self.cache.insert(node_id, raw_text);
    }

    /// Get the cached raw text for a node
    pub fn get_cached_raw_text(&self, node_id: u32) -> Option<&str> {
        self.cache.get(&node_id).map(|s| s.as_str())
    }

    /// Get the raw text for a node, using cache if available.
    /// If not cached, computes it from the source text via the provided fallback
    /// function and caches the result.
    /// Ported from TS getRawTextWithCache().
    pub fn get_raw_text_with_cache<F>(&mut self, node_id: u32, compute: F) -> &str
    where
        F: FnOnce(u32) -> String,
    {
        self.cache
            .entry(node_id)
            .or_insert_with(|| compute(node_id))
            .as_str()
    }
}

impl Default for RawTextCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_and_retrieve() {
        let mut cache = RawTextCache::new();
        cache.cache_raw_text(1, "hello".to_string());
        assert_eq!(cache.get_cached_raw_text(1), Some("hello"));
    }

    #[test]
    fn test_cache_miss() {
        let cache = RawTextCache::new();
        assert_eq!(cache.get_cached_raw_text(999), None);
    }

    #[test]
    fn test_cache_overwrite() {
        let mut cache = RawTextCache::new();
        cache.cache_raw_text(1, "old".to_string());
        cache.cache_raw_text(1, "new".to_string());
        assert_eq!(cache.get_cached_raw_text(1), Some("new"));
    }

    #[test]
    fn test_get_raw_text_with_cache_miss() {
        let mut cache = RawTextCache::new();
        let result = cache.get_raw_text_with_cache(1, |_| "computed".to_string());
        assert_eq!(result, "computed");
        // Should be cached now
        assert_eq!(cache.get_cached_raw_text(1), Some("computed"));
    }

    #[test]
    fn test_get_raw_text_with_cache_hit() {
        let mut cache = RawTextCache::new();
        cache.cache_raw_text(1, "cached".to_string());
        let result = cache.get_raw_text_with_cache(1, |_| "should not compute".to_string());
        assert_eq!(result, "cached");
    }
}
