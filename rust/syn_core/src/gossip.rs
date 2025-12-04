//! Gossip and social spread system.
//!
//! Per GDD §8.6: Implements memory spread within clusters, reputation diffusion,
//! and rumor amplification mechanics. Information spreads through social networks
//! based on relationship strength, NPC traits, and event salience.

use crate::rng::DeterministicRng;
use crate::types::{AbstractNpc, NpcId, Relationship, WorldState};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A piece of gossip that can spread through social networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rumor {
    /// Unique rumor identifier
    pub id: String,
    /// The original event/memory that spawned this rumor
    pub source_event_id: String,
    /// Who the rumor is about (subject)
    pub subject_id: NpcId,
    /// Who originally witnessed/started the rumor
    pub originator_id: NpcId,
    /// Tick when the rumor was created
    pub created_tick: u64,
    /// Base salience (how interesting/memorable the rumor is, 0.0-1.0)
    pub salience: f32,
    /// Emotional valence (-1.0 negative to +1.0 positive)
    pub valence: f32,
    /// Tags categorizing the rumor content
    pub tags: Vec<String>,
    /// How much the rumor affects reputation when believed
    pub reputation_impact: f32,
    /// Whether this is scandalous (spreads faster, distorts more)
    pub is_scandalous: bool,
}

impl Rumor {
    /// Create a new rumor with default values.
    pub fn new(
        id: impl Into<String>,
        source_event_id: impl Into<String>,
        subject_id: NpcId,
        originator_id: NpcId,
        created_tick: u64,
    ) -> Self {
        Self {
            id: id.into(),
            source_event_id: source_event_id.into(),
            subject_id,
            originator_id,
            created_tick,
            salience: 0.5,
            valence: 0.0,
            tags: Vec::new(),
            reputation_impact: 0.0,
            is_scandalous: false,
        }
    }

    /// Builder: set salience
    pub fn with_salience(mut self, salience: f32) -> Self {
        self.salience = salience.clamp(0.0, 1.0);
        self
    }

    /// Builder: set emotional valence
    pub fn with_valence(mut self, valence: f32) -> Self {
        self.valence = valence.clamp(-1.0, 1.0);
        self
    }

    /// Builder: add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder: set reputation impact
    pub fn with_reputation_impact(mut self, impact: f32) -> Self {
        self.reputation_impact = impact;
        self
    }

    /// Builder: mark as scandalous
    pub fn scandalous(mut self) -> Self {
        self.is_scandalous = true;
        self.salience = (self.salience * 1.5).min(1.0);
        self
    }

    /// Check if rumor is still "fresh" (within decay window)
    pub fn is_fresh(&self, current_tick: u64, decay_ticks: u64) -> bool {
        current_tick.saturating_sub(self.created_tick) < decay_ticks
    }

    /// Calculate decay factor based on age (older = less impactful)
    pub fn decay_factor(&self, current_tick: u64, half_life_ticks: u64) -> f32 {
        let age = current_tick.saturating_sub(self.created_tick);
        let decay = 0.5_f32.powf(age as f32 / half_life_ticks as f32);
        decay.clamp(0.01, 1.0)
    }
}

/// Tracking of who knows what rumors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RumorKnowledge {
    /// The rumor being tracked
    pub rumor_id: String,
    /// Tick when this NPC learned the rumor
    pub learned_tick: u64,
    /// How much the NPC believes this rumor (0.0-1.0)
    pub belief: f32,
    /// How distorted the rumor has become (0.0 = original, 1.0 = heavily distorted)
    pub distortion: f32,
    /// Who told this NPC the rumor (None if originator)
    pub source_npc_id: Option<NpcId>,
}

impl RumorKnowledge {
    /// Create new rumor knowledge for an NPC.
    pub fn new(rumor_id: String, learned_tick: u64, source_npc_id: Option<NpcId>) -> Self {
        Self {
            rumor_id,
            learned_tick,
            belief: 1.0,
            distortion: 0.0,
            source_npc_id,
        }
    }

