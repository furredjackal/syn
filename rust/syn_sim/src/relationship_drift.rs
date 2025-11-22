use syn_core::relationship_model::RelationshipVector;
use syn_core::WorldState;

#[derive(Debug, Clone)]
pub struct RelationshipDriftConfig {
    pub affection_decay_per_tick: f32,
    pub trust_decay_per_tick: f32,
    pub resentment_decay_per_tick: f32,
    pub familiarity_growth_per_tick: f32,
}

#[derive(Debug, Clone)]
pub struct RelationshipDriftSystem {
    pub config: RelationshipDriftConfig,
}

impl RelationshipDriftSystem {
    pub fn new(config: RelationshipDriftConfig) -> Self {
        Self { config }
    }

    pub fn tick(&self, world: &mut WorldState) {
        for ((_actor_id, _target_id), rel) in world.relationships.iter_mut() {
            rel.affection = drift_toward_zero(rel.affection, self.config.affection_decay_per_tick);
            rel.trust = drift_toward_zero(rel.trust, self.config.trust_decay_per_tick);
            rel.resentment =
                drift_toward_zero(rel.resentment, self.config.resentment_decay_per_tick);
            rel.familiarity = clamp_axis(rel.familiarity + self.config.familiarity_growth_per_tick);
        }
    }
}

fn drift_toward_zero(value: f32, amount: f32) -> f32 {
    if amount <= 0.0 {
        return value;
    }
    if value > 0.0 {
        (value - amount).max(0.0)
    } else if value < 0.0 {
        (value + amount).min(0.0)
    } else {
        0.0
    }
}

fn clamp_axis(value: f32) -> f32 {
    value.clamp(-10.0, 10.0)
}

pub fn social_action_utility_modifier(rel: &RelationshipVector) -> f32 {
    let base = 1.0;
    let aff_bonus = rel.affection * 0.05;
    let trust_bonus = rel.trust * 0.03;
    let resent_penalty = rel.resentment * -0.04;
    let mut mult = base + aff_bonus + trust_bonus + resent_penalty;
    if mult < 0.1 {
        mult = 0.1;
    }
    mult
}

pub fn conflict_action_utility_modifier(rel: &RelationshipVector) -> f32 {
    let base = 1.0;
    let resent_bonus = rel.resentment * 0.08;
    let aff_penalty = rel.affection * -0.05;
    let mut mult = base + resent_bonus + aff_penalty;
    if mult < 0.0 {
        mult = 0.0;
    }
    mult
}
