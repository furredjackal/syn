//! Skill and activity progression system.
//!
//! Per GDD §10: Skills are modular data tables that map actions → XP vectors.
//! Each skill has a progression curve based on Intelligence and Mood.
//! Failure states still yield partial XP to encourage experimentation.
//! Certain skills unlock exclusive storylets.

use crate::rng::DeterministicRng;
use crate::{StatKind, Stats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a skill.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(pub String);

impl SkillId {
    /// Create a new skill ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        SkillId(id.into())
    }
}

impl From<&str> for SkillId {
    fn from(s: &str) -> Self {
        SkillId(s.to_string())
    }
}

impl From<String> for SkillId {
    fn from(s: String) -> Self {
        SkillId(s)
    }
}

/// Skill category for grouping and UI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    /// Physical activities (sports, fitness, combat)
    Physical,
    /// Mental/intellectual skills (programming, science, logic)
    Mental,
    /// Creative arts (music, art, writing)
    Creative,
    /// Social skills (persuasion, leadership, charm)
    Social,
    /// Practical/trade skills (cooking, mechanics, crafting)
    Practical,
    /// Technical/specialized (hacking, medicine, law)
    Technical,
}

impl Default for SkillCategory {
    fn default() -> Self {
        Self::Practical
    }
}

/// Skill level tier for milestone-based progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SkillTier {
    /// 0-99 XP: Just starting out
    Novice,
    /// 100-299 XP: Basic competence
    Beginner,
    /// 300-599 XP: Solid foundation
    Intermediate,
    /// 600-999 XP: Notable skill
    Advanced,
    /// 1000-1499 XP: Professional level
    Expert,
    /// 1500+ XP: Mastery achieved
    Master,
}

impl SkillTier {
    /// Get tier from XP value.
    pub fn from_xp(xp: u32) -> Self {
        match xp {
            0..=99 => Self::Novice,
            100..=299 => Self::Beginner,
            300..=599 => Self::Intermediate,
            600..=999 => Self::Advanced,
            1000..=1499 => Self::Expert,
            _ => Self::Master,
        }
    }

    /// Get minimum XP for this tier.
    pub fn min_xp(&self) -> u32 {
        match self {
            Self::Novice => 0,
            Self::Beginner => 100,
            Self::Intermediate => 300,
            Self::Advanced => 600,
            Self::Expert => 1000,
            Self::Master => 1500,
        }
    }

    /// Get tier as numeric level (0-5).
    pub fn as_level(&self) -> u8 {
        match self {
            Self::Novice => 0,
            Self::Beginner => 1,
            Self::Intermediate => 2,
            Self::Advanced => 3,
            Self::Expert => 4,
            Self::Master => 5,
        }
    }

    /// Get tier from numeric level.
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => Self::Novice,
            1 => Self::Beginner,
            2 => Self::Intermediate,
            3 => Self::Advanced,
            4 => Self::Expert,
            _ => Self::Master,
        }
    }
}

impl Default for SkillTier {
    fn default() -> Self {
        Self::Novice
    }
}

/// Definition of a skill (immutable data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Unique skill identifier
    pub id: SkillId,
    /// Display name
    pub name: String,
    /// Short description
    pub description: String,
    /// Skill category
    pub category: SkillCategory,
    /// Base XP rate multiplier (default 1.0)
    pub xp_rate: f32,
    /// Stats that affect XP gain (higher stat = faster learning)
    pub stat_affinities: Vec<StatKind>,
    /// Storylet IDs unlocked by this skill at various tiers
    pub unlock_storylets: HashMap<SkillTier, Vec<String>>,
    /// Minimum life stage to learn this skill
    pub min_life_stage: Option<String>,
    /// Whether this skill can decay from disuse
    pub can_decay: bool,
    /// Ticks of inactivity before decay starts
    pub decay_threshold_ticks: u32,
    /// XP lost per tick during decay
    pub decay_rate: f32,
}