    /// Create knowledge for the originator (full belief, no distortion)
    pub fn originator(rumor_id: String, created_tick: u64) -> Self {
        Self {
            rumor_id,
            learned_tick: created_tick,
            belief: 1.0,
            distortion: 0.0,
            source_npc_id: None,
        }
    }
}

/// Result of a gossip spread attempt.
#[derive(Debug, Clone)]
pub struct SpreadResult {
    /// Rumor that was spread
    pub rumor_id: String,
    /// Who the rumor is about
    pub subject_id: NpcId,
    /// NPC who spread the rumor
    pub spreader_id: NpcId,
    /// NPC who received the rumor
    pub recipient_id: NpcId,
    /// Whether they accepted/believed it
    pub accepted: bool,
    /// Their belief level
    pub belief: f32,
    /// Distortion added
    pub distortion: f32,
    /// Whether they'll spread it further
    pub will_spread: bool,
}

/// Social cluster for grouping NPCs by relationship proximity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialCluster {
    /// Cluster identifier
    pub id: String,
    /// Core members of this cluster
    pub members: HashSet<NpcId>,
    /// Cluster cohesion (how tightly connected, 0.0-1.0)
    pub cohesion: f32,
    /// Average trust within the cluster
    pub internal_trust: f32,
}

impl SocialCluster {
    /// Create a new empty social cluster.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            members: HashSet::new(),
            cohesion: 0.5,
            internal_trust: 0.5,
        }
    }

    /// Add an NPC to this cluster.
    pub fn add_member(&mut self, npc_id: NpcId) {
        self.members.insert(npc_id);
    }

    /// Remove an NPC from this cluster.
    pub fn remove_member(&mut self, npc_id: NpcId) {
        self.members.remove(&npc_id);
    }

    /// Check if this cluster contains an NPC.
    pub fn contains(&self, npc_id: NpcId) -> bool {
        self.members.contains(&npc_id)
    }

    /// Get the number of members in this cluster.
    pub fn size(&self) -> usize {
        self.members.len()
    }
}

/// Main gossip system managing rumors and social spread.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GossipSystem {
    /// All active rumors
    pub rumors: HashMap<String, Rumor>,
    /// Who knows which rumors: npc_id -> rumor_id -> knowledge
    pub knowledge: HashMap<NpcId, HashMap<String, RumorKnowledge>>,
    /// Social clusters for spread simulation
    pub clusters: HashMap<String, SocialCluster>,
    /// Reputation modifiers from gossip: subject_id -> cumulative modifier
    pub reputation_effects: HashMap<NpcId, f32>,
    /// Configuration
    pub config: GossipConfig,
}

/// Configuration for gossip mechanics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// Base probability of sharing a rumor per tick
    pub base_spread_chance: f32,
    /// How much trust affects belief
    pub trust_belief_weight: f32,
    /// How much sociability affects spread likelihood
    pub sociability_spread_weight: f32,
    /// Distortion added per transmission
    pub distortion_per_hop: f32,
    /// Ticks before a rumor loses relevance
    pub rumor_decay_ticks: u64,
    /// Half-life for rumor impact decay
    pub rumor_half_life_ticks: u64,
    /// Minimum relationship strength to spread gossip
    pub min_familiarity_to_spread: f32,
    /// Scandalous rumor spread multiplier
    pub scandal_spread_multiplier: f32,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            base_spread_chance: 0.15,
            trust_belief_weight: 0.4,
            sociability_spread_weight: 0.3,
            distortion_per_hop: 0.1,
            rumor_decay_ticks: 168 * 4, // ~4 weeks
            rumor_half_life_ticks: 168,  // 1 week
            min_familiarity_to_spread: 2.0,
            scandal_spread_multiplier: 2.0,
        }
    }
}

