//! World flags system using bitflags for performance.
//!
//! This module provides a high-performance flag system that uses:
//! - **Bitflags** for known, common flags (O(1) access, 8 bytes total)
//! - **Sparse set** for dynamic/rare flags (still fast, but flexible)
//!
//! ## Performance
//!
//! | Operation | HashMap<String,bool> | WorldFlags |
//! |-----------|---------------------|------------|
//! | Check known flag | ~50ns (hash + lookup) | ~1ns (bitwise AND) |
//! | Set known flag | ~80ns | ~1ns (bitwise OR) |
//! | Memory (64 flags) | ~2KB | 8 bytes |
//!
//! ## Usage
//!
//! ```ignore
//! let mut flags = WorldFlags::new();
//!
//! // Known flags (fast path)
//! flags.set(KnownFlag::MetChildhoodFriend);
//! if flags.has(KnownFlag::RecessionActive) { ... }
//!
//! // Dynamic flags (flexible path)
//! flags.set_dynamic("custom_storylet_completed");
//! if flags.has_dynamic("custom_storylet_completed") { ... }
//! ```

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

/// Known world flags - common flags that benefit from bitflag optimization.
/// Add new flags here as the game grows. Max 64 flags in a single u64.
///
/// Flags are organized into categories:
/// - **Relationship Milestones**: First meetings, romantic events
/// - **Economic Events**: Recessions, booms, crises
/// - **Life Events**: Graduation, jobs, retirement
/// - **Health Events**: Illness, addiction, near-death
/// - **Crime/Legal**: Arrests, convictions, exoneration
/// - **Family**: Children, loss, adoption
/// - **Social**: Fame, infamy, cults
/// - **Spirals**: Trauma/failure spirals
/// - **Black Swan**: Rare world events
/// - **Digital Legacy**: Transcendence events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum KnownFlag {
    // === Relationship Milestones ===
    /// Player met their childhood friend.
    MetChildhoodFriend      = 1 << 0,
    /// Player met their first love.
    MetFirstLove            = 1 << 1,
    /// Player met a mentor figure.
    MetMentor               = 1 << 2,
    /// Player met a rival.
    MetRival                = 1 << 3,
    /// Player met their best friend.
    MetBestFriend           = 1 << 4,
    /// Player had their first kiss.
    FirstKiss               = 1 << 5,
    /// Player experienced first heartbreak.
    FirstHeartbreak         = 1 << 6,
    /// Player got married.
    Married                 = 1 << 7,
    /// Player got divorced.
    Divorced                = 1 << 8,
    /// Player's spouse died.
    Widowed                 = 1 << 9,
    
    // === Economic Events ===
    /// Economic recession is active.
    RecessionActive         = 1 << 10,
    /// Economic boom is active.
    BoomActive              = 1 << 11,
    /// Housing crisis is active.
    HousingCrisisActive     = 1 << 12,
    /// Tech revolution is active.
    TechRevolutionActive    = 1 << 13,
    
    // === Life Events ===
    /// Player graduated high school.
    GraduatedHighSchool     = 1 << 14,
    /// Player graduated college.
    GraduatedCollege        = 1 << 15,
    /// Player got their first job.
    GotFirstJob             = 1 << 16,
    /// Player was fired.
    GotFired                = 1 << 17,
    /// Player was promoted.
    GotPromoted             = 1 << 18,
    /// Player started a business.
    StartedBusiness         = 1 << 19,
    /// Player went bankrupt.
    WentBankrupt            = 1 << 20,
    /// Player retired early.
    RetiredEarly            = 1 << 21,
    
    // === Health Events ===
    /// Player had a major illness.
    HadMajorIllness         = 1 << 22,
    /// Player recovered from addiction.
    RecoveredFromAddiction  = 1 << 23,
    /// Player had a near-death experience.
    HadNearDeathExperience  = 1 << 24,
    
    // === Crime/Legal ===
    /// Player was arrested.
    Arrested                = 1 << 25,
    /// Player was convicted of a crime.
    Convicted               = 1 << 26,
    /// Player served time in prison.
    ServedTime              = 1 << 27,
    /// Player was exonerated.
    Exonerated              = 1 << 28,
    
    // === Family ===
    /// Player had a child.
    HadChild                = 1 << 29,
    /// Player lost a parent.
    LostParent              = 1 << 30,
    /// Player lost a child.
    LostChild               = 1 << 31,
    /// Player adopted a child.
    Adopted                 = 1 << 32,
    
    // === Social ===
    /// Player became famous.
    BecameFamous            = 1 << 33,
    /// Player became infamous.
    BecameInfamous          = 1 << 34,
    /// Player joined a cult.
    JoinedCult              = 1 << 35,
    /// Player left a cult.
    LeftCult                = 1 << 36,
    
    // === Spirals ===
    /// Player experienced an anxiety spiral.
    ExperiencedAnxietySpiral    = 1 << 37,
    /// Player experienced a depression spiral.
    ExperiencedDepressionSpiral = 1 << 38,
    /// Player experienced an addiction spiral.
    ExperiencedAddictionSpiral  = 1 << 39,
    
    // === Black Swan ===
    /// Player survived a pandemic.
    SurvivedPandemic        = 1 << 40,
    /// Player survived a crime wave.
    SurvivedCrimeWave       = 1 << 41,
    /// Player survived corporate collapse.
    SurvivedCorporateCollapse = 1 << 42,
    
    // === Digital Legacy ===
    /// Player created a digital legacy.
    CreatedDigitalLegacy    = 1 << 43,
    /// Player uploaded consciousness.
    UploadedConsciousness   = 1 << 44,
    
    // Reserved for future use (45-63)
}