impl SkillDefinition {
    /// Create a basic skill definition.
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: SkillCategory) -> Self {
        Self {
            id: SkillId::new(id),
            name: name.into(),
            description: String::new(),
            category,
            xp_rate: 1.0,
            stat_affinities: Vec::new(),
            unlock_storylets: HashMap::new(),
            min_life_stage: None,
            can_decay: false,
            decay_threshold_ticks: 720, // 30 days
            decay_rate: 0.5,
        }
    }

    /// Builder: set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Builder: set XP rate.
    pub fn with_xp_rate(mut self, rate: f32) -> Self {
        self.xp_rate = rate;
        self
    }

    /// Builder: add stat affinity.
    pub fn with_affinity(mut self, stat: StatKind) -> Self {
        self.stat_affinities.push(stat);
        self
    }

    /// Builder: add unlock storylet at tier.
    pub fn with_unlock(mut self, tier: SkillTier, storylet_id: impl Into<String>) -> Self {
        self.unlock_storylets
            .entry(tier)
            .or_default()
            .push(storylet_id.into());
        self
    }

    /// Builder: enable decay.
    pub fn with_decay(mut self, threshold_ticks: u32, rate: f32) -> Self {
        self.can_decay = true;
        self.decay_threshold_ticks = threshold_ticks;
        self.decay_rate = rate;
        self
    }

    /// Calculate XP modifier based on player stats.
    pub fn calculate_xp_modifier(&self, stats: &Stats) -> f32 {
        let mut modifier = self.xp_rate;

        // Mood affects all learning
        let mood_bonus = (stats.mood / 10.0).clamp(-0.3, 0.3);
        modifier += mood_bonus;

        // Stat affinities provide bonuses
        for stat_kind in &self.stat_affinities {
            let stat_value = stats.get(*stat_kind);
            // Normalized: 50 = baseline, higher = bonus
            let affinity_bonus = (stat_value - 50.0) / 100.0;
            modifier += affinity_bonus * 0.5;
        }

        modifier.max(0.1) // Minimum 10% XP gain
    }
}

impl Default for SkillDefinition {
    fn default() -> Self {
        Self::new("unknown", "Unknown Skill", SkillCategory::Practical)
    }
}

/// Player's progress in a specific skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillProgress {
    /// Current XP (raw accumulated)
    pub xp: u32,
    /// Total XP ever earned (doesn't decrease with decay)
    pub total_xp_earned: u32,
    /// Last tick when this skill was practiced
    pub last_practiced_tick: u64,
    /// Number of times skill was used
    pub practice_count: u32,
    /// Number of failures while practicing
    pub failure_count: u32,
    /// Whether skill was ever at Master tier
    pub achieved_mastery: bool,
}

impl SkillProgress {
    /// Create a new skill progress with zero XP.
    pub fn new() -> Self {
        Self {
            xp: 0,
            total_xp_earned: 0,
            last_practiced_tick: 0,
            practice_count: 0,
            failure_count: 0,
            achieved_mastery: false,
        }
    }

    /// Get current tier.
    pub fn tier(&self) -> SkillTier {
        SkillTier::from_xp(self.xp)
    }

    /// Get current tier level (0-5).
    pub fn level(&self) -> u8 {
        self.tier().as_level()
    }

    /// Get progress percentage toward next tier.
    pub fn progress_to_next_tier(&self) -> f32 {
        let current_tier = self.tier();
        if current_tier == SkillTier::Master {
            return 1.0;
        }

        let next_tier = SkillTier::from_level(current_tier.as_level() + 1);
        let current_min = current_tier.min_xp();
        let next_min = next_tier.min_xp();

        let range = next_min - current_min;
        let progress = self.xp - current_min;

        (progress as f32 / range as f32).clamp(0.0, 1.0)
    }

    /// Add XP and return whether a new tier was reached.
    pub fn add_xp(&mut self, amount: u32, current_tick: u64) -> Option<SkillTier> {
        let old_tier = self.tier();
        self.xp = self.xp.saturating_add(amount);
        self.total_xp_earned = self.total_xp_earned.saturating_add(amount);
        self.last_practiced_tick = current_tick;
        self.practice_count += 1;

        let new_tier = self.tier();
        if new_tier == SkillTier::Master {
            self.achieved_mastery = true;
        }

        if new_tier > old_tier {
            Some(new_tier)
        } else {
            None
        }
    }

    /// Add XP for a failed attempt (partial XP).
    pub fn add_failure_xp(&mut self, base_amount: u32, current_tick: u64) -> Option<SkillTier> {
        self.failure_count += 1;
        // Failures give 25% XP
        let failure_xp = (base_amount as f32 * 0.25).ceil() as u32;
        self.add_xp(failure_xp.max(1), current_tick)
    }