impl GossipSystem {
    /// Create a new gossip system with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new rumor from an event.
    pub fn create_rumor(&mut self, rumor: Rumor) {
        let originator_id = rumor.originator_id;
        let rumor_id = rumor.id.clone();
        let created_tick = rumor.created_tick;

        self.rumors.insert(rumor_id.clone(), rumor);

        // Originator automatically knows the rumor
        self.knowledge
            .entry(originator_id)
            .or_default()
            .insert(
                rumor_id.clone(),
                RumorKnowledge::originator(rumor_id, created_tick),
            );
    }

    /// Check if an NPC knows a specific rumor.
    pub fn knows_rumor(&self, npc_id: NpcId, rumor_id: &str) -> bool {
        self.knowledge
            .get(&npc_id)
            .map(|k| k.contains_key(rumor_id))
            .unwrap_or(false)
    }

    /// Get an NPC's knowledge of a rumor.
    pub fn get_knowledge(&self, npc_id: NpcId, rumor_id: &str) -> Option<&RumorKnowledge> {
        self.knowledge.get(&npc_id)?.get(rumor_id)
    }

    /// Get all rumors an NPC knows.
    pub fn rumors_known_by(&self, npc_id: NpcId) -> Vec<&str> {
        self.knowledge
            .get(&npc_id)
            .map(|k| k.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all NPCs who know a specific rumor.
    pub fn who_knows_rumor(&self, rumor_id: &str) -> Vec<NpcId> {
        self.knowledge
            .iter()
            .filter(|(_, k)| k.contains_key(rumor_id))
            .map(|(npc_id, _)| *npc_id)
            .collect()
    }

    /// Calculate spread probability between two NPCs.
    pub fn calculate_spread_probability(
        &self,
        spreader: &AbstractNpc,
        receiver_id: NpcId,
        rumor: &Rumor,
        relationship: Option<&Relationship>,
    ) -> f32 {
        let mut prob = self.config.base_spread_chance;

        // Sociability increases spread likelihood
        let sociability_factor = spreader.traits.sociability / 50.0;
        prob *= 1.0 + (sociability_factor - 1.0) * self.config.sociability_spread_weight;

        // Relationship familiarity affects spread
        if let Some(rel) = relationship {
            if rel.familiarity < self.config.min_familiarity_to_spread {
                return 0.0; // Won't gossip with strangers
            }
            // Closer relationships more likely to share
            let familiarity_factor = (rel.familiarity / 5.0).clamp(0.5, 2.0);
            prob *= familiarity_factor;
        } else {
            return 0.0; // No relationship = no gossip
        }

        // Salience affects spread (more interesting = more likely to share)
        prob *= 0.5 + rumor.salience;

        // Scandalous rumors spread faster
        if rumor.is_scandalous {
            prob *= self.config.scandal_spread_multiplier;
        }

        // Don't spread rumors about yourself
        if rumor.subject_id == spreader.id {
            prob *= 0.1;
        }

        // Don't spread to the subject
        if receiver_id == rumor.subject_id {
            prob *= 0.2;
        }

        prob.clamp(0.0, 0.95)
    }

    /// Calculate how much an NPC believes a rumor.
    pub fn calculate_belief(
        &self,
        receiver: &AbstractNpc,
        _spreader_id: NpcId,
        rumor: &Rumor,
        relationship: Option<&Relationship>,
        incoming_distortion: f32,
    ) -> f32 {
        let mut belief = 0.5;

        // Trust in spreader affects belief
        if let Some(rel) = relationship {
            let trust_factor = (rel.trust + 10.0) / 20.0; // Normalize -10..10 to 0..1
            belief += (trust_factor - 0.5) * self.config.trust_belief_weight;
        }

        // Distortion reduces belief
        belief -= incoming_distortion * 0.3;

        // Empathy makes NPCs more likely to believe negative rumors less
        if rumor.valence < 0.0 {
            let empathy_factor = receiver.traits.empathy / 100.0;
            belief -= empathy_factor * 0.2 * rumor.valence.abs();
        }

        // Impulsive NPCs believe more readily
        let impulsivity_factor = receiver.traits.impulsivity / 100.0;
        belief += impulsivity_factor * 0.15;

        // Pre-existing relationship with subject affects belief
        // (If you like someone, you're less likely to believe bad things about them)
        // This would need world state access to check

        belief.clamp(0.1, 1.0)
    }

    /// Attempt to spread a rumor from one NPC to another.
    pub fn try_spread_rumor(
        &mut self,
        spreader: &AbstractNpc,
        receiver: &AbstractNpc,
        rumor_id: &str,
        relationship: Option<&Relationship>,
        current_tick: u64,
        rng: &mut DeterministicRng,
    ) -> Option<SpreadResult> {
        // Check if rumor exists and spreader knows it
        let rumor = self.rumors.get(rumor_id)?;
        let spreader_knowledge = self.get_knowledge(spreader.id, rumor_id)?;

        // Don't spread to someone who already knows
        if self.knows_rumor(receiver.id, rumor_id) {
            return None;
        }

        // Calculate spread probability
        let spread_prob = self.calculate_spread_probability(spreader, receiver.id, rumor, relationship);

        // Roll for spread
        if rng.gen_f32() > spread_prob {
            return None;
        }

        // Calculate belief and distortion
        let incoming_distortion = spreader_knowledge.distortion + self.config.distortion_per_hop;
        let belief = self.calculate_belief(receiver, spreader.id, rumor, relationship, incoming_distortion);

        // Determine if receiver will spread further
        let will_spread = belief > 0.4 && receiver.traits.sociability > 30.0;

        // Record the knowledge
        let mut knowledge = RumorKnowledge::new(rumor_id.to_string(), current_tick, Some(spreader.id));
        knowledge.belief = belief;
        knowledge.distortion = incoming_distortion.min(1.0);

        self.knowledge
            .entry(receiver.id)
            .or_default()
            .insert(rumor_id.to_string(), knowledge);

        // Apply reputation effect based on belief
        if belief > 0.3 {
            let rep_delta = rumor.reputation_impact * belief * rumor.decay_factor(current_tick, self.config.rumor_half_life_ticks);
            *self.reputation_effects.entry(rumor.subject_id).or_default() += rep_delta;
        }

        Some(SpreadResult {
            rumor_id: rumor_id.to_string(),
            subject_id: rumor.subject_id,
            spreader_id: spreader.id,
            recipient_id: receiver.id,
            accepted: true,
            belief,
            distortion: incoming_distortion,
            will_spread,
        })
    }

    /// Simulate gossip spread for one tick across all NPCs.
    pub fn tick_spread(
        &mut self,
        world: &WorldState,
        current_tick: u64,
        rng: &mut DeterministicRng,
    ) -> Vec<SpreadResult> {
        let mut results = Vec::new();

        // Collect NPCs who know rumors and might spread them
        let spreaders: Vec<(NpcId, Vec<String>)> = self
            .knowledge
            .iter()
            .map(|(npc_id, known)| (*npc_id, known.keys().cloned().collect()))
            .collect();

        for (spreader_id, known_rumors) in spreaders {
            let spreader = match world.npcs.get(&spreader_id) {
                Some(npc) => npc,
                None => continue,
            };

            // Skip if not sociable enough
            if spreader.traits.sociability < 20.0 {
                continue;
            }

            for rumor_id in known_rumors {
                let rumor = match self.rumors.get(&rumor_id) {
                    Some(r) => r,
                    None => continue,
                };

                // Skip stale rumors
                if !rumor.is_fresh(current_tick, self.config.rumor_decay_ticks) {
                    continue;
                }

                // Find potential recipients (NPCs the spreader knows)
                for (receiver_id, receiver) in &world.npcs {
                    if *receiver_id == spreader_id {
                        continue;
                    }

                    let rel_key = (spreader_id, *receiver_id);
                    let relationship = world.relationships.get(&rel_key);

                    if let Some(result) = self.try_spread_rumor(
                        spreader,
                        receiver,
                        &rumor_id,
                        relationship,
                        current_tick,
                        rng,
                    ) {
                        results.push(result);
                    }
                }
            }
        }

        results
    }

    /// Simulate gossip spread for one tick using borrowed NPC and relationship data.
    /// This variant avoids borrowing WorldState, allowing it to be called from WorldState::tick.
    pub fn tick_spread_with(
        &mut self,
        npcs: &HashMap<NpcId, AbstractNpc>,
        relationships: &HashMap<(NpcId, NpcId), Relationship>,
        current_tick: u64,
        rng: &mut DeterministicRng,
    ) -> Vec<SpreadResult> {
        let mut results = Vec::new();

        // Collect NPCs who know rumors and might spread them
        let spreaders: Vec<(NpcId, Vec<String>)> = self
            .knowledge
            .iter()
            .map(|(npc_id, known)| (*npc_id, known.keys().cloned().collect()))
            .collect();

        for (spreader_id, known_rumors) in spreaders {
            let spreader = match npcs.get(&spreader_id) {
                Some(npc) => npc,
                None => continue,
            };

            // Skip if not sociable enough
            if spreader.traits.sociability < 20.0 {
                continue;
            }

            for rumor_id in known_rumors {
                let rumor = match self.rumors.get(&rumor_id) {
                    Some(r) => r,
                    None => continue,
                };

                // Skip stale rumors
                if !rumor.is_fresh(current_tick, self.config.rumor_decay_ticks) {
                    continue;
                }

                // Find potential recipients (NPCs the spreader knows)
                for (receiver_id, receiver) in npcs {
                    if *receiver_id == spreader_id {
                        continue;
                    }

                    let rel_key = (spreader_id, *receiver_id);
                    let relationship = relationships.get(&rel_key);

                    if let Some(result) = self.try_spread_rumor(
                        spreader,
                        receiver,
                        &rumor_id,
                        relationship,
                        current_tick,
                        rng,
                    ) {
                        results.push(result);
                    }
                }
            }
        }

        results
    }

    /// Get cumulative reputation effect for an NPC from gossip.
    pub fn get_reputation_modifier(&self, npc_id: NpcId) -> f32 {
        self.reputation_effects.get(&npc_id).copied().unwrap_or(0.0)
    }

    /// Decay old rumors and clean up stale data.
    pub fn cleanup(&mut self, current_tick: u64) {
        let decay_threshold = self.config.rumor_decay_ticks * 2;

        // Remove very old rumors
        self.rumors.retain(|_, rumor| {
            current_tick.saturating_sub(rumor.created_tick) < decay_threshold
        });

        // Remove knowledge of deleted rumors
        let valid_rumors: HashSet<&String> = self.rumors.keys().collect();
        for knowledge in self.knowledge.values_mut() {
            knowledge.retain(|rumor_id, _| valid_rumors.contains(rumor_id));
        }

        // Decay reputation effects over time
        for effect in self.reputation_effects.values_mut() {
            *effect *= 0.99; // Slow decay
        }
        self.reputation_effects.retain(|_, effect| effect.abs() > 0.01);
    }

    /// Build social clusters from relationship data.
    pub fn build_clusters_from_relationships(
        &mut self,
        relationships: &HashMap<(NpcId, NpcId), Relationship>,
        npcs: &HashMap<NpcId, AbstractNpc>,
    ) {
        self.clusters.clear();

        // Simple clustering: group NPCs by strong mutual connections
        let mut assigned: HashSet<NpcId> = HashSet::new();
        let mut cluster_id = 0;

        for npc_id in npcs.keys() {
            if assigned.contains(npc_id) {
                continue;
            }

            let mut cluster = SocialCluster::new(format!("cluster_{}", cluster_id));
            cluster.add_member(*npc_id);
            assigned.insert(*npc_id);

            // Find connected NPCs with strong relationships
            let mut to_check = vec![*npc_id];
            while let Some(current) = to_check.pop() {
                for other_id in npcs.keys() {
                    if assigned.contains(other_id) {
                        continue;
                    }

                    let rel = relationships.get(&(current, *other_id));
                    if let Some(r) = rel {
                        // Strong connection threshold
                        if r.familiarity >= 4.0 && r.trust >= 2.0 {
                            cluster.add_member(*other_id);
                            assigned.insert(*other_id);
                            to_check.push(*other_id);
                        }
                    }
                }
            }

            if cluster.size() > 1 {
                // Calculate cluster stats
                let mut total_trust = 0.0;
                let mut count = 0;
                for m1 in &cluster.members {
                    for m2 in &cluster.members {
                        if m1 != m2 {
                            if let Some(r) = relationships.get(&(*m1, *m2)) {
                                total_trust += r.trust;
                                count += 1;
                            }
                        }
                    }
                }
                if count > 0 {
                    cluster.internal_trust = total_trust / count as f32;
                }
                cluster.cohesion = (cluster.size() as f32 / 10.0).min(1.0);

                self.clusters.insert(cluster.id.clone(), cluster);
                cluster_id += 1;
            }
        }
    }

    /// Get cluster containing an NPC.
    pub fn get_npc_cluster(&self, npc_id: NpcId) -> Option<&SocialCluster> {
        self.clusters.values().find(|c| c.contains(npc_id))
    }

    /// Get all members of an NPC's cluster.
    pub fn get_cluster_members(&self, npc_id: NpcId) -> Vec<NpcId> {
        self.get_npc_cluster(npc_id)
            .map(|c| c.members.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Spread a rumor quickly through a cluster (high-trust environment).
    pub fn spread_within_cluster(
        &mut self,
        rumor_id: &str,
        cluster_id: &str,
        current_tick: u64,
        base_belief: f32,
    ) {
        let cluster = match self.clusters.get(cluster_id) {
            Some(c) => c.clone(),
            None => return,
        };

        for member_id in &cluster.members {
            if self.knows_rumor(*member_id, rumor_id) {
                continue;
            }

            // High trust within cluster = high belief
            let belief = base_belief * (0.7 + cluster.internal_trust / 30.0);
            let distortion = 0.05; // Low distortion within trusted groups

            let knowledge = RumorKnowledge {
                rumor_id: rumor_id.to_string(),
                learned_tick: current_tick,
                belief: belief.clamp(0.3, 1.0),
                distortion,
                source_npc_id: None, // Cluster spread
            };

            self.knowledge
                .entry(*member_id)
                .or_default()
                .insert(rumor_id.to_string(), knowledge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AttachmentStyle, Traits};

    fn test_npc(id: u64, sociability: f32, empathy: f32) -> AbstractNpc {
        AbstractNpc {
            id: NpcId(id),
            age: 30,
            job: "Worker".to_string(),
            district: "Downtown".to_string(),
            household_id: id,
            traits: Traits {
                sociability,
                empathy,
                impulsivity: 50.0,
                ..Default::default()
            },
            seed: id,
            attachment_style: AttachmentStyle::Secure,
        }
    }

    #[test]
    fn test_create_rumor() {
        let mut system = GossipSystem::new();

        let rumor = Rumor::new("rumor1", "event1", NpcId(2), NpcId(1), 100)
            .with_salience(0.8)
            .with_valence(-0.5)
            .with_reputation_impact(-5.0);

        system.create_rumor(rumor);

        assert!(system.rumors.contains_key("rumor1"));
        assert!(system.knows_rumor(NpcId(1), "rumor1")); // Originator knows
        assert!(!system.knows_rumor(NpcId(2), "rumor1")); // Subject doesn't know yet
    }

    #[test]
    fn test_scandal_boost() {
        let rumor = Rumor::new("scandal", "event", NpcId(2), NpcId(1), 0)
            .with_salience(0.5)
            .scandalous();

        assert!(rumor.is_scandalous);
        assert!(rumor.salience > 0.5); // Boosted
    }

    #[test]
    fn test_rumor_decay() {
        let rumor = Rumor::new("old_rumor", "event", NpcId(2), NpcId(1), 0);

        assert!(rumor.is_fresh(100, 200));
        assert!(!rumor.is_fresh(300, 200));

        let decay = rumor.decay_factor(168, 168); // At half-life
        assert!((decay - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_spread_probability() {
        let system = GossipSystem::new();
        let spreader = test_npc(1, 70.0, 50.0);
        let rumor = Rumor::new("r1", "e1", NpcId(3), NpcId(1), 0).with_salience(0.8);

        let relationship = Relationship {
            familiarity: 5.0,
            trust: 5.0,
            ..Default::default()
        };

        let prob = system.calculate_spread_probability(&spreader, NpcId(2), &rumor, Some(&relationship));
        assert!(prob > 0.0);
        assert!(prob < 1.0);

        // No relationship = no spread
        let prob_none = system.calculate_spread_probability(&spreader, NpcId(2), &rumor, None);
        assert_eq!(prob_none, 0.0);
    }

    #[test]
    fn test_belief_calculation() {
        let system = GossipSystem::new();
        let receiver = test_npc(2, 50.0, 70.0);
        let rumor = Rumor::new("r1", "e1", NpcId(3), NpcId(1), 0).with_valence(-0.8);

        let high_trust_rel = Relationship {
            trust: 8.0,
            ..Default::default()
        };

        let low_trust_rel = Relationship {
            trust: -5.0,
            ..Default::default()
        };

        let belief_high = system.calculate_belief(&receiver, NpcId(1), &rumor, Some(&high_trust_rel), 0.0);
        let belief_low = system.calculate_belief(&receiver, NpcId(1), &rumor, Some(&low_trust_rel), 0.0);

        assert!(belief_high > belief_low);
    }

    #[test]
    fn test_social_cluster_building() {
        let mut system = GossipSystem::new();

        let mut npcs = HashMap::new();
        npcs.insert(NpcId(1), test_npc(1, 60.0, 50.0));
        npcs.insert(NpcId(2), test_npc(2, 60.0, 50.0));
        npcs.insert(NpcId(3), test_npc(3, 60.0, 50.0));
        npcs.insert(NpcId(4), test_npc(4, 60.0, 50.0));

        let mut relationships = HashMap::new();
        // Strong bidirectional connections: 1<->2, 2<->3 (should cluster together)
        relationships.insert(
            (NpcId(1), NpcId(2)),
            Relationship { familiarity: 6.0, trust: 5.0, ..Default::default() },
        );
        relationships.insert(
            (NpcId(2), NpcId(1)),
            Relationship { familiarity: 6.0, trust: 5.0, ..Default::default() },
        );
        relationships.insert(
            (NpcId(2), NpcId(3)),
            Relationship { familiarity: 5.0, trust: 4.0, ..Default::default() },
        );
        relationships.insert(
            (NpcId(3), NpcId(2)),
            Relationship { familiarity: 5.0, trust: 4.0, ..Default::default() },
        );
        // Weak connection to 4
        relationships.insert(
            (NpcId(1), NpcId(4)),
            Relationship { familiarity: 1.0, trust: 0.0, ..Default::default() },
        );

        system.build_clusters_from_relationships(&relationships, &npcs);

        // NPCs 1, 2, 3 should be in same cluster
        let cluster1 = system.get_npc_cluster(NpcId(1));
        let cluster2 = system.get_npc_cluster(NpcId(2));

        assert!(cluster1.is_some(), "NPC 1 should be in a cluster");
        if let (Some(c1), Some(c2)) = (cluster1, cluster2) {
            assert_eq!(c1.id, c2.id, "NPCs 1 and 2 should be in same cluster");
        }
    }

    #[test]
    fn test_reputation_effects() {
        let mut system = GossipSystem::new();

        let rumor = Rumor::new("bad_rumor", "event", NpcId(2), NpcId(1), 0)
            .with_reputation_impact(-10.0);

        system.create_rumor(rumor);

        // Simulate someone believing the rumor
        *system.reputation_effects.entry(NpcId(2)).or_default() += -10.0 * 0.8; // 80% belief

        let rep_mod = system.get_reputation_modifier(NpcId(2));
        assert!(rep_mod < 0.0);
    }
}