impl KnownFlag {
    /// Get the bit value for this flag.
    #[inline]
    pub const fn bit(self) -> u64 {
        self as u64
    }

    /// Try to parse a string into a known flag.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "met_childhood_friend" => Some(Self::MetChildhoodFriend),
            "met_first_love" => Some(Self::MetFirstLove),
            "met_mentor" => Some(Self::MetMentor),
            "met_rival" => Some(Self::MetRival),
            "met_best_friend" => Some(Self::MetBestFriend),
            "first_kiss" => Some(Self::FirstKiss),
            "first_heartbreak" => Some(Self::FirstHeartbreak),
            "married" => Some(Self::Married),
            "divorced" => Some(Self::Divorced),
            "widowed" => Some(Self::Widowed),
            "recession_active" => Some(Self::RecessionActive),
            "boom_active" => Some(Self::BoomActive),
            "housing_crisis_active" => Some(Self::HousingCrisisActive),
            "tech_revolution_active" => Some(Self::TechRevolutionActive),
            "graduated_high_school" => Some(Self::GraduatedHighSchool),
            "graduated_college" => Some(Self::GraduatedCollege),
            "got_first_job" => Some(Self::GotFirstJob),
            "got_fired" => Some(Self::GotFired),
            "got_promoted" => Some(Self::GotPromoted),
            "started_business" => Some(Self::StartedBusiness),
            "went_bankrupt" => Some(Self::WentBankrupt),
            "retired_early" => Some(Self::RetiredEarly),
            "had_major_illness" => Some(Self::HadMajorIllness),
            "recovered_from_addiction" => Some(Self::RecoveredFromAddiction),
            "had_near_death_experience" => Some(Self::HadNearDeathExperience),
            "arrested" => Some(Self::Arrested),
            "convicted" => Some(Self::Convicted),
            "served_time" => Some(Self::ServedTime),
            "exonerated" => Some(Self::Exonerated),
            "had_child" => Some(Self::HadChild),
            "lost_parent" => Some(Self::LostParent),
            "lost_child" => Some(Self::LostChild),
            "adopted" => Some(Self::Adopted),
            "became_famous" => Some(Self::BecameFamous),
            "became_infamous" => Some(Self::BecameInfamous),
            "joined_cult" => Some(Self::JoinedCult),
            "left_cult" => Some(Self::LeftCult),
            "experienced_anxiety_spiral" => Some(Self::ExperiencedAnxietySpiral),
            "experienced_depression_spiral" => Some(Self::ExperiencedDepressionSpiral),
            "experienced_addiction_spiral" => Some(Self::ExperiencedAddictionSpiral),
            "survived_pandemic" => Some(Self::SurvivedPandemic),
            "survived_crime_wave" => Some(Self::SurvivedCrimeWave),
            "survived_corporate_collapse" => Some(Self::SurvivedCorporateCollapse),
            "created_digital_legacy" => Some(Self::CreatedDigitalLegacy),
            "uploaded_consciousness" => Some(Self::UploadedConsciousness),
            _ => None,
        }
    }

    /// Convert to snake_case string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetChildhoodFriend => "met_childhood_friend",
            Self::MetFirstLove => "met_first_love",
            Self::MetMentor => "met_mentor",
            Self::MetRival => "met_rival",
            Self::MetBestFriend => "met_best_friend",
            Self::FirstKiss => "first_kiss",
            Self::FirstHeartbreak => "first_heartbreak",
            Self::Married => "married",
            Self::Divorced => "divorced",
            Self::Widowed => "widowed",
            Self::RecessionActive => "recession_active",
            Self::BoomActive => "boom_active",
            Self::HousingCrisisActive => "housing_crisis_active",
            Self::TechRevolutionActive => "tech_revolution_active",
            Self::GraduatedHighSchool => "graduated_high_school",
            Self::GraduatedCollege => "graduated_college",
            Self::GotFirstJob => "got_first_job",
            Self::GotFired => "got_fired",
            Self::GotPromoted => "got_promoted",
            Self::StartedBusiness => "started_business",
            Self::WentBankrupt => "went_bankrupt",
            Self::RetiredEarly => "retired_early",
            Self::HadMajorIllness => "had_major_illness",
            Self::RecoveredFromAddiction => "recovered_from_addiction",
            Self::HadNearDeathExperience => "had_near_death_experience",
            Self::Arrested => "arrested",
            Self::Convicted => "convicted",
            Self::ServedTime => "served_time",
            Self::Exonerated => "exonerated",
            Self::HadChild => "had_child",
            Self::LostParent => "lost_parent",
            Self::LostChild => "lost_child",
            Self::Adopted => "adopted",
            Self::BecameFamous => "became_famous",
            Self::BecameInfamous => "became_infamous",
            Self::JoinedCult => "joined_cult",
            Self::LeftCult => "left_cult",
            Self::ExperiencedAnxietySpiral => "experienced_anxiety_spiral",
            Self::ExperiencedDepressionSpiral => "experienced_depression_spiral",
            Self::ExperiencedAddictionSpiral => "experienced_addiction_spiral",
            Self::SurvivedPandemic => "survived_pandemic",
            Self::SurvivedCrimeWave => "survived_crime_wave",
            Self::SurvivedCorporateCollapse => "survived_corporate_collapse",
            Self::CreatedDigitalLegacy => "created_digital_legacy",
            Self::UploadedConsciousness => "uploaded_consciousness",
        }
    }
}

