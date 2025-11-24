use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// Compact bitset for storylet tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TagBitset(pub u64);

impl TagBitset {
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn matches(&self, other: &TagBitset) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn from_tags(tags: Vec<String>) -> Self {
        Self::from_tags_slice(&tags)
    }

    pub fn from_tags_slice(tags: &[String]) -> Self {
        let mut bitset = 0u64;
        for tag in tags {
            let mut hasher = DefaultHasher::new();
            tag.hash(&mut hasher);
            let bit = (hasher.finish() % 64) as u64;
            bitset |= 1u64 << bit;
        }
        TagBitset(bitset)
    }
}

impl BitAnd for TagBitset {
    type Output = TagBitset;
    fn bitand(self, rhs: TagBitset) -> TagBitset {
        TagBitset(self.0 & rhs.0)
    }
}

impl BitAndAssign for TagBitset {
    fn bitand_assign(&mut self, rhs: TagBitset) {
        self.0 &= rhs.0;
    }
}

impl BitOr for TagBitset {
    type Output = TagBitset;
    fn bitor(self, rhs: TagBitset) -> TagBitset {
        TagBitset(self.0 | rhs.0)
    }
}

impl BitOrAssign for TagBitset {
    fn bitor_assign(&mut self, rhs: TagBitset) {
        self.0 |= rhs.0;
    }
}
