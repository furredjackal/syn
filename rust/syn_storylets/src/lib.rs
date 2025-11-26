//! # SYN Storylets: Modular Narrative Atoms
//!
//! This crate provides the core data types and structures for the storylet system.
//! Storylets are the fundamental building blocks of the Event Director's narrative engine—
//! they represent modular, reusable narrative atoms that encode prerequisites, triggers,
//! outcomes, and all the conditional logic needed to tell stories in the SYN world.
//!
//! ## Design Philosophy
//!
//! - **Deterministic Narrative**: All storylet selection, outcomes, and timing derive from a
//!   single world seed, ensuring perfect reproducibility.
//! - **Emergent Storytelling**: Stories emerge from systems, not scripts. Storylets interact
//!   through NPC stats, relationships, traits, memories, and district state.
//! - **Modular Composition**: Each storylet is self-contained but can reference other storylets
//!   as follow-ups, creating narrative chains.
//! - **Authored as JSON**: Story designers write storylets in JSON format for human readability
//!   and easier authoring. At runtime, they're compiled into a binary format with indexes.
//!
//! ## Key Types
//!
//! - [`StoryletId`]: Deterministic identifier for a storylet.
//! - [`Tag`]: Semantic tags like "romance", "conflict", "trauma", etc.
//! - [`StoryDomain`]: High-level narrative category (Romance, Career, Family, etc.).
//! - [`LifeStage`]: Character life stage filter (child, teen, adult, elder, digital).
//! - [`TriggerKind`]: How a storylet activates (time_tick, player_action, mood_spike, etc.).
//! - [`Prerequisites`]: Structured filters for stats, traits, relationships, memory, and world state.
//! - [`Cooldowns`]: Global, per-actor, per-relationship, and per-district cooldown timers.
//! - [`Outcome`]: What happens when a storylet resolves (stat/relationship/mood deltas, flags, memories).
//! - [`RoleSlot`]: Named roles that must be filled by NPCs ("protagonist", "target", "rival", etc.).
//! - [`StoryletDef`]: The complete storylet definition bundling all the above.
//!
//! ## Validation
//!
//! The `validation` module provides a configurable validator for `StoryletDef` instances.
//! See [`validation::default_storylet_validator`] for sensible defaults or build custom validators.
//!
//! ## Compilation
//!
//! The `compiler` module enables offline compilation of JSON storylets into an indexed binary library.
//! The `binary` module handles serialization/deserialization of compiled libraries.
//!
//! # Example: Compiling Storylets
//! ```text
//! $ ./target/release/storyletc --input ./storylets --output ./storylets.bin
//! ```

use serde::{Deserialize, Serialize};

pub mod validation;
pub mod library;
pub mod compiler;
pub mod binary;
pub mod errors;

#[cfg(feature = "mmap")]
pub mod mapped;

/// A deterministic identifier for a storylet.
///
/// This is typically a short, human-readable string key that uniquely identifies a storylet
/// within the authored content. Examples: "romance_first_date", "career_promotion", "trauma_confrontation".
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct StoryletId(pub String);

impl StoryletId {
    /// Create a new StoryletId from a string.
    pub fn new(s: impl Into<String>) -> Self {
        StoryletId(s.into())
    }
}

/// A semantic tag for categorizing storylets.
///
/// Tags enable cross-cutting concerns like content warnings ("adult", "trauma"),
/// narrative themes ("romance", "conflict", "comedy"), and accessibility/lifecycle
/// filtering ("teen", "child", "elder"). Multiple tags per storylet allow flexible
/// content filtering and querying.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Tag(pub String);

impl Tag {
    /// Create a new Tag from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Tag(s.into())
    }
}

/// High-level narrative domain or category.
///
/// Domains define the broad thematic space of a storylet. They help the Event Director
/// manage pacing and variety by distributing events across emotional/social domains
/// rather than overwhelming the player with a single theme.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryDomain {
    /// Romantic relationships, attraction, intimacy, heartbreak.
    Romance,
    /// Conflict, arguments, rivalry, confrontation, betrayal.
    Conflict,
    /// Career advancement, job loss, workplace drama, mentorship.
    Career,
    /// Psychological trauma, grief, loss, recovery, healing.
    Trauma,
    /// Addiction, dependency, recovery, support, relapse.
    Addiction,
    /// Family relationships, inheritance, generational dynamics.
    Family,
    /// Platonic bonds, social groups, belonging, isolation.
    Friendship,
    /// Mundane daily life, routines, minor interactions, slice-of-life.
    SliceOfLife,
    /// District-level events (crime wave, economic crash, cultural shift).
    District,
    /// Digital/virtual world events, AI interactions, net culture.
    Digital,
}