    /// Apply decay if inactive too long.
    pub fn apply_decay(&mut self, current_tick: u64, definition: &SkillDefinition) -> bool {
        if !definition.can_decay {
            return false;
        }

        let inactive_ticks = current_tick.saturating_sub(self.last_practiced_tick);
        if inactive_ticks < definition.decay_threshold_ticks as u64 {
            return false;
        }

        // Calculate decay amount
        let decay_periods = (inactive_ticks - definition.decay_threshold_ticks as u64) / 24; // Per day
        let decay_amount = (decay_periods as f32 * definition.decay_rate) as u32;

        if decay_amount > 0 {
            // Don't decay below Novice tier minimum
            let min_xp = SkillTier::Novice.min_xp();
            self.xp = self.xp.saturating_sub(decay_amount).max(min_xp);
            true
        } else {
            false
        }
    }
}

impl Default for SkillProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Activity definition - actions that grant skill XP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Unique activity identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Skills trained by this activity and XP amounts
    pub skill_xp: HashMap<SkillId, u32>,
    /// Base duration in ticks
    pub duration_ticks: u32,
    /// Stat requirements to attempt
    pub stat_requirements: HashMap<StatKind, f32>,
    /// Minimum skill tier required
    pub skill_requirements: HashMap<SkillId, SkillTier>,
    /// Success probability modifier (base 0.7 = 70%)
    pub base_success_rate: f32,
    /// Stats that affect success rate
    pub success_stat_modifiers: Vec<StatKind>,
}

impl Activity {
    /// Create a new activity with default values.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            skill_xp: HashMap::new(),
            duration_ticks: 1,
            stat_requirements: HashMap::new(),
            skill_requirements: HashMap::new(),
            base_success_rate: 0.7,
            success_stat_modifiers: Vec::new(),
        }
    }

    /// Builder: grant XP to a skill.
    pub fn grants_xp(mut self, skill: impl Into<SkillId>, amount: u32) -> Self {
        self.skill_xp.insert(skill.into(), amount);
        self
    }

    /// Builder: set duration.
    pub fn with_duration(mut self, ticks: u32) -> Self {
        self.duration_ticks = ticks;
        self
    }

    /// Builder: require stat minimum.
    pub fn requires_stat(mut self, stat: StatKind, min: f32) -> Self {
        self.stat_requirements.insert(stat, min);
        self
    }

    /// Builder: require skill tier.
    pub fn requires_skill(mut self, skill: impl Into<SkillId>, tier: SkillTier) -> Self {
        self.skill_requirements.insert(skill.into(), tier);
        self
    }

    /// Check if player meets requirements.
    pub fn can_attempt(&self, stats: &Stats, skills: &SkillState) -> bool {
        // Check stat requirements
        for (stat_kind, min_value) in &self.stat_requirements {
            if stats.get(*stat_kind) < *min_value {
                return false;
            }
        }

        // Check skill requirements
        for (skill_id, min_tier) in &self.skill_requirements {
            let current_tier = skills.get_tier(skill_id);
            if current_tier < *min_tier {
                return false;
            }
        }

        true
    }

    /// Calculate success probability.
    pub fn calculate_success_rate(&self, stats: &Stats) -> f32 {
        let mut rate = self.base_success_rate;

        for stat_kind in &self.success_stat_modifiers {
            let stat_value = stats.get(*stat_kind);
            // 50 = baseline, each 10 points above/below = ±5%
            let modifier = (stat_value - 50.0) / 200.0;
            rate += modifier;
        }

        rate.clamp(0.05, 0.99)
    }

    /// Attempt the activity and return XP gains.
    pub fn attempt(
        &self,
        stats: &Stats,
        skill_state: &mut SkillState,
        skill_defs: &SkillRegistry,
        current_tick: u64,
        rng: &mut DeterministicRng,
    ) -> ActivityResult {
        let success_rate = self.calculate_success_rate(stats);
        let roll = rng.gen_f32();
        let succeeded = roll < success_rate;

        let mut xp_gains = Vec::new();
        let mut tier_ups = Vec::new();

        for (skill_id, base_xp) in &self.skill_xp {
            let definition = skill_defs.get(skill_id);
            let modifier = definition
                .map(|d| d.calculate_xp_modifier(stats))
                .unwrap_or(1.0);
            let modified_xp = (*base_xp as f32 * modifier).round() as u32;

            let progress = skill_state.get_or_create_mut(skill_id);
            let tier_change = if succeeded {
                progress.add_xp(modified_xp, current_tick)
            } else {
                progress.add_failure_xp(modified_xp, current_tick)
            };

            xp_gains.push((skill_id.clone(), modified_xp, succeeded));
            if let Some(new_tier) = tier_change {
                tier_ups.push((skill_id.clone(), new_tier));
            }
        }

        ActivityResult {
            activity_id: self.id.clone(),
            succeeded,
            xp_gains,
            tier_ups,
            roll,
            success_rate,
        }
    }
}

