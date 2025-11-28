//! String interning for identifiers - reduces memory and enables O(1) comparisons.
//!
//! This module provides a global string interner for frequently-used identifiers like:
//! - NPC IDs and names
//! - Storylet IDs and tags  
//! - Trait names
//! - Skill names
//! - Relationship types
//! - District names
//!
//! # Benefits
//!
//! - **Memory reduction**: Each unique string stored only once
//! - **O(1) comparisons**: Compare interned strings via pointer equality
//! - **Cache-friendly**: Interned strings have stable addresses
//! - **Thread-safe**: Uses `ThreadedRodeo` for concurrent access
//!
//! # Usage
//!
//! ```ignore
//! use syn_core::intern::{intern, resolve, InternedStr};
//!
//! // Intern a string (returns a cheap Copy handle)
//! let id: InternedStr = intern("npc_alice");
//!
//! // Resolve back to &str
//! let s: &str = resolve(id);
//! assert_eq!(s, "npc_alice");
//!
//! // Fast comparison - just compares the u32 key
//! let id2 = intern("npc_alice");
//! assert_eq!(id, id2);  // Same string = same key
//! ```
//!
//! # Serialization
//!
//! `InternedStr` serializes as a regular string for JSON compatibility.
//! On deserialization, strings are automatically re-interned.

use lasso::{Spur, ThreadedRodeo};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

/// Global string interner instance.
/// 
/// Uses `ThreadedRodeo` for lock-free reads and concurrent writes.
static INTERNER: OnceLock<ThreadedRodeo> = OnceLock::new();

/// Gets or initializes the global interner.
fn interner() -> &'static ThreadedRodeo {
    INTERNER.get_or_init(ThreadedRodeo::default)
}

/// An interned string handle.
///
/// This is a cheap `Copy` type (just a `u32` internally) that represents
/// an interned string. Use [`resolve`] to get the string value back.
///
/// # Performance
///
/// - `Copy`, `Clone`: Free (it's just a u32)
/// - `Eq`, `Hash`: O(1) - compares/hashes the key, not the string
/// - `resolve()`: O(1) - direct array lookup
/// - `intern()`: O(1) amortized for existing strings, O(n) for new strings
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternedStr(Spur);

impl InternedStr {
    /// Creates an interned string from a string slice.
    ///
    /// If the string was previously interned, returns the existing handle.
    /// Otherwise, stores the string and returns a new handle.
    #[inline]
    pub fn new(s: &str) -> Self {
        Self(interner().get_or_intern(s))
    }

    /// Creates an interned string from a static string slice.
    ///
    /// This is slightly more efficient for string literals.
    #[inline]
    pub fn from_static(s: &'static str) -> Self {
        Self(interner().get_or_intern_static(s))
    }

    /// Resolves this handle back to its string value.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        // SAFETY: ThreadedRodeo guarantees strings live for 'static
        interner().resolve(&self.0)
    }

    /// Returns the raw key value (useful for debugging).
    #[inline]
    pub fn key(&self) -> u32 {
        self.0.into_inner().get()
    }

    /// Checks if a string is already interned without interning it.
    #[inline]
    pub fn get(s: &str) -> Option<Self> {
        interner().get(s).map(Self)
    }
}

impl Hash for InternedStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash just the key, not the string - O(1)
        self.0.hash(state);
    }
}

impl fmt::Debug for InternedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InternedStr({:?})", self.as_str())
    }
}

impl fmt::Display for InternedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for InternedStr {
    fn default() -> Self {
        Self::from_static("")
    }
}

impl From<&str> for InternedStr {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for InternedStr {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&String> for InternedStr {
    #[inline]
    fn from(s: &String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for InternedStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for InternedStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for InternedStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for InternedStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

// Serde integration - serialize as string, re-intern on deserialize
impl Serialize for InternedStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for InternedStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(&s))
    }
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Interns a string and returns a handle.
///
/// This is a convenience function for `InternedStr::new()`.
///
/// # Example
///
/// ```ignore
/// let id = intern("npc_alice");
/// assert_eq!(resolve(id), "npc_alice");
/// ```
#[inline]
pub fn intern(s: &str) -> InternedStr {
    InternedStr::new(s)
}

/// Interns a static string literal.
///
/// Slightly more efficient than `intern()` for compile-time strings.
#[inline]
pub fn intern_static(s: &'static str) -> InternedStr {
    InternedStr::from_static(s)
}

/// Resolves an interned string handle to its value.
#[inline]
pub fn resolve(id: InternedStr) -> &'static str {
    id.as_str()
}

/// Returns interner statistics for debugging.
pub fn interner_stats() -> InternerStats {
    let interner = interner();
    InternerStats {
        num_strings: interner.len(),
        // Note: lasso doesn't expose memory stats directly
    }
}

/// Statistics about the string interner.
#[derive(Debug, Clone, Copy)]
pub struct InternerStats {
    /// Number of unique strings interned.
    pub num_strings: usize,
}

// ============================================================================
// Typed interned string wrappers
// ============================================================================

/// Macro to create a typed wrapper around InternedStr.
///
/// This provides type safety - you can't accidentally compare an NpcId with a StoryletId.
macro_rules! define_interned_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name(pub InternedStr);