/// Character life stage, used for age-gating and demographic filtering.
///
/// These stages determine which storylets are eligible for a character at any given time.
/// For example, a "first crush" storylet should only fire for teens/young adults, not children.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeStage {
    /// Childhood (0–12 years).
    Child,
    /// Adolescence (13–19 years).
    Teen,
    /// Young adulthood (20–35 years).
    YoungAdult,
    /// Adulthood (36–65 years).
    Adult,
    /// Senior life (66+ years).
    Elder,
    /// Digital entity (AI, uploaded consciousness).
    Digital,
}

/// Event trigger types: how and when a storylet can fire.
///
/// The Event Director uses trigger kinds to determine eligibility. A storylet might only
/// fire on a time tick, only in response to a player choice, or only when a mood spike occurs.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerKind {
    /// Regular time-based (tick) trigger: checked every hour (1 tick).
    TimeTick,
    /// Player directly chose this as a narrative option.
    PlayerAction,
    /// Automatic trigger when an NPC's mood spikes (rapid change).
    MoodSpike,
    /// Triggered by a memory echo (recalling past events).
    MemoryEcho,
    /// District-level pulse trigger (economic/crime updates).
    DistrictPulse,
    /// Custom application-defined trigger.
    Custom(String),
}

/// Thresholds for stats that must be satisfied for a storylet to be eligible.
///
/// Stats in SYN represent quantified emotional/social attributes of characters.
/// Examples: mood, stress, wealth, reputation. Each stat can be filtered with
/// lower and upper bounds, allowing storylets like "depression spiral" (low mood)
/// or "crisis of privilege" (high wealth) to gate themselves appropriately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatThresholds {
    /// Stat name (e.g., "mood", "stress", "wealth").
    pub stat: String,
    /// Inclusive minimum. If None, no lower bound.
    pub min: Option<f32>,
    /// Inclusive maximum. If None, no upper bound.
    pub max: Option<f32>,
}

/// Thresholds for personality traits.
///
/// Traits like "impulsivity", "empathy", "ambition" represent stable personality patterns.
/// Storylets can gate on trait thresholds to ensure personality-appropriate events
/// (e.g., high-impulsivity characters are more likely to get into risky situations).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraitThresholds {
    /// Trait name (e.g., "impulsivity", "empathy").
    pub trait_name: String,
    /// Inclusive minimum. If None, no lower bound.
    pub min: Option<f32>,
    /// Inclusive maximum. If None, no upper bound.
    pub max: Option<f32>,
}

/// Thresholds for a specific relationship vector axis.
///
/// The five-axis relationship model (Affection, Trust, Attraction, Familiarity, Resentment)
/// each range from -10.0 to +10.0. Storylets can gate on individual axes, e.g.,
/// "romantic confession" requires high Affection + high Attraction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipThreshold {
    /// The relationship axis name: one of "affection", "trust", "attraction", "familiarity", "resentment".
    pub axis: String,
    /// Inclusive minimum value.
    pub min: Option<f32>,
    /// Inclusive maximum value.
    pub max: Option<f32>,
}

/// Relationship-level prerequisites: target character must satisfy these conditions.
///
/// This filters based on relationships between specific role slots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipPrerequisites {
    /// The source role (e.g., "protagonist").
    pub from_role: String,
    /// The target role (e.g., "target").
    pub to_role: String,
    /// List of relationship axis thresholds.
    pub thresholds: Vec<RelationshipThreshold>,
}

/// Prerequisites related to an NPC's memory system.
///
/// The memory system tracks significant emotional events and milestones.
/// Storylets can require or forbid certain memory tags as gating mechanisms,
/// e.g., "reconciliation" storylet requires prior "conflict" memory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryPrerequisites {
    /// Memory tags that MUST exist for this storylet to be eligible.
    pub must_have_tags: Vec<String>,
    /// Memory tags that MUST NOT exist (to prevent re-triggering).
    pub must_not_have_tags: Vec<String>,
}

