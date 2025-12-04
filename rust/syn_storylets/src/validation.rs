//! Validation layer for storylet definitions.
//!
//! This module provides structured validation for `StoryletDef` instances,
//! checking against game design constraints and allowed vocabularies.
//! It supports both compiled-time (offline) and runtime validation paths.

use crate::{Cooldowns, MemoryEntry, Outcome, Prerequisites, StoryletDef, StoryletId};
use std::collections::HashSet;

/// Validation error types for storylet definitions.
///
/// Each variant describes a specific validation failure, enabling
/// precise error reporting to authors and downstream consumers.
#[derive(Debug, Clone, PartialEq)]
pub enum StoryletValidationError {
    /// Storylet ID is missing or empty.
    MissingId,
    /// Storylet ID doesn't match the required pattern: lowercase alphanumerics, `.`, `_`.
    InvalidIdFormat {
        id: String,
        reason: String,
    },
    /// Storylet name is missing or empty.
    MissingName,
    /// Heat value is out of valid range (0–10).
    InvalidHeatRange { value: u8 },
    /// Weight is not positive (must be > 0.0).
    InvalidWeight { value: f32 },
    /// A stat name referenced in prerequisites is not in the allowed set.
    UnknownStatName { stat: String },
    /// A trait name referenced in prerequisites is not in the allowed set.
    UnknownTraitName { trait_name: String },
    /// A relationship axis is not in the allowed set.
    UnknownRelationshipAxis { axis: String },
    /// A stat threshold has an invalid range (min > max or outside plausible bounds).
    InvalidStatThreshold {
        stat: String,
        min: Option<f32>,
        max: Option<f32>,
        reason: String,
    },
    /// A trait threshold has an invalid range.
    InvalidTraitThreshold {
        trait_name: String,
        reason: String,
    },
    /// A relationship threshold is outside the -10..=10 range.
    InvalidRelationshipThreshold {
        axis: String,
        min: Option<f32>,
        max: Option<f32>,
    },
    /// A cooldown value is invalid (negative or too large).
    InvalidCooldown {
        cooldown_type: String,
        value: u32,
        reason: String,
    },
    /// A follow-up storylet ID has invalid format.
    InvalidFollowUpId {
        id: String,
        reason: String,
    },
    /// A role name is missing or invalid.
    InvalidRoleName { name: String },
    /// A tag contains disallowed characters or is empty.
    InvalidTag { tag: String },
    /// A memory tag is not in the allowed set.
    UnknownMemoryTag { tag: String },
    /// A global flag is not in the allowed set.
    UnknownGlobalFlag { flag: String },
    /// A memory entry contains invalid roles or other issues.
    InvalidMemoryEntry { reason: String },
}

impl std::fmt::Display for StoryletValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingId => write!(f, "Storylet ID is missing or empty"),
            Self::InvalidIdFormat { id, reason } => {
                write!(f, "Invalid storylet ID '{}': {}", id, reason)
            }
            Self::MissingName => write!(f, "Storylet name is missing or empty"),
            Self::InvalidHeatRange { value } => {
                write!(f, "Heat value {} is outside valid range 0–10", value)
            }
            Self::InvalidWeight { value } => {
                write!(f, "Weight {} must be positive (> 0.0)", value)
            }
            Self::UnknownStatName { stat } => {
                write!(f, "Stat '{}' is not in the allowed stat set", stat)
            }
            Self::UnknownTraitName { trait_name } => {
                write!(f, "Trait '{}' is not in the allowed trait set", trait_name)
            }
            Self::UnknownRelationshipAxis { axis } => {
                write!(f, "Relationship axis '{}' is not in the allowed set", axis)
            }
            Self::InvalidStatThreshold {
                stat,
                min,
                max,
                reason,
            } => {
                write!(
                    f,
                    "Stat '{}' threshold [min={:?}, max={:?}] is invalid: {}",
                    stat, min, max, reason
                )
            }
            Self::InvalidTraitThreshold { trait_name, reason } => {
                write!(f, "Trait '{}' threshold is invalid: {}", trait_name, reason)
            }
            Self::InvalidRelationshipThreshold { axis, min, max } => {
                write!(
                    f,
                    "Relationship axis '{}' threshold [min={:?}, max={:?}] is outside -10..=10 range",
                    axis, min, max
                )
            }
            Self::InvalidCooldown {
                cooldown_type,
                value,
                reason,
            } => {
                write!(
                    f,
                    "{} cooldown value {} is invalid: {}",
                    cooldown_type, value, reason
                )
            }
            Self::InvalidFollowUpId { id, reason } => {
                write!(f, "Invalid follow-up storylet ID '{}': {}", id, reason)
            }
            Self::InvalidRoleName { name } => {
                write!(f, "Invalid role name '{}' (must be non-empty)", name)
            }
            Self::InvalidTag { tag } => {
                write!(f, "Invalid tag '{}' (must be non-empty, alphanumeric/underscore)", tag)
            }
            Self::UnknownMemoryTag { tag } => {
                write!(f, "Memory tag '{}' is not in the allowed set", tag)
            }
            Self::UnknownGlobalFlag { flag } => {
                write!(f, "Global flag '{}' is not in the allowed set", flag)
            }
            Self::InvalidMemoryEntry { reason } => {
                write!(f, "Invalid memory entry: {}", reason)
            }
        }
    }
}