/// Result of attempting an activity.
#[derive(Debug, Clone)]
pub struct ActivityResult {
    /// Unique identifier of the activity attempted.
    pub activity_id: String,
    /// Whether the attempt succeeded.
    pub succeeded: bool,
    /// XP gains: (skill_id, xp_amount, was_success_xp).
    pub xp_gains: Vec<(SkillId, u32, bool)>,
    /// Skills that reached a new tier.
    pub tier_ups: Vec<(SkillId, SkillTier)>,
    /// The random roll used for success check.
    pub roll: f32,
    /// The calculated success rate.
    pub success_rate: f32,
}

/// Player's skill state - all skill progress.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillState {
    /// Progress per skill
    pub skills: HashMap<SkillId, SkillProgress>,
}

impl SkillState {
    /// Create a new empty skill state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get progress for a skill.
    pub fn get(&self, skill_id: &SkillId) -> Option<&SkillProgress> {
        self.skills.get(skill_id)
    }

    /// Get mutable progress for a skill.
    pub fn get_mut(&mut self, skill_id: &SkillId) -> Option<&mut SkillProgress> {
        self.skills.get_mut(skill_id)
    }

    /// Get or create progress for a skill.
    pub fn get_or_create_mut(&mut self, skill_id: &SkillId) -> &mut SkillProgress {
        self.skills
            .entry(skill_id.clone())
            .or_insert_with(SkillProgress::new)
    }

    /// Get tier for a skill (Novice if not started).
    pub fn get_tier(&self, skill_id: &SkillId) -> SkillTier {
        self.skills
            .get(skill_id)
            .map(|p| p.tier())
            .unwrap_or(SkillTier::Novice)
    }

    /// Get XP for a skill (0 if not started).
    pub fn get_xp(&self, skill_id: &SkillId) -> u32 {
        self.skills.get(skill_id).map(|p| p.xp).unwrap_or(0)
    }

    /// Get level (0-5) for a skill.
    pub fn get_level(&self, skill_id: &SkillId) -> u8 {
        self.get_tier(skill_id).as_level()
    }

    /// Check if player has reached a tier in a skill.
    pub fn has_tier(&self, skill_id: &SkillId, required_tier: SkillTier) -> bool {
        self.get_tier(skill_id) >= required_tier
    }

    /// Check if player has minimum XP in a skill.
    pub fn has_min_xp(&self, skill_id: &SkillId, min_xp: u32) -> bool {
        self.get_xp(skill_id) >= min_xp
    }

    /// List all skills with progress.
    pub fn list_skills(&self) -> Vec<&SkillId> {
        self.skills.keys().collect()
    }

    /// List skills above Novice tier.
    pub fn list_learned_skills(&self) -> Vec<&SkillId> {
        self.skills
            .iter()
            .filter(|(_, p)| p.tier() > SkillTier::Novice)
            .map(|(id, _)| id)
            .collect()
    }

    /// Get all unlocked storylets based on skill levels.
    pub fn get_unlocked_storylets(&self, skill_defs: &SkillRegistry) -> Vec<String> {
        let mut unlocked = Vec::new();

        for (skill_id, progress) in &self.skills {
            if let Some(def) = skill_defs.get(skill_id) {
                let current_tier = progress.tier();
                for (tier, storylets) in &def.unlock_storylets {
                    if current_tier >= *tier {
                        unlocked.extend(storylets.clone());
                    }
                }
            }
        }

        unlocked.sort();
        unlocked.dedup();
        unlocked
    }

    /// Apply decay to all skills.
    pub fn apply_decay(&mut self, current_tick: u64, skill_defs: &SkillRegistry) {
        for (skill_id, progress) in self.skills.iter_mut() {
            if let Some(def) = skill_defs.get(skill_id) {
                progress.apply_decay(current_tick, def);
            }
        }
    }
}