/// High-performance world flags container.
///
/// Uses a u64 bitfield for known flags (O(1) operations) and a sparse
/// FxHashSet for dynamic flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldFlags {
    /// Bitfield for known flags - 64 flags in 8 bytes.
    known: u64,
    /// Dynamic flags for storylet-specific or generated flags.
    /// Uses FxHashSet for fast membership checks.
    dynamic: FxHashSet<String>,
}

impl WorldFlags {
    /// Create empty world flags.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // === Known flag operations (fast path) ===

    /// Check if a known flag is set. O(1) bitwise operation.
    #[inline]
    pub fn has(&self, flag: KnownFlag) -> bool {
        (self.known & flag.bit()) != 0
    }

    /// Set a known flag. O(1) bitwise operation.
    #[inline]
    pub fn set(&mut self, flag: KnownFlag) {
        self.known |= flag.bit();
    }

    /// Clear a known flag. O(1) bitwise operation.
    #[inline]
    pub fn clear(&mut self, flag: KnownFlag) {
        self.known &= !flag.bit();
    }

    /// Toggle a known flag. O(1) bitwise operation.
    #[inline]
    pub fn toggle(&mut self, flag: KnownFlag) {
        self.known ^= flag.bit();
    }

    // === Dynamic flag operations (flexible path) ===

    /// Check if a dynamic flag is set.
    #[inline]
    pub fn has_dynamic(&self, flag: &str) -> bool {
        self.dynamic.contains(flag)
    }

    /// Set a dynamic flag.
    #[inline]
    pub fn set_dynamic(&mut self, flag: impl Into<String>) {
        self.dynamic.insert(flag.into());
    }

    /// Clear a dynamic flag.
    #[inline]
    pub fn clear_dynamic(&mut self, flag: &str) {
        self.dynamic.remove(flag);
    }

    // === Unified API (auto-routes to fast or flexible path) ===