/// Global world state prerequisites.
///
/// These gate storylets on district/economy/global conditions rather than character state.
/// Examples: "recession_storylets" only fire if the economy is in recession,
/// "crime_wave_reaction" requires high district crime level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldStatePrerequisites {
    /// Minimum crime level for the character's district.
    pub min_crime_level: Option<f32>,
    /// If true, the storylet only fires when a recession is active globally.
    pub recession_active: Option<bool>,
    /// Optional black swan event ID that must be active.
    pub required_black_swan_id: Option<String>,
}

/// Global flags: arbitrary boolean bits that can gate storylets.
///
/// Useful for implementing branching state machines and one-time events.
/// For example: "first_job_storylet" requires flag "has_ever_worked" to be false.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalFlags {
    /// Flags that must be set (true).
    pub must_be_set: Vec<String>,
    /// Flags that must be unset (false).
    pub must_be_unset: Vec<String>,
}

/// A named role that must be filled by an NPC within a storylet.
///
/// Roles define the "slots" that NPCs fill during a narrative event. Examples:
/// - "protagonist": the focal character
/// - "target": the other character in a dyadic relationship
/// - "rival": an antagonistic presence
/// - "manager": an authority figure
///
/// Each role can have optional constraints (e.g., minimum relationship affection for a "love_interest" role).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleSlot {
    /// Role name.
    pub name: String,
    /// If true, this role must be filled for the storylet to fire.
    pub required: bool,
    /// Optional stat or trait constraints for the actor filling this role.
    pub constraints: Option<String>,
}

/// The complete set of prerequisites that must be satisfied for a storylet to be eligible.
///
/// Prerequisites are AND'ed together: all conditions that are specified must be true.
/// Unspecified conditions are ignored (treated as "always true").
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Prerequisites {
    /// Life stage gate: character must be in one of these stages.
    pub life_stages: Option<Vec<LifeStage>>,
    /// Stat thresholds: all listed stats must fall within their bounds.
    pub stat_thresholds: Option<Vec<StatThresholds>>,
    /// Personality trait thresholds.
    pub trait_thresholds: Option<Vec<TraitThresholds>>,
    /// Relationship requirements between role slots.
    pub relationship_prerequisites: Option<Vec<RelationshipPrerequisites>>,
    /// Memory-based gating.
    pub memory_prerequisites: Option<MemoryPrerequisites>,
    /// Global world state checks.
    pub world_state_prerequisites: Option<WorldStatePrerequisites>,
    /// Global flag checks.
    pub global_flags: Option<GlobalFlags>,
}

/// Cooldown timers to prevent storylets from firing too frequently.
///
/// Cooldowns are essential to pacing and prevent narrative fatigue.
/// Different cooldown types allow fine-grained control:
/// - Global: affects entire storylet across all actors
/// - Per-actor: per NPC instance
/// - Per-relationship: per unique pair of characters
/// - Per-district: per district context
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Cooldowns {
    /// Minimum ticks (hours) before this storylet can fire again globally.
    pub global_cooldown_ticks: Option<u32>,
    /// Minimum ticks before this storylet can fire for a specific NPC.
    pub per_actor_cooldown_ticks: Option<u32>,
    /// Minimum ticks before this storylet can fire between a specific pair of characters.
    pub per_relationship_cooldown_ticks: Option<u32>,
    /// Minimum ticks before this storylet can fire in a specific district.
    pub per_district_cooldown_ticks: Option<u32>,
}

/// A delta to apply to an NPC stat.
///
/// When a storylet resolves, it can modify NPC stats. For example, a "breakup" storylet
/// might reduce Affection by 5 and increase Resentment by 3.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatDelta {
    /// Stat name.
    pub stat: String,
    /// Delta to apply (positive or negative).
    pub delta: f32,
}

/// A delta to apply to a relationship axis.
///
/// Modifies one axis of a relationship vector. For example, a "betrayal" storylet
/// might decrease Trust by 7 and increase Resentment by 5.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipDelta {
    /// Source role.
    pub from_role: String,
    /// Target role.
    pub to_role: String,
    /// Axis name ("affection", "trust", etc.).
    pub axis: String,
    /// Delta to apply.
    pub delta: f32,
}