impl std::error::Error for StoryletValidationError {}

/// Configuration for validating storylet definitions.
///
/// The validator is initialized with allowed vocabularies (stat names, trait names, etc.)
/// and can then validate any number of `StoryletDef` instances against these constraints.
///
/// # Example
/// ```
/// # use syn_storylets::validation::StoryletValidator;
/// let validator = syn_storylets::validation::default_storylet_validator();
/// // Use validator.validate_storylet(...) on storylets
/// ```
#[derive(Debug, Clone)]
pub struct StoryletValidator {
    /// Allowed stat names (e.g., "mood", "wealth", "health", "karma", "stress").
    allowed_stats: HashSet<String>,
    /// Allowed personality trait names (e.g., "impulsivity", "empathy", "ambition").
    allowed_traits: HashSet<String>,
    /// Allowed relationship axes (always: "affection", "trust", "attraction", "familiarity", "resentment").
    allowed_axes: HashSet<String>,
    /// Allowed global flag names for flag gates.
    allowed_flags: HashSet<String>,
    /// Allowed memory tag names.
    allowed_memory_tags: HashSet<String>,
    /// Expected range for stat values (used in threshold validation).
    stat_ranges: std::collections::HashMap<String, (f32, f32)>,
}

impl StoryletValidator {
    /// Create a new validator with empty allowed sets.
    /// Use builder methods to configure or use `default_storylet_validator()` for sensible defaults.
    pub fn new() -> Self {
        StoryletValidator {
            allowed_stats: HashSet::new(),
            allowed_traits: HashSet::new(),
            allowed_axes: HashSet::new(),
            allowed_flags: HashSet::new(),
            allowed_memory_tags: HashSet::new(),
            stat_ranges: std::collections::HashMap::new(),
        }
    }

    /// Add an allowed stat name.
    pub fn with_stat(mut self, stat: impl Into<String>) -> Self {
        self.allowed_stats.insert(stat.into());
        self
    }

    /// Add multiple allowed stat names.
    pub fn with_stats(mut self, stats: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for stat in stats {
            self.allowed_stats.insert(stat.into());
        }
        self
    }

    /// Add a stat with its expected range for validation.
    pub fn with_stat_range(mut self, stat: impl Into<String>, min: f32, max: f32) -> Self {
        let stat_name = stat.into();
        self.allowed_stats.insert(stat_name.clone());
        self.stat_ranges.insert(stat_name, (min, max));
        self
    }

    /// Add an allowed trait name.
    pub fn with_trait(mut self, trait_name: impl Into<String>) -> Self {
        self.allowed_traits.insert(trait_name.into());
        self
    }

    /// Add multiple allowed trait names.
    pub fn with_traits(mut self, traits: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for trait_name in traits {
            self.allowed_traits.insert(trait_name.into());
        }
        self
    }

    /// Add a trait with its expected range for validation.
    pub fn with_trait_range(mut self, trait_name: impl Into<String>, min: f32, max: f32) -> Self {
        let trait_str = trait_name.into();
        self.allowed_traits.insert(trait_str.clone());
        self.stat_ranges.insert(trait_str, (min, max));
        self
    }

    /// Set allowed relationship axes (typically: affection, trust, attraction, familiarity, resentment).
    pub fn with_axes(mut self, axes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for axis in axes {
            self.allowed_axes.insert(axis.into());
        }
        self
    }

