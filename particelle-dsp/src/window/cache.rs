use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::spec::{NormalizationMode, WindowSpec};
use super::normalize;

/// Cache key: serialized spec string + length + normalization mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    /// JSON-serialized `WindowSpec` — stable, unique per variant + params.
    spec_json: String,
    length: usize,
    norm: NormalizationMode,
}

/// Thread-safe window cache.
///
/// Windows are computed once per `(WindowSpec, length, NormalizationMode)` triple
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
        norm: NormalizationMode,
    ) -> Arc<[f64]> {
        let key = CacheKey {
            spec_json: serde_json::to_string(spec).expect("WindowSpec serialization must not fail"),
            length,
            norm,
        };

        let mut cache = self.cache.lock().expect("WindowCache lock poisoned");

        if let Some(w) = cache.get(&key) {
            return Arc::clone(w);
        }

        let mut values = compute_window(spec, length);
        normalize(&mut values, norm);
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

/// Compute a raw (unnormalized) window of `length` samples.
///
/// # Panics
/// Panics with a clear message if `length == 0`.
pub fn compute_window(spec: &WindowSpec, length: usize) -> Vec<f64> {
    assert!(length > 0, "Window length must be > 0");
    // TODO: implement all 35+ window functions in particelle-dsp Phase 2
    // Return a Hann window as placeholder for all types
    let n = length as f64 - 1.0;
    (0..length)
        .map(|i| {
            let _ = spec; // will dispatch on spec
            0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / n).cos())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_returns_same_arc_for_same_key() {
        let cache = WindowCache::new();
        let a = cache.get(&WindowSpec::Hann, 1024, NormalizationMode::Peak);
        let b = cache.get(&WindowSpec::Hann, 1024, NormalizationMode::Peak);
        assert!(Arc::ptr_eq(&a, &b), "Same key must return the same Arc");
    }

    #[test]
    fn cache_returns_different_arc_for_different_length() {
        let cache = WindowCache::new();
        let a = cache.get(&WindowSpec::Hann, 512, NormalizationMode::None);
        let b = cache.get(&WindowSpec::Hann, 1024, NormalizationMode::None);
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn window_output_length_matches_request() {
        let cache = WindowCache::new();
        for &len in &[64usize, 256, 1024, 4096] {
            let w = cache.get(&WindowSpec::Hann, len, NormalizationMode::None);
            assert_eq!(w.len(), len);
        }
    }
}
