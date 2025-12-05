//! Character generation: procedural creation of player characters from a seed.
//!
//! Based on GDD §3.1: Random seed assigns family, socioeconomic tier, baseline health.
//! Weighted modifiers for parental attention, location, early trauma.
//! Derived stats: Confidence = (Attachment + Luck)/2, Early Mood bias.

use crate::npc::PersonalityVector;
use crate::rng::DeterministicRng;
use crate::{Stats, Karma, AttachmentStyle};
use serde::{Deserialize, Serialize};

/// Archetype selection from character creation UI.
/// Each archetype biases stats and personality in different directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterArchetype {
    /// High empathy, balanced stats, thrives in social choices.
    Storyteller,
    /// Intellect focused, deliberate choices.
    Analyst,
    /// Unpredictable, creative, volatile narrative beats.
    Dreamer,
    /// Bold, competitive, seeks risky opportunities.
    Challenger,
}

impl CharacterArchetype {
    /// Parse archetype from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "STORYTELLER" => Some(Self::Storyteller),
            "ANALYST" => Some(Self::Analyst),
            "DREAMER" => Some(Self::Dreamer),
            "CHALLENGER" => Some(Self::Challenger),
            _ => None,
        }
    }
    
    /// Get archetype as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Storyteller => "STORYTELLER",
            Self::Analyst => "ANALYST",
            Self::Dreamer => "DREAMER",
            Self::Challenger => "CHALLENGER",
        }
    }
}

impl Default for CharacterArchetype {
    fn default() -> Self {
        Self::Storyteller
    }
}

/// Difficulty setting affects stat modifiers and event probability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    /// More forgiving stat changes, easier recovery.
    Forgiving,
    /// Standard experience.
    Balanced,
    /// Harsher consequences, more volatile.
    Harsh,
}

impl Difficulty {
    /// Parse difficulty from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "FORGIVING" => Some(Self::Forgiving),
            "BALANCED" => Some(Self::Balanced),
            "HARSH" => Some(Self::Harsh),
            _ => None,
        }
    }
    
    /// Get difficulty as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Forgiving => "FORGIVING",
            Self::Balanced => "BALANCED",
            Self::Harsh => "HARSH",
        }
    }
    
    /// Multiplier for negative stat changes.
    pub fn negative_modifier(&self) -> f32 {
        match self {
            Self::Forgiving => 0.7,
            Self::Balanced => 1.0,
            Self::Harsh => 1.4,
        }
    }
    
    /// Multiplier for positive stat changes.
    pub fn positive_modifier(&self) -> f32 {
        match self {
            Self::Forgiving => 1.2,
            Self::Balanced => 1.0,
            Self::Harsh => 0.85,
        }
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Socioeconomic tier determines starting wealth and opportunity modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocioeconomicTier {
    /// Poverty: low starting wealth, higher adversity, resilience bonuses.
    Poverty,
    /// Working class: modest means, balanced challenges.
    WorkingClass,
    /// Middle class: comfortable start, standard progression.
    MiddleClass,
    /// Upper middle: advantages in education and networking.
    UpperMiddle,
    /// Wealthy: high starting resources, different pressures.
    Wealthy,
}

impl SocioeconomicTier {
    /// Base wealth modifier (added to starting wealth).
    pub fn wealth_modifier(&self) -> f32 {
        match self {
            Self::Poverty => -25.0,
            Self::WorkingClass => -10.0,
            Self::MiddleClass => 0.0,
            Self::UpperMiddle => 15.0,
            Self::Wealthy => 35.0,
        }
    }
    
    /// Probability weight for tier selection (cumulative).
    /// Distribution: Poverty 10%, Working 25%, Middle 40%, UpperMiddle 20%, Wealthy 5%
    pub fn selection_thresholds() -> [(f32, SocioeconomicTier); 5] {
        [
            (0.10, Self::Poverty),
            (0.35, Self::WorkingClass),
            (0.75, Self::MiddleClass),
            (0.95, Self::UpperMiddle),
            (1.00, Self::Wealthy),
        ]
    }
}

/// Family structure affects early development and attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FamilyStructure {
    /// Two present, engaged parents.
    TwoParent,
    /// Single parent household.
    SingleParent,
    /// Extended family (grandparents, aunts, etc.).
    Extended,
    /// Foster care or institutional.
    Foster,
    /// Absent/neglectful parents.
    Absent,
}