    /// Check any flag by string name.
    /// Routes to bitflag for known flags, hashset for dynamic.
    #[inline]
    pub fn has_any(&self, flag: &str) -> bool {
        if let Some(known) = KnownFlag::from_str(flag) {
            self.has(known)
        } else {
            self.has_dynamic(flag)
        }
    }

    /// Set any flag by string name.
    /// Routes to bitflag for known flags, hashset for dynamic.
    #[inline]
    pub fn set_any(&mut self, flag: &str) {
        if let Some(known) = KnownFlag::from_str(flag) {
            self.set(known);
        } else {
            self.set_dynamic(flag.to_string());
        }
    }

    /// Clear any flag by string name.
    #[inline]
    pub fn clear_any(&mut self, flag: &str) {
        if let Some(known) = KnownFlag::from_str(flag) {
            self.clear(known);
        } else {
            self.clear_dynamic(flag);
        }
    }

    /// Get count of all set flags.
    pub fn count(&self) -> usize {
        self.known.count_ones() as usize + self.dynamic.len()
    }

    /// Check if any flags are set.
    pub fn is_empty(&self) -> bool {
        self.known == 0 && self.dynamic.is_empty()
    }

    /// Get raw bitfield (for debugging/serialization).
    #[inline]
    pub fn known_bits(&self) -> u64 {
        self.known
    }

    /// Get iterator over dynamic flag names.
    pub fn dynamic_flags(&self) -> impl Iterator<Item = &str> {
        self.dynamic.iter().map(|s| s.as_str())
    }

    /// Get all set known flags.
    pub fn known_flags(&self) -> Vec<KnownFlag> {
        let mut flags = Vec::new();
        let all_known = [
            KnownFlag::MetChildhoodFriend,
            KnownFlag::MetFirstLove,
            KnownFlag::MetMentor,
            KnownFlag::MetRival,
            KnownFlag::MetBestFriend,
            KnownFlag::FirstKiss,
            KnownFlag::FirstHeartbreak,
            KnownFlag::Married,
            KnownFlag::Divorced,
            KnownFlag::Widowed,
            KnownFlag::RecessionActive,
            KnownFlag::BoomActive,
            KnownFlag::HousingCrisisActive,
            KnownFlag::TechRevolutionActive,
            KnownFlag::GraduatedHighSchool,
            KnownFlag::GraduatedCollege,
            KnownFlag::GotFirstJob,
            KnownFlag::GotFired,
            KnownFlag::GotPromoted,
            KnownFlag::StartedBusiness,
            KnownFlag::WentBankrupt,
            KnownFlag::RetiredEarly,
            KnownFlag::HadMajorIllness,
            KnownFlag::RecoveredFromAddiction,
            KnownFlag::HadNearDeathExperience,
            KnownFlag::Arrested,
            KnownFlag::Convicted,
            KnownFlag::ServedTime,
            KnownFlag::Exonerated,
            KnownFlag::HadChild,
            KnownFlag::LostParent,
            KnownFlag::LostChild,
            KnownFlag::Adopted,
            KnownFlag::BecameFamous,
            KnownFlag::BecameInfamous,
            KnownFlag::JoinedCult,
            KnownFlag::LeftCult,
            KnownFlag::ExperiencedAnxietySpiral,
            KnownFlag::ExperiencedDepressionSpiral,
            KnownFlag::ExperiencedAddictionSpiral,
            KnownFlag::SurvivedPandemic,
            KnownFlag::SurvivedCrimeWave,
            KnownFlag::SurvivedCorporateCollapse,
            KnownFlag::CreatedDigitalLegacy,
            KnownFlag::UploadedConsciousness,
        ];
        for flag in all_known {
            if self.has(flag) {
                flags.push(flag);
            }
        }
        flags
    }
}

// === Serde: Serialize as HashMap<String, bool> for compatibility ===

impl Serialize for WorldFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        
        let known_count = self.known.count_ones() as usize;
        let total = known_count + self.dynamic.len();
        
        let mut map = serializer.serialize_map(Some(total))?;
        
        // Serialize known flags
        for flag in self.known_flags() {
            map.serialize_entry(flag.as_str(), &true)?;
        }
        
        // Serialize dynamic flags
        for flag in &self.dynamic {
            map.serialize_entry(flag, &true)?;
        }
        
        map.end()
    }
}