/// A mood shift to apply to an NPC.
///
/// Mood represents short-term emotional state. Storylets often cause mood swings
/// (e.g., "romantic breakthrough" might spike mood by +8).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoodDelta {
    /// Role that experiences the mood shift.
    pub role: String,
    /// Mood delta.
    pub delta: f32,
}

/// A trait change to apply to an NPC.
///
/// Storylets can slowly shift personality traits. For example, surviving a trauma
/// might increase "resilience" by 1 and "trust" by -0.5.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraitChange {
    /// Role that experiences the trait change.
    pub role: String,
    /// Trait name.
    pub trait_name: String,
    /// Change magnitude.
    pub change: f32,
}

/// A flag to set or clear in global state.
///
/// Useful for marking one-time events or state transitions that affect storylet eligibility
/// (e.g., "first_love_experienced" flag for gating follow-up romance storylets).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlagOperation {
    /// Flag name.
    pub flag: String,
    /// If true, set the flag. If false, clear it.
    pub set: bool,
}

/// A memory entry to create when a storylet resolves.
///
/// Memories are tagged, emotionally-weighted records of significant events.
/// They affect future storylet eligibility and can trigger "memory echo" events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryEntry {
    /// Role(s) who experience this memory (comma-separated if multiple).
    pub roles: String,
    /// Tags describing the memory (e.g., "romance", "betrayal", "milestone").
    pub tags: Vec<String>,
    /// Emotional intensity (range typically 0–10).
    pub intensity: u8,
    /// Optional narrative description for debug/UI.
    pub description: Option<String>,
}

/// A follow-up storylet reference.
///
/// Allows chaining: when this storylet resolves, another can be scheduled.
/// Can be immediate (same tick), delayed (Nth tick), or conditional (if flags are met).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FollowUpStorylet {
    /// The ID of the follow-up storylet.
    pub storylet_id: String,
    /// Delay in ticks before this follow-up fires (0 = immediate).
    pub delay_ticks: u32,
    /// Optional condition: must be empty or this flag must be set.
    pub conditional_on_flag: Option<String>,
}

/// All outcomes that resolve when a storylet is selected.
///
/// Outcomes bundle stat deltas, relationship changes, mood shifts, trait modifications,
/// flag operations, memory entries, and potential follow-up storylets into a cohesive
/// narrative resolution.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Outcome {
    /// Stats to modify.
    pub stat_deltas: Option<Vec<StatDelta>>,
    /// Relationships to modify.
    pub relationship_deltas: Option<Vec<RelationshipDelta>>,
    /// Mood shifts.
    pub mood_deltas: Option<Vec<MoodDelta>>,
    /// Trait changes.
    pub trait_changes: Option<Vec<TraitChange>>,
    /// Global flags to set/clear.
    pub flag_operations: Option<Vec<FlagOperation>>,
    /// Memory entries to create.
    pub memory_entries: Option<Vec<MemoryEntry>>,
    /// Follow-up storylets to schedule.
    pub follow_ups: Option<Vec<FollowUpStorylet>>,
}

/// A complete storylet definition.
///
/// This is the authoritative data structure that combines all aspects of a narrative atom:
/// identification, prerequisites, triggers, cooldowns, roles, and outcomes.
/// When authored in JSON and loaded at runtime, this structure is parsed, validated,
/// and compiled into runtime-efficient formats (indexed, hashed, etc.) by the Event Director.
///
/// # Example
/// A "first_date" romance storylet might have:
/// - `domain`: Romance
/// - `heat`: 3 (mild intensity)
/// - `weight`: 0.4 (moderate frequency when eligible)
/// - `roles`: ["protagonist", "love_interest"]
/// - Prerequisites: high Affection + high Attraction, no prior breakup memory
/// - Outcome: relationship boost, mood spike, memory creation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryletDef {
    /// Unique identifier for this storylet.
    pub id: StoryletId,
    /// Human-readable name.
    pub name: String,
    /// Brief description of the narrative.
    pub description: Option<String>,
    /// Semantic tags for filtering and categorization.
    pub tags: Vec<Tag>,
    /// Primary narrative domain.
    pub domain: StoryDomain,
    /// Target life stage(s).
    pub life_stage: LifeStage,
    /// Narrative intensity (0–10; higher = more emotionally significant).
    pub heat: u8,
    /// Base weight for selection when eligible (e.g., 0.5 = 50% relative probability).
    pub weight: f32,
    /// Named roles that must be filled by NPCs.
    pub roles: Vec<RoleSlot>,
    /// All prerequisite conditions.
    pub prerequisites: Prerequisites,
    /// Trigger kinds: how this storylet can fire.
    pub triggers: Vec<TriggerKind>,
    /// Cooldown settings.
    pub cooldowns: Cooldowns,
    /// Outcomes that occur when this storylet is selected.
    pub outcomes: Outcome,
}