    /// Add an allowed global flag.
    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.allowed_flags.insert(flag.into());
        self
    }

    /// Add multiple allowed global flags.
    pub fn with_flags(mut self, flags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for flag in flags {
            self.allowed_flags.insert(flag.into());
        }
        self
    }

    /// Add an allowed memory tag.
    pub fn with_memory_tag(mut self, tag: impl Into<String>) -> Self {
        self.allowed_memory_tags.insert(tag.into());
        self
    }

    /// Add multiple allowed memory tags.
    pub fn with_memory_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.allowed_memory_tags.insert(tag.into());
        }
        self
    }

    /// Validate a storylet definition.
    ///
    /// Returns `Ok(())` if the storylet is valid.
    /// Returns `Err(vec)` containing all validation errors found.
    pub fn validate_storylet(&self, s: &StoryletDef) -> Result<(), Vec<StoryletValidationError>> {
        let mut errors = Vec::new();

        // Validate ID
        errors.extend(self.validate_id(&s.id.0));

        // Validate name
        if s.name.trim().is_empty() {
            errors.push(StoryletValidationError::MissingName);
        }

        // Validate heat range (0–10)
        if s.heat > 10 {
            errors.push(StoryletValidationError::InvalidHeatRange { value: s.heat });
        }

        // Validate weight > 0
        if s.weight <= 0.0 || !s.weight.is_finite() {
            errors.push(StoryletValidationError::InvalidWeight { value: s.weight });
        }

        // Validate tags
        for tag in &s.tags {
            if tag.0.is_empty() {
                errors.push(StoryletValidationError::InvalidTag {
                    tag: tag.0.clone(),
                });
            }
        }

        // Validate prerequisites
        errors.extend(self.validate_prerequisites(&s.prerequisites));

        // Validate cooldowns
        errors.extend(self.validate_cooldowns(&s.cooldowns));

        // Validate outcomes
        errors.extend(self.validate_outcomes(&s.outcomes));

        // Validate roles
        for role in &s.roles {
            if role.name.trim().is_empty() {
                errors.push(StoryletValidationError::InvalidRoleName {
                    name: role.name.clone(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate the ID format: lowercase alphanumerics, `.`, `_`.
    fn validate_id(&self, id: &str) -> Vec<StoryletValidationError> {
        let mut errors = Vec::new();

        if id.is_empty() {
            errors.push(StoryletValidationError::MissingId);
            return errors;
        }

        let is_valid_char = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_';

        if !id.chars().all(is_valid_char) {
            errors.push(StoryletValidationError::InvalidIdFormat {
                id: id.to_string(),
                reason: "must contain only lowercase alphanumerics, '.', or '_'".to_string(),
            });
        }

        // Check it doesn't start or end with `.` or `_`
        if id.starts_with('.') || id.starts_with('_') {
            errors.push(StoryletValidationError::InvalidIdFormat {
                id: id.to_string(),
                reason: "must not start with '.' or '_'".to_string(),
            });
        }

        if id.ends_with('.') || id.ends_with('_') {
            errors.push(StoryletValidationError::InvalidIdFormat {
                id: id.to_string(),
                reason: "must not end with '.' or '_'".to_string(),
            });
        }

        errors
    }

    /// Validate prerequisites.
    fn validate_prerequisites(
        &self,
        prereqs: &Prerequisites,
    ) -> Vec<StoryletValidationError> {
        let mut errors = Vec::new();

        // Validate stat thresholds
        if let Some(stat_thresholds) = &prereqs.stat_thresholds {
            for st in stat_thresholds {
                // Check stat is allowed
                if !self.allowed_stats.contains(&st.stat) {
                    errors.push(StoryletValidationError::UnknownStatName {
                        stat: st.stat.clone(),
                    });
                    continue;
                }

                // Check range validity
                if let Some((min_allowed, max_allowed)) = self.stat_ranges.get(&st.stat) {
                    let check_min = st.min.unwrap_or(*min_allowed);
                    let check_max = st.max.unwrap_or(*max_allowed);

                    if check_min > check_max {
                        errors.push(StoryletValidationError::InvalidStatThreshold {
                            stat: st.stat.clone(),
                            min: st.min,
                            max: st.max,
                            reason: "min > max".to_string(),
                        });
                    }

                    if let Some(min_val) = st.min {
                        if min_val < *min_allowed || min_val > *max_allowed {
                            errors.push(StoryletValidationError::InvalidStatThreshold {
                                stat: st.stat.clone(),
                                min: st.min,
                                max: st.max,
                                reason: format!("min {} outside range {}..={}", min_val, min_allowed, max_allowed),
                            });
                        }
                    }

                    if let Some(max_val) = st.max {
                        if max_val < *min_allowed || max_val > *max_allowed {
                            errors.push(StoryletValidationError::InvalidStatThreshold {
                                stat: st.stat.clone(),
                                min: st.min,
                                max: st.max,
                                reason: format!("max {} outside range {}..={}", max_val, min_allowed, max_allowed),
                            });
                        }
                    }
                }
            }
        }

        // Validate trait thresholds
        if let Some(trait_thresholds) = &prereqs.trait_thresholds {
            for tt in trait_thresholds {
                if !self.allowed_traits.contains(&tt.trait_name) {
                    errors.push(StoryletValidationError::UnknownTraitName {
                        trait_name: tt.trait_name.clone(),
                    });
                    continue;
                }

                // Check range validity
                if let Some((min_allowed, max_allowed)) = self.stat_ranges.get(&tt.trait_name) {
                    let check_min = tt.min.unwrap_or(*min_allowed);
                    let check_max = tt.max.unwrap_or(*max_allowed);

                    if check_min > check_max {
                        errors.push(StoryletValidationError::InvalidTraitThreshold {
                            trait_name: tt.trait_name.clone(),
                            reason: "min > max".to_string(),
                        });
                    }
                }
            }
        }

        // Validate relationship prerequisites
        if let Some(rel_prereqs) = &prereqs.relationship_prerequisites {
            for rp in rel_prereqs {
                for threshold in &rp.thresholds {
                    if !self.allowed_axes.contains(&threshold.axis) {
                        errors.push(StoryletValidationError::UnknownRelationshipAxis {
                            axis: threshold.axis.clone(),
                        });
                    }

                    // Relationship axes are always in -10..=10 range
                    if let Some(min) = threshold.min {
                        if min < -10.0 || min > 10.0 {
                            errors.push(StoryletValidationError::InvalidRelationshipThreshold {
                                axis: threshold.axis.clone(),
                                min: Some(min),
                                max: threshold.max,
                            });
                        }
                    }

                    if let Some(max) = threshold.max {
                        if max < -10.0 || max > 10.0 {
                            errors.push(StoryletValidationError::InvalidRelationshipThreshold {
                                axis: threshold.axis.clone(),
                                min: threshold.min,
                                max: Some(max),
                            });
                        }
                    }

                    if let (Some(min), Some(max)) = (threshold.min, threshold.max) {
                        if min > max {
                            errors.push(StoryletValidationError::InvalidRelationshipThreshold {
                                axis: threshold.axis.clone(),
                                min: Some(min),
                                max: Some(max),
                            });
                        }
                    }
                }
            }
        }

        // Validate memory prerequisites
        if let Some(mem_prereqs) = &prereqs.memory_prerequisites {
            for tag in &mem_prereqs.must_have_tags {
                if !self.allowed_memory_tags.contains(tag) {
                    errors.push(StoryletValidationError::UnknownMemoryTag {
                        tag: tag.clone(),
                    });
                }
            }

            for tag in &mem_prereqs.must_not_have_tags {
                if !self.allowed_memory_tags.contains(tag) {
                    errors.push(StoryletValidationError::UnknownMemoryTag {
                        tag: tag.clone(),
                    });
                }
            }
        }

        // Validate global flags
        if let Some(global_flags) = &prereqs.global_flags {
            for flag in &global_flags.must_be_set {
                if !self.allowed_flags.contains(flag) {
                    errors.push(StoryletValidationError::UnknownGlobalFlag {
                        flag: flag.clone(),
                    });
                }
            }

            for flag in &global_flags.must_be_unset {
                if !self.allowed_flags.contains(flag) {
                    errors.push(StoryletValidationError::UnknownGlobalFlag {
                        flag: flag.clone(),
                    });
                }
            }
        }

        errors
    }

    /// Validate cooldowns.
    fn validate_cooldowns(&self, cooldowns: &Cooldowns) -> Vec<StoryletValidationError> {
        let mut errors = Vec::new();

        if let Some(global) = cooldowns.global_cooldown_ticks {
            if global > 100_000 {
                errors.push(StoryletValidationError::InvalidCooldown {
                    cooldown_type: "global".to_string(),
                    value: global,
                    reason: "cooldown too large (max 100,000 ticks)".to_string(),
                });
            }
        }

        if let Some(per_actor) = cooldowns.per_actor_cooldown_ticks {
            if per_actor > 100_000 {
                errors.push(StoryletValidationError::InvalidCooldown {
                    cooldown_type: "per_actor".to_string(),
                    value: per_actor,
                    reason: "cooldown too large (max 100,000 ticks)".to_string(),
                });
            }
        }

        if let Some(per_rel) = cooldowns.per_relationship_cooldown_ticks {
            if per_rel > 100_000 {
                errors.push(StoryletValidationError::InvalidCooldown {
                    cooldown_type: "per_relationship".to_string(),
                    value: per_rel,
                    reason: "cooldown too large (max 100,000 ticks)".to_string(),
                });
            }
        }

        if let Some(per_dist) = cooldowns.per_district_cooldown_ticks {
            if per_dist > 100_000 {
                errors.push(StoryletValidationError::InvalidCooldown {
                    cooldown_type: "per_district".to_string(),
                    value: per_dist,
                    reason: "cooldown too large (max 100,000 ticks)".to_string(),
                });
            }
        }

        errors
    }

    /// Validate outcomes.
    fn validate_outcomes(&self, outcomes: &Outcome) -> Vec<StoryletValidationError> {
        let mut errors = Vec::new();

        // Validate follow-up IDs
        if let Some(follow_ups) = &outcomes.follow_ups {
            for fu in follow_ups {
                errors.extend(self.validate_followup_id(&fu.storylet_id));
            }
        }

        // Validate memory entries
        if let Some(memories) = &outcomes.memory_entries {
            for mem in memories {
                errors.extend(self.validate_memory_entry(mem));
            }
        }

        // Validate flag operations
        if let Some(flags) = &outcomes.flag_operations {
            for flag_op in flags {
                if !self.allowed_flags.contains(&flag_op.flag) {
                    errors.push(StoryletValidationError::UnknownGlobalFlag {
                        flag: flag_op.flag.clone(),
                    });
                }
            }
        }

        errors
    }

    /// Validate follow-up storylet ID format.
    fn validate_followup_id(&self, id: &str) -> Vec<StoryletValidationError> {
        self.validate_id(id)
            .into_iter()
            .map(|err| {
                // Convert to InvalidFollowUpId variant
                if let StoryletValidationError::MissingId = err {
                    StoryletValidationError::InvalidFollowUpId {
                        id: id.to_string(),
                        reason: "missing or empty".to_string(),
                    }
                } else if let StoryletValidationError::InvalidIdFormat {
                    id: err_id,
                    reason,
                } = err
                {
                    StoryletValidationError::InvalidFollowUpId {
                        id: err_id,
                        reason,
                    }
                } else {
                    err
                }
            })
            .collect()
    }

    /// Validate a memory entry.
    fn validate_memory_entry(&self, mem: &MemoryEntry) -> Vec<StoryletValidationError> {
        let mut errors = Vec::new();

        // Validate all tags
        for tag in &mem.tags {
            if !self.allowed_memory_tags.contains(tag) {
                errors.push(StoryletValidationError::UnknownMemoryTag {
                    tag: tag.clone(),
                });
            }
        }

        // Validate intensity is reasonable (0–10)
        if mem.intensity > 10 {
            errors.push(StoryletValidationError::InvalidMemoryEntry {
                reason: format!("intensity {} out of range 0–10", mem.intensity),
            });
        }

        errors
    }
}

impl Default for StoryletValidator {
    fn default() -> Self {
        default_storylet_validator()
    }
}

/// Factory function providing a sensible default validator for the SYN game.
///
/// Includes:
/// - Standard SYN stats: mood, stress, wealth, health, karma, reputation
/// - Common personality traits: impulsivity, empathy, ambition, resilience, trust
/// - Five-axis relationships: affection, trust, attraction, familiarity, resentment
/// - Global flags: first_love_experienced, has_ever_worked, experienced_trauma, etc.
/// - Memory tags: romance, conflict, milestone, trauma, achievement, etc.
///
/// These defaults can be extended or modified by chaining builder methods.
pub fn default_storylet_validator() -> StoryletValidator {
    StoryletValidator::new()
        // Default stats with ranges (typical SYN ranges)
        .with_stat_range("mood", -10.0, 10.0)
        .with_stat_range("stress", 0.0, 100.0)
        .with_stat_range("wealth", -50.0, 500.0)
        .with_stat_range("health", 0.0, 100.0)
        .with_stat_range("karma", -100.0, 100.0)
        .with_stat_range("reputation", -50.0, 50.0)
        .with_stat_range("energy", 0.0, 100.0)
        .with_stat_range("hunger", 0.0, 100.0)
        // Default personality traits (per GDD §7.4: all traits are 0-100)
        .with_trait_range("stability", 0.0, 100.0)
        .with_trait_range("confidence", 0.0, 100.0)
        .with_trait_range("sociability", 0.0, 100.0)
        .with_trait_range("empathy", 0.0, 100.0)
        .with_trait_range("impulsivity", 0.0, 100.0)
        .with_trait_range("ambition", 0.0, 100.0)
        .with_trait_range("charm", 0.0, 100.0)
        // Relationship axes (always -10..=10)
        .with_axes(vec![
            "affection",
            "trust",
            "attraction",
            "familiarity",
            "resentment",
        ])
        // Common global flags
        .with_flags(vec![
            "first_love_experienced",
            "has_ever_worked",
            "experienced_trauma",
            "completed_education",
            "married",
            "divorced",
            "had_child",
            "experienced_loss",
        ])
        // Common memory tags
        .with_memory_tags(vec![
            "romance",
            "conflict",
            "milestone",
            "trauma",
            "achievement",
            "betrayal",
            "reconciliation",
            "goodbye",
            "first_meeting",
            "intimacy",
        ])
}

/// Convenience function to validate multiple storylets at once.
///
/// Returns `Ok(())` if all storylets are valid.
/// Returns `Err(vec)` containing tuples of (storylet_id, errors) for any invalid storylets.
/// This is useful for batch validation during import or compilation phases.
///
/// # Example
/// ```
/// # use syn_storylets::validation::validate_storylets;
/// # use syn_storylets::{StoryletDef, StoryletId, StoryDomain, LifeStage};
/// let validator = syn_storylets::validation::default_storylet_validator();
/// let storylets = vec![
///     StoryletDef::new(
///         StoryletId::new("test.story"),
///         "Test Story".to_string(),
///         StoryDomain::Romance,
///         LifeStage::Adult,
///     ),
/// ];
/// match validate_storylets(&validator, &storylets) {
///     Ok(()) => println!("All storylets valid!"),
///     Err(failures) => {
///         for (id, errors) in failures {
///             eprintln!("Storylet '{}' has {} validation errors", id.0, errors.len());
///         }
///     }
/// }
/// ```
pub fn validate_storylets(
    validator: &StoryletValidator,
    storylets: &[StoryletDef],
) -> Result<(), Vec<(StoryletId, Vec<StoryletValidationError>)>> {
    let mut failures = Vec::new();

    for storylet in storylets {
        if let Err(errors) = validator.validate_storylet(storylet) {
            failures.push((storylet.id.clone(), errors));
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GlobalFlags, MemoryPrerequisites, RelationshipPrerequisites,
        RelationshipThreshold, StatThresholds, StoryDomain, StoryletId, Tag, LifeStage, FollowUpStorylet,
    };

    #[test]
    fn test_id_validation_valid() {
        let validator = default_storylet_validator();
        let errors = validator.validate_id("story.romance.first_date");
        assert!(errors.is_empty(), "Valid ID should not produce errors");
    }

    #[test]
    fn test_id_validation_uppercase() {
        let validator = default_storylet_validator();
        let errors = validator.validate_id("Story.Romance.FirstDate");
        assert!(
            !errors.is_empty(),
            "Uppercase ID should produce validation error"
        );
    }

    #[test]
    fn test_id_validation_starts_with_dot() {
        let validator = default_storylet_validator();
        let errors = validator.validate_id(".invalid");
        assert!(
            !errors.is_empty(),
            "ID starting with dot should produce error"
        );
    }

    #[test]
    fn test_heat_out_of_range() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);
        storylet.heat = 15; // Out of range

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Heat > 10 should fail validation");

        if let Err(errors) = result {
            assert!(
                errors.iter().any(|e| matches!(e, StoryletValidationError::InvalidHeatRange { .. })),
                "Expected InvalidHeatRange error"
            );
        }
    }

    #[test]
    fn test_weight_must_be_positive() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);
        storylet.weight = -0.5; // Negative weight

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Negative weight should fail validation");
    }

    #[test]
    fn test_weight_zero_invalid() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);
        storylet.weight = 0.0; // Zero weight

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Zero weight should fail validation");
    }

    #[test]
    fn test_unknown_stat_in_prerequisites() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.prerequisites.stat_thresholds = Some(vec![StatThresholds {
            stat: "unknown_stat".to_string(),
            min: Some(-5.0),
            max: Some(5.0),
        }]);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Unknown stat should fail validation");

        if let Err(errors) = result {
            assert!(
                errors.iter().any(|e| matches!(e, StoryletValidationError::UnknownStatName { .. })),
                "Expected UnknownStatName error"
            );
        }
    }

    #[test]
    fn test_relationship_threshold_out_of_range() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.prerequisites.relationship_prerequisites = Some(vec![RelationshipPrerequisites {
            from_role: "protagonist".to_string(),
            to_role: "love_interest".to_string(),
            thresholds: vec![RelationshipThreshold {
                axis: "affection".to_string(),
                min: Some(20.0), // Out of range
                max: None,
            }],
        }]);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Relationship threshold > 10 should fail");
    }

    #[test]
    fn test_valid_storylet() {
        let validator = default_storylet_validator();
        let storylet =
            StoryletDef::new(StoryletId::new("test.valid"), "Valid Storylet".to_string(), StoryDomain::Romance, LifeStage::Adult);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_ok(), "Valid storylet should pass validation");
    }

    #[test]
    fn test_complex_valid_storylet() {
        let validator = default_storylet_validator();
        let mut storylet = StoryletDef::new(
            StoryletId::new("romance.first_date"),
            "First Date".to_string(),
            StoryDomain::Romance,
            LifeStage::YoungAdult,
        );

        storylet.tags = vec![Tag::new("romance"), Tag::new("milestone")];
        storylet.heat = 4;
        storylet.weight = 0.8;

        storylet.prerequisites.stat_thresholds = Some(vec![StatThresholds {
            stat: "mood".to_string(),
            min: Some(-5.0),
            max: Some(10.0),
        }]);

        storylet.prerequisites.relationship_prerequisites = Some(vec![RelationshipPrerequisites {
            from_role: "protagonist".to_string(),
            to_role: "love_interest".to_string(),
            thresholds: vec![RelationshipThreshold {
                axis: "affection".to_string(),
                min: Some(3.0),
                max: None,
            }],
        }]);

        storylet.cooldowns = crate::Cooldowns {
            global_cooldown_ticks: Some(240),
            per_actor_cooldown_ticks: Some(120),
            per_relationship_cooldown_ticks: Some(480),
            per_district_cooldown_ticks: None,
        };

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_ok(), "Complex but valid storylet should pass validation");
    }

    #[test]
    fn test_multiple_validation_errors() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("BAD_ID"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.heat = 25; // Invalid heat
        storylet.weight = 0.0; // Invalid weight

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Should have multiple errors");

        if let Err(errors) = result {
            assert!(errors.len() >= 3, "Should have at least 3 errors (ID, heat, weight)");
        }
    }

    #[test]
    fn test_stat_threshold_min_greater_than_max() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.prerequisites.stat_thresholds = Some(vec![StatThresholds {
            stat: "mood".to_string(),
            min: Some(5.0),
            max: Some(-5.0), // min > max
        }]);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err());

        if let Err(errors) = result {
            assert!(
                errors.iter().any(|e| matches!(e, StoryletValidationError::InvalidStatThreshold { .. })),
                "Expected InvalidStatThreshold error"
            );
        }
    }

    #[test]
    fn test_unknown_memory_tag() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.prerequisites.memory_prerequisites = Some(MemoryPrerequisites {
            must_have_tags: vec!["unknown_memory_tag".to_string()],
            must_not_have_tags: vec![],
        });

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err());

        if let Err(errors) = result {
            assert!(
                errors.iter().any(|e| matches!(e, StoryletValidationError::UnknownMemoryTag { .. })),
                "Expected UnknownMemoryTag error"
            );
        }
    }

    #[test]
    fn test_unknown_global_flag() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.prerequisites.global_flags = Some(GlobalFlags {
            must_be_set: vec!["unknown_flag".to_string()],
            must_be_unset: vec![],
        });

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err());

        if let Err(errors) = result {
            assert!(
                errors.iter().any(|e| matches!(e, StoryletValidationError::UnknownGlobalFlag { .. })),
                "Expected UnknownGlobalFlag error"
            );
        }
    }

    #[test]
    fn test_invalid_followup_id() {
        let validator = default_storylet_validator();
        let mut storylet =
            StoryletDef::new(StoryletId::new("test"), "Test".to_string(), StoryDomain::Romance, LifeStage::Adult);

        storylet.outcomes.follow_ups = Some(vec![FollowUpStorylet {
            storylet_id: "BadFollowUpID".to_string(), // Invalid format
            delay_ticks: 0,
            conditional_on_flag: None,
        }]);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err());

        if let Err(errors) = result {
            assert!(
                errors.iter().any(|e| matches!(e, StoryletValidationError::InvalidFollowUpId { .. })),
                "Expected InvalidFollowUpId error"
            );
        }
    }

    #[test]
    fn test_batch_validation_all_valid() {
        let validator = default_storylet_validator();
        let storylets = vec![
            StoryletDef::new(
                StoryletId::new("story.one"),
                "Story One".to_string(),
                StoryDomain::Romance,
                LifeStage::Adult,
            ),
            StoryletDef::new(
                StoryletId::new("story.two"),
                "Story Two".to_string(),
                StoryDomain::Career,
                LifeStage::YoungAdult,
            ),
        ];

        let result = validate_storylets(&validator, &storylets);
        assert!(result.is_ok(), "All valid storylets should pass batch validation");
    }

    #[test]
    fn test_batch_validation_one_invalid() {
        let validator = default_storylet_validator();

        let valid_storylet = StoryletDef::new(
            StoryletId::new("story.valid"),
            "Valid Story".to_string(),
            StoryDomain::Romance,
            LifeStage::Adult,
        );

        let mut invalid_storylet = StoryletDef::new(
            StoryletId::new("story.invalid"),
            "Invalid Story".to_string(),
            StoryDomain::Career,
            LifeStage::Teen,
        );
        invalid_storylet.weight = -1.0; // Invalid

        let storylets = vec![valid_storylet, invalid_storylet];

        let result = validate_storylets(&validator, &storylets);
        assert!(result.is_err(), "Should catch the invalid storylet");

        if let Err(failures) = result {
            assert_eq!(failures.len(), 1, "Should have exactly 1 failure");
            assert_eq!(failures[0].0.0, "story.invalid", "Should report the invalid storylet");
        }
    }

    #[test]
    fn test_batch_validation_multiple_invalid() {
        let validator = default_storylet_validator();

        let mut invalid1 = StoryletDef::new(
            StoryletId::new("BAD_ID_1"),
            "Bad 1".to_string(),
            StoryDomain::Romance,
            LifeStage::Adult,
        );
        invalid1.heat = 50; // Out of range

        let invalid2 = StoryletDef::new(
            StoryletId::new("bad_id_2"),
            "".to_string(), // Empty name
            StoryDomain::Career,
            LifeStage::Teen,
        );

        let valid = StoryletDef::new(
            StoryletId::new("story.valid"),
            "Valid".to_string(),
            StoryDomain::Family,
            LifeStage::Adult,
        );

        let storylets = vec![invalid1, valid, invalid2];

        let result = validate_storylets(&validator, &storylets);
        assert!(result.is_err());

        if let Err(failures) = result {
            assert_eq!(failures.len(), 2, "Should have exactly 2 failures");
            let failed_ids: Vec<_> = failures.iter().map(|(id, _)| id.0.clone()).collect();
            assert!(failed_ids.contains(&"BAD_ID_1".to_string()));
            assert!(failed_ids.contains(&"bad_id_2".to_string()));
        }
    }

    #[test]
    fn test_batch_validation_empty_list() {
        let validator = default_storylet_validator();
        let storylets: Vec<StoryletDef> = vec![];

        let result = validate_storylets(&validator, &storylets);
        assert!(
            result.is_ok(),
            "Empty storylet list should be valid (no errors to report)"
        );
    }

    #[test]
    fn test_validator_builder_extension() {
        // Test that validator can be extended with custom stats
        let validator = default_storylet_validator()
            .with_stat_range("custom_stat", 0.0, 50.0)
            .with_memory_tag("custom_memory_tag");

        let mut storylet = StoryletDef::new(
            StoryletId::new("test.custom"),
            "Test Custom".to_string(),
            StoryDomain::Romance,
            LifeStage::Adult,
        );

        storylet.prerequisites.stat_thresholds = Some(vec![StatThresholds {
            stat: "custom_stat".to_string(),
            min: Some(10.0),
            max: Some(40.0),
        }]);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_ok(), "Custom stat should be recognized by extended validator");
    }

    #[test]
    fn test_clearly_invalid_storylet() {
        let validator = default_storylet_validator();

        let mut storylet = StoryletDef::new(
            StoryletId::new("BadID@#$"),
            "Clear Test".to_string(),
            StoryDomain::Conflict,
            LifeStage::Child,
        );

        storylet.heat = 100; // Way out of range
        storylet.weight = 0.0; // Must be positive
        storylet.cooldowns.global_cooldown_ticks = Some(1_000_000); // Too large

        storylet.prerequisites.stat_thresholds = Some(vec![StatThresholds {
            stat: "nonexistent_stat".to_string(),
            min: Some(999.0),
            max: Some(-999.0), // min > max
        }]);

        let result = validator.validate_storylet(&storylet);
        assert!(result.is_err(), "Should have multiple errors");

        if let Err(errors) = result {
            assert!(errors.len() >= 5, "Should have at least 5 different errors");
            // Verify we have ID format error
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, StoryletValidationError::InvalidIdFormat { .. })),
                "Should report invalid ID format"
            );
            // Verify we have heat error
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, StoryletValidationError::InvalidHeatRange { .. })),
                "Should report invalid heat"
            );
            // Verify we have weight error
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, StoryletValidationError::InvalidWeight { .. })),
                "Should report invalid weight"
            );
            // Verify we have unknown stat error
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, StoryletValidationError::UnknownStatName { .. })),
                "Should report unknown stat"
            );
        }
    }
}