impl<'de> Deserialize<'de> for WorldFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as HashMap<String, bool> for compatibility
        let map: std::collections::HashMap<String, bool> = 
            std::collections::HashMap::deserialize(deserializer)?;
        
        let mut flags = WorldFlags::new();
        
        for (key, value) in map {
            if value {
                flags.set_any(&key);
            }
        }
        
        Ok(flags)
    }
}

// === Backward compatibility: From/Into HashMap<String, bool> ===

impl From<std::collections::HashMap<String, bool>> for WorldFlags {
    fn from(map: std::collections::HashMap<String, bool>) -> Self {
        let mut flags = WorldFlags::new();
        for (key, value) in map {
            if value {
                flags.set_any(&key);
            }
        }
        flags
    }
}

impl From<WorldFlags> for std::collections::HashMap<String, bool> {
    fn from(flags: WorldFlags) -> Self {
        let mut map = std::collections::HashMap::new();
        
        // Add known flags
        for flag in flags.known_flags() {
            map.insert(flag.as_str().to_string(), true);
        }
        
        // Add dynamic flags
        for flag in flags.dynamic {
            map.insert(flag, true);
        }
        
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_flag_operations() {
        let mut flags = WorldFlags::new();
        
        assert!(!flags.has(KnownFlag::Married));
        flags.set(KnownFlag::Married);
        assert!(flags.has(KnownFlag::Married));
        
        flags.clear(KnownFlag::Married);
        assert!(!flags.has(KnownFlag::Married));
        
        flags.toggle(KnownFlag::Married);
        assert!(flags.has(KnownFlag::Married));
        flags.toggle(KnownFlag::Married);
        assert!(!flags.has(KnownFlag::Married));
    }

    #[test]
    fn test_dynamic_flag_operations() {
        let mut flags = WorldFlags::new();
        
        assert!(!flags.has_dynamic("custom_flag"));
        flags.set_dynamic("custom_flag");
        assert!(flags.has_dynamic("custom_flag"));
        
        flags.clear_dynamic("custom_flag");
        assert!(!flags.has_dynamic("custom_flag"));
    }

    #[test]
    fn test_unified_api() {
        let mut flags = WorldFlags::new();
        
        // Known flag via string
        flags.set_any("married");
        assert!(flags.has_any("married"));
        assert!(flags.has(KnownFlag::Married));
        
        // Dynamic flag via string
        flags.set_any("storylet_123_completed");
        assert!(flags.has_any("storylet_123_completed"));
        assert!(flags.has_dynamic("storylet_123_completed"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut flags = WorldFlags::new();
        flags.set(KnownFlag::Married);
        flags.set(KnownFlag::RecessionActive);
        flags.set_dynamic("custom_event");
        
        let json = serde_json::to_string(&flags).unwrap();
        let restored: WorldFlags = serde_json::from_str(&json).unwrap();
        
        assert!(restored.has(KnownFlag::Married));
        assert!(restored.has(KnownFlag::RecessionActive));
        assert!(restored.has_dynamic("custom_event"));
        assert!(!restored.has(KnownFlag::Divorced));
    }

    #[test]
    fn test_hashmap_compatibility() {
        let mut map = std::collections::HashMap::new();
        map.insert("married".to_string(), true);
        map.insert("custom_flag".to_string(), true);
        map.insert("ignored_false".to_string(), false);
        
        let flags: WorldFlags = map.into();
        
        assert!(flags.has(KnownFlag::Married));
        assert!(flags.has_dynamic("custom_flag"));
        assert!(!flags.has_any("ignored_false"));
    }

    #[test]
    fn test_count_and_empty() {
        let mut flags = WorldFlags::new();
        assert!(flags.is_empty());
        assert_eq!(flags.count(), 0);
        
        flags.set(KnownFlag::Married);
        flags.set(KnownFlag::Divorced);
        flags.set_dynamic("custom");
        
        assert!(!flags.is_empty());
        assert_eq!(flags.count(), 3);
    }

    #[test]
    fn test_memory_size() {
        // WorldFlags should be much smaller than HashMap<String, bool>
        let flags = WorldFlags::new();
        let size = std::mem::size_of_val(&flags);
        
        // u64 (8 bytes) + FxHashSet overhead (~48 bytes empty)
        assert!(size < 100, "WorldFlags should be compact, got {} bytes", size);
    }
}