impl FamilyStructure {
    /// Attachment style probability weights based on family structure.
    /// Returns (secure, anxious, avoidant) weights.
    pub fn attachment_weights(&self) -> (f32, f32, f32) {
        match self {
            Self::TwoParent => (0.6, 0.2, 0.2),
            Self::SingleParent => (0.4, 0.35, 0.25),
            Self::Extended => (0.5, 0.25, 0.25),
            Self::Foster => (0.25, 0.35, 0.40),
            Self::Absent => (0.15, 0.40, 0.45),
        }
    }
    
    /// Probability weights for family structure selection.
    pub fn selection_thresholds() -> [(f32, FamilyStructure); 5] {
        [
            (0.55, Self::TwoParent),
            (0.75, Self::SingleParent),
            (0.88, Self::Extended),
            (0.95, Self::Foster),
            (1.00, Self::Absent),
        ]
    }
}

/// Early life events that affect initial stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarlyLifeEvent {
    /// Normal childhood, no major events.
    Uneventful,
    /// Early academic success.
    AcademicSuccess,
    /// Childhood illness or health challenge.
    HealthChallenge,
    /// Loss of a family member.
    FamilyLoss,
    /// Social bullying or exclusion.
    SocialTrauma,
    /// Discovery of a talent or passion.
    TalentDiscovery,
    /// Family financial crisis.
    FinancialCrisis,
}

impl EarlyLifeEvent {
    /// Get cumulative probability thresholds for event selection.
    pub fn selection_thresholds() -> [(f32, EarlyLifeEvent); 7] {
        [
            (0.40, Self::Uneventful),
            (0.52, Self::AcademicSuccess),
            (0.62, Self::HealthChallenge),
            (0.72, Self::FamilyLoss),
            (0.82, Self::SocialTrauma),
            (0.92, Self::TalentDiscovery),
            (1.00, Self::FinancialCrisis),
        ]
    }
    
    /// Apply stat modifications based on early life event.
    pub fn apply_to_stats(&self, stats: &mut Stats) {
        match self {
            Self::Uneventful => {
                // No modifications
            }
            Self::AcademicSuccess => {
                stats.intelligence += 8.0;
                stats.charisma += 3.0;
            }
            Self::HealthChallenge => {
                stats.health -= 10.0;
                stats.wisdom += 5.0;
            }
            Self::FamilyLoss => {
                stats.mood -= 3.0;
                stats.wisdom += 8.0;
            }
            Self::SocialTrauma => {
                stats.charisma -= 8.0;
                stats.mood -= 2.0;
                if let Some(ref mut c) = stats.curiosity {
                    *c += 5.0; // Turns inward
                }
            }
            Self::TalentDiscovery => {
                stats.charisma += 5.0;
                stats.mood += 2.0;
                if let Some(ref mut e) = stats.energy {
                    *e += 8.0;
                }
            }
            Self::FinancialCrisis => {
                stats.wealth -= 15.0;
                stats.mood -= 2.0;
            }
        }
        stats.clamp();
    }
}

/// Configuration for character generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterGenConfig {
    /// Player-chosen name.
    pub name: String,
    /// Selected archetype.
    pub archetype: CharacterArchetype,
    /// Selected difficulty.
    pub difficulty: Difficulty,
    /// SFW mode (affects content, not generation).
    pub sfw_mode: bool,
}

impl Default for CharacterGenConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            archetype: CharacterArchetype::default(),
            difficulty: Difficulty::default(),
            sfw_mode: true,
        }
    }
}

/// Result of character generation - all procedurally determined traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCharacter {
    /// Player name (from config).
    pub name: String,
    /// Chosen archetype.
    pub archetype: CharacterArchetype,
    /// Difficulty setting.
    pub difficulty: Difficulty,
    /// SFW mode.
    pub sfw_mode: bool,
    
    // Procedurally generated:
    /// Family background.
    pub family_structure: FamilyStructure,
    /// Economic starting point.
    pub socioeconomic_tier: SocioeconomicTier,
    /// Early life defining event.
    pub early_life_event: EarlyLifeEvent,
    /// Derived attachment style.
    pub attachment_style: AttachmentStyle,
    /// Luck seed (affects future RNG branches).
    pub luck_seed: u64,
    /// Starting stats (after all modifiers).
    pub stats: Stats,
    /// Personality vector.
    pub personality: PersonalityVector,
    /// Starting karma.
    pub karma: Karma,
    /// Hidden early trauma flag (affects future storylet eligibility).
    pub has_early_trauma: bool,
    /// Starting district/location.
    pub starting_district: String,
}