/// Registry of all skill definitions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillRegistry {
    /// All skill definitions
    pub skills: HashMap<SkillId, SkillDefinition>,
}

impl SkillRegistry {
    /// Create a new empty skill registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create registry with default skills.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register a skill definition.
    pub fn register(&mut self, definition: SkillDefinition) {
        self.skills.insert(definition.id.clone(), definition);
    }

    /// Get skill definition by ID.
    pub fn get(&self, id: &SkillId) -> Option<&SkillDefinition> {
        self.skills.get(id)
    }

    /// List all skill IDs.
    pub fn list(&self) -> Vec<&SkillId> {
        self.skills.keys().collect()
    }

    /// List skills by category.
    pub fn list_by_category(&self, category: SkillCategory) -> Vec<&SkillDefinition> {
        self.skills
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Register default skills for the game.
    pub fn register_defaults(&mut self) {
        // === Physical Skills ===
        self.register(
            SkillDefinition::new("fitness", "Fitness", SkillCategory::Physical)
                .with_description("Overall physical conditioning and stamina")
                .with_affinity(StatKind::Health)
                .with_xp_rate(1.0)
                .with_unlock(SkillTier::Intermediate, "career.athlete")
                .with_unlock(SkillTier::Expert, "career.personal_trainer"),
        );

        self.register(
            SkillDefinition::new("sports", "Sports", SkillCategory::Physical)
                .with_description("Competitive athletic abilities")
                .with_affinity(StatKind::Health)
                .with_affinity(StatKind::Charisma)
                .with_xp_rate(0.9),
        );

        self.register(
            SkillDefinition::new("martial_arts", "Martial Arts", SkillCategory::Physical)
                .with_description("Combat and self-defense techniques")
                .with_affinity(StatKind::Health)
                .with_xp_rate(0.8)
                .with_decay(1440, 0.3), // Decays after 60 days
        );

        // === Mental Skills ===
        self.register(
            SkillDefinition::new("programming", "Programming", SkillCategory::Mental)
                .with_description("Software development and coding")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(1.1)
                .with_unlock(SkillTier::Beginner, "career.junior_dev")
                .with_unlock(SkillTier::Advanced, "career.senior_dev")
                .with_unlock(SkillTier::Master, "career.tech_lead"),
        );

        self.register(
            SkillDefinition::new("science", "Science", SkillCategory::Mental)
                .with_description("Scientific knowledge and research methods")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(1.0)
                .with_unlock(SkillTier::Advanced, "career.researcher"),
        );

        self.register(
            SkillDefinition::new("logic", "Logic & Strategy", SkillCategory::Mental)
                .with_description("Problem-solving and strategic thinking")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(0.9),
        );

        // === Creative Skills ===
        self.register(
            SkillDefinition::new("art", "Visual Art", SkillCategory::Creative)
                .with_description("Drawing, painting, and visual design")
                .with_xp_rate(1.25)
                .with_unlock(SkillTier::Intermediate, "gallery.invite")
                .with_unlock(SkillTier::Expert, "career.artist"),
        );

        self.register(
            SkillDefinition::new("music", "Music", SkillCategory::Creative)
                .with_description("Musical performance and composition")
                .with_xp_rate(1.15)
                .with_unlock(SkillTier::Advanced, "career.musician"),
        );

        self.register(
            SkillDefinition::new("writing", "Writing", SkillCategory::Creative)
                .with_description("Creative and technical writing")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(1.0)
                .with_unlock(SkillTier::Intermediate, "career.journalist")
                .with_unlock(SkillTier::Expert, "career.author"),
        );

        // === Social Skills ===
        self.register(
            SkillDefinition::new("persuasion", "Persuasion", SkillCategory::Social)
                .with_description("Convincing and influencing others")
                .with_affinity(StatKind::Charisma)
                .with_xp_rate(1.0)
                .with_unlock(SkillTier::Advanced, "career.sales_exec"),
        );

        self.register(
            SkillDefinition::new("leadership", "Leadership", SkillCategory::Social)
                .with_description("Managing and inspiring teams")
                .with_affinity(StatKind::Charisma)
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(0.85)
                .with_unlock(SkillTier::Expert, "career.manager"),
        );

        self.register(
            SkillDefinition::new("networking", "Networking", SkillCategory::Social)
                .with_description("Building professional connections")
                .with_affinity(StatKind::Charisma)
                .with_xp_rate(1.0),
        );

        // === Practical Skills ===
        self.register(
            SkillDefinition::new("cooking", "Cooking", SkillCategory::Practical)
                .with_description("Food preparation and culinary arts")
                .with_xp_rate(1.2)
                .with_unlock(SkillTier::Advanced, "career.chef"),
        );

        self.register(
            SkillDefinition::new("mechanics", "Mechanics", SkillCategory::Practical)
                .with_description("Repairing and maintaining machines")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(1.0)
                .with_unlock(SkillTier::Intermediate, "career.mechanic"),
        );

        self.register(
            SkillDefinition::new("gardening", "Gardening", SkillCategory::Practical)
                .with_description("Growing plants and landscaping")
                .with_xp_rate(1.1),
        );

        // === Technical Skills ===
        self.register(
            SkillDefinition::new("hacking", "Hacking", SkillCategory::Technical)
                .with_description("Computer security and exploitation")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(0.7)
                .with_decay(480, 0.5), // Decays faster - 20 days
        );

        self.register(
            SkillDefinition::new("medicine", "Medicine", SkillCategory::Technical)
                .with_description("Medical knowledge and treatment")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(0.8)
                .with_unlock(SkillTier::Expert, "career.doctor"),
        );

        self.register(
            SkillDefinition::new("law", "Law", SkillCategory::Technical)
                .with_description("Legal knowledge and procedures")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(0.75)
                .with_unlock(SkillTier::Expert, "career.lawyer"),
        );

        self.register(
            SkillDefinition::new("finance", "Finance", SkillCategory::Technical)
                .with_description("Investment and money management")
                .with_affinity(StatKind::Intelligence)
                .with_xp_rate(0.9)
                .with_unlock(SkillTier::Advanced, "career.financial_advisor"),
        );
    }
}

/// Prerequisite for skill-gated storylets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPrerequisite {
    /// Skill that must be learned
    pub skill_id: SkillId,
    /// Minimum tier required
    pub min_tier: Option<SkillTier>,
    /// Minimum XP required (alternative to tier)
    pub min_xp: Option<u32>,
    /// Maximum tier allowed (for beginner-only content)
    pub max_tier: Option<SkillTier>,
}

