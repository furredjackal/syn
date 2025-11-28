//! Bleeding-edge stable collection types.
//!
//! This module provides high-performance collection aliases that can be
//! swapped out transparently. All game code should use these types instead
//! of `std::collections` directly.
//!
//! ## Performance Characteristics
//!
//! | Type | Use Case | Why |
//! |------|----------|-----|
//! | `FastHashMap` | Hot paths (tick loop) | FxHash is 2-3x faster than SipHash |
//! | `FastHashSet` | Membership checks | Same hash performance benefits |
//! | `SmallVec8` | Small, stack-allocated vecs | Avoids heap for ≤8 elements |
//! | `CompactString` | Short strings | Inline storage for ≤24 chars |

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

// Re-export for convenience
pub use compact_str::CompactString;
pub use rustc_hash::{FxBuildHasher, FxHasher};
pub use smallvec;

/// Fast HashMap using FxHash (Rustc's internal hasher).
/// 
/// ~2-3x faster than std HashMap for integer/small keys.
/// NOT cryptographically secure - don't use for untrusted input.
pub type FastHashMap<K, V> = FxHashMap<K, V>;

/// Fast HashSet using FxHash.
pub type FastHashSet<T> = FxHashSet<T>;

/// Small vector that stores up to 8 elements on the stack.
/// Falls back to heap allocation for larger sizes.
pub type SmallVec8<T> = SmallVec<[T; 8]>;

/// Small vector for 4 elements (relationship axes, etc.)
pub type SmallVec4<T> = SmallVec<[T; 4]>;

/// Small vector for 16 elements (NPC lists, etc.)
pub type SmallVec16<T> = SmallVec<[T; 16]>;

/// Wrapper for FastHashMap that implements Serialize/Deserialize
/// by converting to/from std HashMap.
#[derive(Debug, Clone, Default)]
pub struct SerializableFastMap<K, V>(pub FastHashMap<K, V>);

impl<K, V> SerializableFastMap<K, V> 
where 
    K: std::hash::Hash + Eq,
{
    /// Create a new empty serializable fast map.
    pub fn new() -> Self {
        Self(FastHashMap::default())
    }

    /// Create a new serializable fast map with the given capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self(FastHashMap::with_capacity_and_hasher(cap, FxBuildHasher))
    }

    /// Insert a key-value pair into the map.
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    /// Get a reference to the value for a key.
    #[inline]
    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.get(k)
    }

    /// Get a mutable reference to the value for a key.
    #[inline]
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.0.get_mut(k)
    }

    /// Check if the map contains a key.
    #[inline]
    pub fn contains_key(&self, k: &K) -> bool {
        self.0.contains_key(k)
    }

    /// Remove a key from the map, returning its value if present.
    #[inline]
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.0.remove(k)
    }

    /// Get the number of entries in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over key-value pairs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter()
    }

    /// Iterate over key-value pairs mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.0.iter_mut()
    }

    /// Iterate over keys.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys()
    }

    /// Iterate over values.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.values()
    }
}

impl<K, V> Serialize for SerializableFastMap<K, V>
where
    K: Serialize + std::hash::Hash + Eq,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de, K, V> Deserialize<'de> for SerializableFastMap<K, V>
where
    K: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let std_map: std::collections::HashMap<K, V> = std::collections::HashMap::deserialize(deserializer)?;
        let mut fast_map = FastHashMap::with_capacity_and_hasher(std_map.len(), FxBuildHasher);
        for (k, v) in std_map {
            fast_map.insert(k, v);
        }
        Ok(Self(fast_map))
    }
}

impl<K, V> std::ops::Deref for SerializableFastMap<K, V> {
    type Target = FastHashMap<K, V>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> std::ops::DerefMut for SerializableFastMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V> FromIterator<(K, V)> for SerializableFastMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_hashmap_basic() {
        let mut map: FastHashMap<u64, String> = FastHashMap::default();
        map.insert(1, "one".to_string());
        map.insert(2, "two".to_string());
        
        assert_eq!(map.get(&1), Some(&"one".to_string()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_smallvec_stack_allocation() {
        let mut vec: SmallVec8<i32> = SmallVec::new();
        for i in 0..8 {
            vec.push(i);
        }
        // Still on stack
        assert!(!vec.spilled());
        
        vec.push(8);
        // Now on heap
        assert!(vec.spilled());
    }

    #[test]
    fn test_serializable_fast_map() {
        let mut map: SerializableFastMap<String, i32> = SerializableFastMap::new();
        map.insert("foo".to_string(), 42);
        map.insert("bar".to_string(), 99);

        let json = serde_json::to_string(&map).unwrap();
        let restored: SerializableFastMap<String, i32> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.get(&"foo".to_string()), Some(&42));
        assert_eq!(restored.get(&"bar".to_string()), Some(&99));
    }

    #[test]
    fn test_compact_string() {
        let short: CompactString = "hello".into();
        let long: CompactString = "this is a much longer string that exceeds inline capacity".into();
        
        assert_eq!(short.as_str(), "hello");
        assert!(long.len() > 24); // Just verify it's longer than inline capacity
    }
}