impl StoryletDef {
    /// Create a new StoryletDef with default/minimal values.
    pub fn new(id: StoryletId, name: String, domain: StoryDomain, life_stage: LifeStage) -> Self {
        StoryletDef {
            id,
            name,
            description: None,
            tags: vec![],
            domain,
            life_stage,
            heat: 5,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites::default(),
            triggers: vec![TriggerKind::TimeTick],
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storylet_serialization_roundtrip() {
        // Construct a sample storylet
        let mut storylet = StoryletDef::new(
            StoryletId::new("test_romance_date"),
            "First Date".to_string(),
            StoryDomain::Romance,
            LifeStage::YoungAdult,
        );

        storylet.description = Some("Two characters go on a romantic date.".to_string());
        storylet.tags = vec![Tag::new("romance"), Tag::new("milestone")];
        storylet.heat = 4;
        storylet.weight = 0.6;

        // Add roles
        storylet.roles = vec![
            RoleSlot {
                name: "protagonist".to_string(),
                required: true,
                constraints: None,
            },
            RoleSlot {
                name: "love_interest".to_string(),
                required: true,
                constraints: Some("affection > 3".to_string()),
            },
        ];

        // Add prerequisites
        storylet.prerequisites = Prerequisites {
            life_stages: Some(vec![LifeStage::YoungAdult, LifeStage::Adult]),
            stat_thresholds: Some(vec![StatThresholds {
                stat: "mood".to_string(),
                min: Some(-5.0),
                max: None,
            }]),
            relationship_prerequisites: Some(vec![RelationshipPrerequisites {
                from_role: "protagonist".to_string(),
                to_role: "love_interest".to_string(),
                thresholds: vec![RelationshipThreshold {
                    axis: "affection".to_string(),
                    min: Some(4.0),
                    max: None,
                }],
            }]),
            ..Default::default()
        };

        // Add triggers
        storylet.triggers = vec![TriggerKind::TimeTick, TriggerKind::PlayerAction];

        // Add cooldowns
        storylet.cooldowns = Cooldowns {
            global_cooldown_ticks: Some(240), // 10 days
            per_actor_cooldown_ticks: Some(120),
            per_relationship_cooldown_ticks: Some(480),
            per_district_cooldown_ticks: None,
        };

        // Add outcomes
        storylet.outcomes = Outcome {
            relationship_deltas: Some(vec![RelationshipDelta {
                from_role: "protagonist".to_string(),
                to_role: "love_interest".to_string(),
                axis: "affection".to_string(),
                delta: 2.5,
            }]),
            mood_deltas: Some(vec![
                MoodDelta {
                    role: "protagonist".to_string(),
                    delta: 3.0,
                },
                MoodDelta {
                    role: "love_interest".to_string(),
                    delta: 2.5,
                },
            ]),
            memory_entries: Some(vec![MemoryEntry {
                roles: "protagonist,love_interest".to_string(),
                tags: vec!["romance".to_string(), "milestone".to_string()],
                intensity: 7,
                description: Some("Went on a romantic date together.".to_string()),
            }]),
            ..Default::default()
        };

        // Serialize to JSON
        let json = serde_json::to_string(&storylet).expect("Failed to serialize");

        // Deserialize back
        let deserialized: StoryletDef =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert equality
        assert_eq!(storylet, deserialized, "Roundtrip serialization failed");
    }

    #[test]
    fn test_storylet_json_format() {
        let storylet = StoryletDef::new(
            StoryletId::new("simple_test"),
            "Simple Storylet".to_string(),
            StoryDomain::SliceOfLife,
            LifeStage::Adult,
        );

        let json = serde_json::to_string_pretty(&storylet).expect("Failed to serialize");
        println!("Storylet JSON:\n{}", json);

        // Verify it contains expected keys
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"domain\""));
        assert!(json.contains("\"life_stage\""));
    }
}