impl SkillPrerequisite {
    /// Create a tier-based prerequisite.
    pub fn tier(skill: impl Into<SkillId>, min: SkillTier) -> Self {
        Self {
            skill_id: skill.into(),
            min_tier: Some(min),
            min_xp: None,
            max_tier: None,
        }
    }

    /// Create an XP-based prerequisite.
    pub fn xp(skill: impl Into<SkillId>, min: u32) -> Self {
        Self {
            skill_id: skill.into(),
            min_tier: None,
            min_xp: Some(min),
            max_tier: None,
        }
    }

    /// Create a tier range prerequisite.
    pub fn tier_range(skill: impl Into<SkillId>, min: SkillTier, max: SkillTier) -> Self {
        Self {
            skill_id: skill.into(),
            min_tier: Some(min),
            min_xp: None,
            max_tier: Some(max),
        }
    }

    /// Check if skill state meets this prerequisite.
    pub fn is_met(&self, skill_state: &SkillState) -> bool {
        let tier = skill_state.get_tier(&self.skill_id);
        let xp = skill_state.get_xp(&self.skill_id);

        // Check minimum tier
        if let Some(min_tier) = &self.min_tier {
            if tier < *min_tier {
                return false;
            }
        }

        // Check minimum XP
        if let Some(min_xp) = self.min_xp {
            if xp < min_xp {
                return false;
            }
        }

        // Check maximum tier
        if let Some(max_tier) = &self.max_tier {
            if tier > *max_tier {
                return false;
            }
        }

        true
    }
}