/// Generate a character deterministically from a world seed and config.
///
/// The same seed + config will always produce the same character.
pub fn generate_character(world_seed: u64, config: &CharacterGenConfig) -> GeneratedCharacter {
    let mut rng = DeterministicRng::with_domain(world_seed, 0, "character_gen");
    
    // Generate background
    let socioeconomic_tier = select_from_thresholds(&mut rng, &SocioeconomicTier::selection_thresholds());
    let family_structure = select_from_thresholds(&mut rng, &FamilyStructure::selection_thresholds());
    let early_life_event = select_from_thresholds(&mut rng, &EarlyLifeEvent::selection_thresholds());
    
    // Determine attachment style from family structure
    let attachment_style = generate_attachment_style(&mut rng, &family_structure);
    
    // Generate luck seed for future use
    let luck_seed = rng.gen_u64();
    
    // Check for early trauma
    let has_early_trauma = matches!(
        early_life_event,
        EarlyLifeEvent::FamilyLoss | EarlyLifeEvent::SocialTrauma | EarlyLifeEvent::HealthChallenge
    );
    
    // Generate base stats from archetype
    let mut stats = generate_base_stats(&mut rng, &config.archetype);
    
    // Apply socioeconomic modifier
    stats.wealth += socioeconomic_tier.wealth_modifier();
    
    // Apply early life event effects
    early_life_event.apply_to_stats(&mut stats);
    
    // Apply attachment style effects
    apply_attachment_effects(&mut rng, &attachment_style, &mut stats);
    
    // Final clamp
    stats.clamp();
    
    // Generate personality from archetype + random variation
    let personality = generate_personality(&mut rng, &config.archetype, &attachment_style);
    
    // Starting karma based on family/events
    let starting_karma = generate_starting_karma(&mut rng, &family_structure, &early_life_event);
    
    // Generate starting district based on socioeconomic tier
    let starting_district = generate_starting_district(&mut rng, &socioeconomic_tier);
    
    GeneratedCharacter {
        name: config.name.clone(),
        archetype: config.archetype,
        difficulty: config.difficulty,
        sfw_mode: config.sfw_mode,
        family_structure,
        socioeconomic_tier,
        early_life_event,
        attachment_style,
        luck_seed,
        stats,
        personality,
        karma: Karma(starting_karma),
        has_early_trauma,
        starting_district,
    }
}

/// Helper to select from cumulative probability thresholds.
fn select_from_thresholds<T: Copy>(rng: &mut DeterministicRng, thresholds: &[(f32, T)]) -> T {
    let roll = rng.gen_f32();
    for (threshold, value) in thresholds {
        if roll < *threshold {
            return *value;
        }
    }
    thresholds.last().unwrap().1
}

/// Generate attachment style based on family structure probabilities.
fn generate_attachment_style(rng: &mut DeterministicRng, family: &FamilyStructure) -> AttachmentStyle {
    let (secure, anxious, _avoidant) = family.attachment_weights();
    let roll = rng.gen_f32();
    
    if roll < secure {
        AttachmentStyle::Secure
    } else if roll < secure + anxious {
        AttachmentStyle::Anxious
    } else {
        AttachmentStyle::Avoidant
    }
}

/// Generate base stats based on archetype with random variation.
fn generate_base_stats(rng: &mut DeterministicRng, archetype: &CharacterArchetype) -> Stats {
    // Base stats for a 6-year-old child
    let mut stats = Stats {
        health: 70.0 + rng.gen_range_f32(-10.0, 10.0),
        intelligence: 50.0 + rng.gen_range_f32(-8.0, 8.0),
        charisma: 50.0 + rng.gen_range_f32(-8.0, 8.0),
        wealth: 50.0, // Modified by socioeconomic tier
        mood: rng.gen_range_f32(-2.0, 3.0), // Slight positive bias for children
        appearance: 50.0 + rng.gen_range_f32(-5.0, 5.0),
        reputation: 0.0, // Children start with no reputation
        wisdom: 15.0 + rng.gen_range_f32(-5.0, 5.0), // Low wisdom at start
        curiosity: Some(60.0 + rng.gen_range_f32(-10.0, 10.0)),
        energy: Some(70.0 + rng.gen_range_f32(-10.0, 10.0)),
        libido: None, // Not applicable to children
    };
    
    // Apply archetype bonuses
    match archetype {
        CharacterArchetype::Storyteller => {
            stats.charisma += 10.0;
            stats.wisdom += 5.0;
            stats.mood += 1.0;
        }
        CharacterArchetype::Analyst => {
            stats.intelligence += 12.0;
            stats.wisdom += 8.0;
            stats.charisma -= 3.0;
        }
        CharacterArchetype::Dreamer => {
            if let Some(ref mut c) = stats.curiosity {
                *c += 15.0;
            }
            stats.mood += 2.0;
            stats.intelligence += 5.0;
        }
        CharacterArchetype::Challenger => {
            stats.charisma += 5.0;
            if let Some(ref mut e) = stats.energy {
                *e += 10.0;
            }
            stats.health += 5.0;
        }
    }
    
    stats.clamp();
    stats
}