        impl $name {
            /// Creates a new interned ID from a string.
            #[inline]
            pub fn new(s: &str) -> Self {
                Self(InternedStr::new(s))
            }

            /// Creates a new interned ID from a static string.
            #[inline]
            pub fn from_static(s: &'static str) -> Self {
                Self(InternedStr::from_static(s))
            }

            /// Returns the string value.
            #[inline]
            pub fn as_str(&self) -> &'static str {
                self.0.as_str()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0.as_str())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.0.as_str())
            }
        }

        impl From<&str> for $name {
            #[inline]
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }

        impl From<String> for $name {
            #[inline]
            fn from(s: String) -> Self {
                Self::new(&s)
            }
        }

        impl From<&String> for $name {
            #[inline]
            fn from(s: &String) -> Self {
                Self::new(s)
            }
        }

        impl AsRef<str> for $name {
            #[inline]
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Self(InternedStr::deserialize(deserializer)?))
            }
        }
    };
}

// Define typed IDs for different domains
define_interned_id!(
    /// An interned NPC identifier.
    ///
    /// Example: `NpcId::new("alice_chen")`
    NpcName
);

define_interned_id!(
    /// An interned storylet identifier.
    ///
    /// Example: `StoryletId::new("morning_coffee")`
    StoryletTag
);

define_interned_id!(
    /// An interned trait name.
    ///
    /// Example: `TraitName::new("ambitious")`
    TraitName
);

define_interned_id!(
    /// An interned skill name.
    ///
    /// Example: `SkillName::new("cooking")`
    SkillName
);

define_interned_id!(
    /// An interned district name.
    ///
    /// Example: `DistrictName::new("downtown")`
    DistrictName
);

define_interned_id!(
    /// An interned location name.
    ///
    /// Example: `LocationName::new("coffee_shop")`
    LocationName
);

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_interning() {
        let id1 = intern("hello");
        let id2 = intern("hello");
        let id3 = intern("world");

        // Same string = same key
        assert_eq!(id1, id2);
        assert_eq!(id1.key(), id2.key());

        // Different string = different key
        assert_ne!(id1, id3);
        assert_ne!(id1.key(), id3.key());

        // Resolve back to string
        assert_eq!(resolve(id1), "hello");
        assert_eq!(resolve(id3), "world");
    }

    #[test]
    fn test_interned_str_traits() {
        let id = intern("test");

        // Display
        assert_eq!(format!("{}", id), "test");

        // Debug
        assert!(format!("{:?}", id).contains("test"));

        // AsRef
        let s: &str = id.as_ref();
        assert_eq!(s, "test");

        // PartialEq with str
        assert!(id == "test");
        assert!(id == String::from("test"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let id = intern("serialization_test");
        
        // Serialize to JSON
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"serialization_test\"");

        // Deserialize back
        let id2: InternedStr = serde_json::from_str(&json).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_typed_ids() {
        let npc = NpcName::new("alice");
        let skill = SkillName::new("cooking");

        // Different types can't be compared (compile error if uncommented)
        // assert_ne!(npc, skill);

        // But they can have the same underlying string
        let npc2 = NpcName::new("alice");
        assert_eq!(npc, npc2);

        // Serialize/deserialize
        let json = serde_json::to_string(&npc).unwrap();
        assert_eq!(json, "\"alice\"");
    }

    #[test]
    fn test_hash_performance() {
        use std::collections::HashMap;

        // Interned strings as keys are fast to hash
        let mut map: HashMap<InternedStr, i32> = HashMap::new();
        
        let key1 = intern("key1");
        let key2 = intern("key2");
        
        map.insert(key1, 100);
        map.insert(key2, 200);

        assert_eq!(map.get(&key1), Some(&100));
        assert_eq!(map.get(&key2), Some(&200));

        // Looking up with re-interned key works
        let key1_again = intern("key1");
        assert_eq!(map.get(&key1_again), Some(&100));
    }

    #[test]
    fn test_from_conversions() {
        let s = "from_str";
        let owned = String::from("from_string");

        let id1: InternedStr = s.into();
        let id2: InternedStr = owned.clone().into();
        let id3: InternedStr = (&owned).into();

        assert_eq!(id1.as_str(), "from_str");
        assert_eq!(id2.as_str(), "from_string");
        assert_eq!(id3.as_str(), "from_string");
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_interner_stats() {
        // Intern some strings
        intern("stats_test_1");
        intern("stats_test_2");
        intern("stats_test_3");

        let stats = interner_stats();
        assert!(stats.num_strings >= 3);
    }

    #[test]
    fn test_default() {
        let id: InternedStr = InternedStr::default();
        assert_eq!(id.as_str(), "");

        let npc: NpcName = NpcName::default();
        assert_eq!(npc.as_str(), "");
    }
}
