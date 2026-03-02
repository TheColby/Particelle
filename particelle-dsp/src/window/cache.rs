use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::schema::{WindowSpec, WindowNormalization};
use super::generator::generate;
use super::normalization::apply_normalization;

/// Cache key: serialized spec string + length/// Cache key definition for identical windows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    // Note: Due to f64s in WindowSpec, deriving Eq and Hash directly is tricky.
    // For Phase 2 we use a string-serialized key to side-step float hashing complexity
    // in the map, ensuring deterministic lookups.
    key_hash: String,
}

impl CacheKey {
    fn new(spec: &WindowSpec, len: usize, norm: WindowNormalization) -> Self {
        // Since norm is now simple we can just serialize the spec and append len/norm
        let serialized_spec = serde_json::to_string(spec).unwrap();
        let key_hash = format!("{}-{}-{:?}", serialized_spec, len, norm);
        Self { key_hash }
    }
}

/// Thread-safe window cache.
///
/// Windows are computed once per `(WindowSpec, length, WindowNormalization)` triple
/// and returned as `Arc<[f64]>`. The audio thread receives a clone of the Arc,
/// which is a pointer copy — no allocation. The cache itself is never accessed
/// from the audio thread after initialization.
pub struct WindowCache {
    cache: Mutex<HashMap<CacheKey, Arc<[f64]>>>,
}

impl WindowCache {
    pub fn new() -> Self {
        Self { cache: Mutex::new(HashMap::new()) }
    }

    /// Retrieve a window, computing and caching it if not already present.
    pub fn get(
        &self,
        spec: &WindowSpec,
        length: usize,
        norm: WindowNormalization,
    ) -> Arc<[f64]> {
        let key = CacheKey::new(spec, length, norm);

        let mut cache = self.cache.lock().expect("WindowCache lock poisoned");

        if let Some(w) = cache.get(&key) {
            return Arc::clone(w);
        }

        let mut values = generate(spec, length);
        apply_normalization(&mut values, norm);
        let arc: Arc<[f64]> = values.into();
        cache.insert(key, Arc::clone(&arc));
        arc
    }

    /// Number of cached windows.
    pub fn len(&self) -> usize {
        self.cache.lock().expect("WindowCache lock poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for WindowCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_returns_same_arc_for_same_key() {
        let cache = WindowCache::new();
        let a = cache.get(&WindowSpec::Hann, 1024, WindowNormalization::Peak);
        let b = cache.get(&WindowSpec::Hann, 1024, WindowNormalization::Peak);
        assert!(Arc::ptr_eq(&a, &b), "Same key must return the same Arc");
    }

    #[test]
    fn cache_returns_different_arc_for_different_length() {
        let cache = WindowCache::new();
        let a = cache.get(&WindowSpec::Hann, 512, WindowNormalization::None);
        let b = cache.get(&WindowSpec::Hann, 1024, WindowNormalization::None);
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn window_output_length_matches_request() {
        let cache = WindowCache::new();
        for &len in &[64usize, 256, 1024, 4096] {
            let w = cache.get(&WindowSpec::Hann, len, WindowNormalization::None);
            assert_eq!(w.len(), len);
        }
    }
}