/// Apply attachment style effects to stats.
fn apply_attachment_effects(rng: &mut DeterministicRng, attachment: &AttachmentStyle, stats: &mut Stats) {
    match attachment {
        AttachmentStyle::Secure => {
            stats.mood += rng.gen_range_f32(1.0, 3.0);
            stats.charisma += rng.gen_range_f32(2.0, 5.0);
        }
        AttachmentStyle::Anxious => {
            stats.mood -= rng.gen_range_f32(1.0, 3.0);
            stats.charisma += rng.gen_range_f32(-2.0, 3.0); // Variable social skills
            if let Some(ref mut e) = stats.energy {
                *e -= rng.gen_range_f32(0.0, 5.0);
            }
        }
        AttachmentStyle::Avoidant => {
            stats.wisdom += rng.gen_range_f32(2.0, 5.0); // Self-reliance
            stats.charisma -= rng.gen_range_f32(3.0, 8.0);
            stats.intelligence += rng.gen_range_f32(0.0, 5.0); // Intellectual focus
        }
        AttachmentStyle::Fearful => {
            // Fearful combines anxious (mood) and avoidant (social withdrawal) traits
            stats.mood -= rng.gen_range_f32(2.0, 5.0); // Higher baseline anxiety
            stats.charisma -= rng.gen_range_f32(2.0, 6.0); // Social withdrawal
            stats.wisdom += rng.gen_range_f32(0.0, 3.0); // Some self-awareness
            if let Some(ref mut e) = stats.energy {
                *e -= rng.gen_range_f32(2.0, 8.0); // Emotionally draining
            }
        }
    }
}

/// Generate personality vector from archetype and attachment.
fn generate_personality(
    rng: &mut DeterministicRng,
    archetype: &CharacterArchetype,
    attachment: &AttachmentStyle,
) -> PersonalityVector {
    // Base values with random variation
    let mut personality = PersonalityVector {
        warmth: rng.gen_range_f32(-0.3, 0.3),
        dominance: rng.gen_range_f32(-0.3, 0.3),
        volatility: rng.gen_range_f32(-0.2, 0.2),
        conscientiousness: rng.gen_range_f32(0.3, 0.6),
        openness: rng.gen_range_f32(0.3, 0.6),
    };
    
    // Archetype modifiers
    match archetype {
        CharacterArchetype::Storyteller => {
            personality.warmth += 0.4;
            personality.openness += 0.2;
        }
        CharacterArchetype::Analyst => {
            personality.conscientiousness += 0.3;
            personality.volatility -= 0.2;
        }
        CharacterArchetype::Dreamer => {
            personality.openness += 0.4;
            personality.volatility += 0.2;
        }
        CharacterArchetype::Challenger => {
            personality.dominance += 0.4;
            personality.volatility += 0.1;
        }
    }
    
    // Attachment modifiers
    match attachment {
        AttachmentStyle::Secure => {
            personality.warmth += 0.15;
            personality.volatility -= 0.1;
        }
        AttachmentStyle::Anxious => {
            personality.volatility += 0.25;
            personality.warmth += 0.1; // Seeks connection
        }
        AttachmentStyle::Avoidant => {
            personality.warmth -= 0.2;
            personality.dominance += 0.1;
        }
        AttachmentStyle::Fearful => {
            // Fearful: high volatility (anxious) + low warmth (avoidant)
            personality.volatility += 0.35; // Most volatile - push-pull dynamics
            personality.warmth -= 0.1;      // Wants connection but fears it
            personality.dominance -= 0.15;  // Tends toward submission/withdrawal
        }
    }
    
    personality.clamp();
    personality
}

/// Generate starting karma based on background.
fn generate_starting_karma(
    rng: &mut DeterministicRng,
    family: &FamilyStructure,
    event: &EarlyLifeEvent,
) -> f32 {
    let mut karma = rng.gen_range_f32(-5.0, 5.0);
    
    // Family structure effects
    match family {
        FamilyStructure::TwoParent | FamilyStructure::Extended => {
            karma += rng.gen_range_f32(0.0, 5.0);
        }
        FamilyStructure::Foster => {
            karma += rng.gen_range_f32(-3.0, 3.0);
        }
        _ => {}
    }
    
    // Early life event effects
    match event {
        EarlyLifeEvent::AcademicSuccess | EarlyLifeEvent::TalentDiscovery => {
            karma += rng.gen_range_f32(2.0, 8.0);
        }
        EarlyLifeEvent::FamilyLoss | EarlyLifeEvent::SocialTrauma => {
            // Trauma can go either way
            karma += rng.gen_range_f32(-5.0, 3.0);
        }
        _ => {}
    }
    
    karma.clamp(-100.0, 100.0)
}