/// Type alias for player skill state (used in WorldState).
pub type PlayerSkills = SkillState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tier_from_xp() {
        assert_eq!(SkillTier::from_xp(0), SkillTier::Novice);
        assert_eq!(SkillTier::from_xp(99), SkillTier::Novice);
        assert_eq!(SkillTier::from_xp(100), SkillTier::Beginner);
        assert_eq!(SkillTier::from_xp(300), SkillTier::Intermediate);
        assert_eq!(SkillTier::from_xp(600), SkillTier::Advanced);
        assert_eq!(SkillTier::from_xp(1000), SkillTier::Expert);
        assert_eq!(SkillTier::from_xp(1500), SkillTier::Master);
        assert_eq!(SkillTier::from_xp(9999), SkillTier::Master);
    }

    #[test]
    fn test_skill_progress_add_xp() {
        let mut progress = SkillProgress::new();
        assert_eq!(progress.tier(), SkillTier::Novice);

        // Add XP to reach Beginner
        let tier_change = progress.add_xp(100, 0);
        assert_eq!(tier_change, Some(SkillTier::Beginner));
        assert_eq!(progress.tier(), SkillTier::Beginner);
        assert_eq!(progress.xp, 100);
    }

    #[test]
    fn test_skill_progress_failure_xp() {
        let mut progress = SkillProgress::new();

        // Failure gives 25% XP
        progress.add_failure_xp(100, 0);
        assert_eq!(progress.xp, 25);
        assert_eq!(progress.failure_count, 1);
    }

    #[test]
    fn test_skill_definition_xp_modifier() {
        let def = SkillDefinition::new("test", "Test", SkillCategory::Mental)
            .with_xp_rate(1.0)
            .with_affinity(StatKind::Intelligence);

        // Default stats (50 intelligence, 0 mood)
        let stats = Stats::default();
        let modifier = def.calculate_xp_modifier(&stats);
        assert!((modifier - 1.0).abs() < 0.01);

        // High intelligence should boost
        let mut high_int = Stats::default();
        high_int.intelligence = 80.0;
        let modifier = def.calculate_xp_modifier(&high_int);
        assert!(modifier > 1.0);
    }

    #[test]
    fn test_skill_state_get_tier() {
        let mut state = SkillState::new();
        let skill_id = SkillId::new("programming");

        // No progress = Novice
        assert_eq!(state.get_tier(&skill_id), SkillTier::Novice);

        // Add some XP
        state.get_or_create_mut(&skill_id).add_xp(150, 0);
        assert_eq!(state.get_tier(&skill_id), SkillTier::Beginner);
    }

    #[test]
    fn test_skill_prerequisite() {
        let mut state = SkillState::new();
        let skill_id = SkillId::new("programming");

        let prereq = SkillPrerequisite::tier("programming", SkillTier::Beginner);

        // Not met initially
        assert!(!prereq.is_met(&state));

        // Add XP to meet requirement
        state.get_or_create_mut(&skill_id).add_xp(100, 0);
        assert!(prereq.is_met(&state));
    }

    #[test]
    fn test_skill_registry_defaults() {
        let registry = SkillRegistry::with_defaults();
        assert!(registry.get(&SkillId::new("programming")).is_some());
        assert!(registry.get(&SkillId::new("art")).is_some());
        assert!(registry.get(&SkillId::new("cooking")).is_some());
    }

    #[test]
    fn test_unlocked_storylets() {
        let registry = SkillRegistry::with_defaults();
        let mut state = SkillState::new();

        // No skills = no unlocks
        let unlocked = state.get_unlocked_storylets(&registry);
        assert!(unlocked.is_empty());

        // Advance programming to Advanced
        state.get_or_create_mut(&SkillId::new("programming")).xp = 700;

        let unlocked = state.get_unlocked_storylets(&registry);
        assert!(unlocked.contains(&"career.junior_dev".to_string()));
        assert!(unlocked.contains(&"career.senior_dev".to_string()));
        assert!(!unlocked.contains(&"career.tech_lead".to_string()));
    }

    #[test]
    fn test_activity_attempt() {
        let registry = SkillRegistry::with_defaults();
        let mut state = SkillState::new();
        let stats = Stats::default();
        let mut rng = DeterministicRng::new(42);

        let activity = Activity::new("study_programming", "Study Programming")
            .grants_xp("programming", 10)
            .with_duration(1);

        let result = activity.attempt(&stats, &mut state, &registry, 0, &mut rng);
        
        // Should have gained some XP
        assert!(state.get_xp(&SkillId::new("programming")) > 0);
        assert!(!result.xp_gains.is_empty());
    }

    #[test]
    fn test_skill_decay() {
        let def = SkillDefinition::new("test", "Test", SkillCategory::Mental)
            .with_decay(24, 1.0); // Decay after 24 ticks

        let mut progress = SkillProgress::new();
        progress.xp = 200;
        progress.last_practiced_tick = 0;

        // Decay after 50 ticks (26 inactive after threshold)
        let decayed = progress.apply_decay(50, &def);
        assert!(decayed);
        assert!(progress.xp < 200);
    }
}