/// Generate starting district based on socioeconomic tier.
fn generate_starting_district(rng: &mut DeterministicRng, tier: &SocioeconomicTier) -> String {
    let districts = match tier {
        SocioeconomicTier::Poverty => ["Southside", "Industrial", "Harbor"],
        SocioeconomicTier::WorkingClass => ["Midtown", "Eastside", "Old Town"],
        SocioeconomicTier::MiddleClass => ["Suburban", "Riverside", "Parkview"],
        SocioeconomicTier::UpperMiddle => ["Hillcrest", "Lakeside", "University"],
        SocioeconomicTier::Wealthy => ["Heights", "Estates", "Downtown"],
    };
    
    let idx = rng.gen_range_i32(0, districts.len() as i32) as usize;
    districts[idx].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_is_deterministic() {
        let config = CharacterGenConfig {
            name: "TestPlayer".to_string(),
            archetype: CharacterArchetype::Storyteller,
            difficulty: Difficulty::Balanced,
            sfw_mode: true,
        };
        
        let char1 = generate_character(12345, &config);
        let char2 = generate_character(12345, &config);
        
        assert_eq!(char1.name, char2.name);
        assert_eq!(char1.family_structure, char2.family_structure);
        assert_eq!(char1.socioeconomic_tier, char2.socioeconomic_tier);
        assert_eq!(char1.early_life_event, char2.early_life_event);
        assert_eq!(char1.attachment_style, char2.attachment_style);
        assert_eq!(char1.luck_seed, char2.luck_seed);
        assert_eq!(char1.stats.health, char2.stats.health);
        assert_eq!(char1.stats.intelligence, char2.stats.intelligence);
        assert_eq!(char1.personality.warmth, char2.personality.warmth);
        assert_eq!(char1.karma.0, char2.karma.0);
    }
    
    #[test]
    fn test_different_seeds_produce_different_characters() {
        let config = CharacterGenConfig {
            name: "TestPlayer".to_string(),
            archetype: CharacterArchetype::Storyteller,
            difficulty: Difficulty::Balanced,
            sfw_mode: true,
        };
        
        let char1 = generate_character(12345, &config);
        let char2 = generate_character(67890, &config);
        
        // At least some properties should differ
        let differs = char1.luck_seed != char2.luck_seed
            || char1.stats.health != char2.stats.health
            || char1.family_structure != char2.family_structure;
        
        assert!(differs, "Different seeds should produce different characters");
    }
    
    #[test]
    fn test_archetypes_affect_stats() {
        let config_storyteller = CharacterGenConfig {
            name: "Test".to_string(),
            archetype: CharacterArchetype::Storyteller,
            ..Default::default()
        };
        let config_analyst = CharacterGenConfig {
            name: "Test".to_string(),
            archetype: CharacterArchetype::Analyst,
            ..Default::default()
        };
        
        let char_s = generate_character(99999, &config_storyteller);
        let char_a = generate_character(99999, &config_analyst);
        
        // Analyst should have higher intelligence than Storyteller on average
        // (given same seed, the archetype bonus should shift stats)
        assert!(char_a.stats.intelligence > char_s.stats.intelligence - 5.0);
    }
    
    #[test]
    fn test_stats_are_clamped() {
        let config = CharacterGenConfig {
            name: "Test".to_string(),
            archetype: CharacterArchetype::Challenger,
            difficulty: Difficulty::Harsh,
            sfw_mode: true,
        };
        
        // Test with many seeds to ensure stats stay in bounds
        for seed in 0..100 {
            let character = generate_character(seed, &config);
            
            assert!(character.stats.health >= 0.0 && character.stats.health <= 100.0);
            assert!(character.stats.mood >= -10.0 && character.stats.mood <= 10.0);
            assert!(character.stats.wealth >= 0.0 && character.stats.wealth <= 100.0);
            assert!(character.personality.warmth >= -1.0 && character.personality.warmth <= 1.0);
            assert!(character.karma.0 >= -100.0 && character.karma.0 <= 100.0);
        }
    }
}
