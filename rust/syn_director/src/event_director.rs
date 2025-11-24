use crate::{
    storylet_library::EventContext, storylet_outcome::SimulationContext, Storylet,
    StoryletLibrary,
};
use syn_core::DeterministicRng;

impl crate::EventDirector {
    /// Scan storylets using tag index and prerequisite checks.
    pub fn scan<'a>(&'a self, ctx: &EventContext, library: &'a StoryletLibrary) -> Vec<&'a Storylet> {
        library
            .eligible_for(ctx)
            .into_iter()
            .filter(|s| s.prerequisites.passes(ctx))
            .collect()
    }

    /// Choose a storylet deterministically using weighted factors from context.
    pub fn choose<'a>(&self, options: &[&'a Storylet], ctx: &EventContext) -> Option<&'a Storylet> {
        let mut rng = DeterministicRng::new(ctx.seed);
        let mut best: Option<(&Storylet, f32)> = None;
        for storylet in options {
            let weight = storylet.weight
                * ctx.mood.multiplier()
                * ctx.personality.bias()
                * ctx.relationship_vector.factor();
            if let Some((_, best_weight)) = best {
                if weight > best_weight + f32::EPSILON {
                    best = Some((*storylet, weight));
                } else if (weight - best_weight).abs() <= f32::EPSILON {
                    // deterministic tie-breaker
                    if rng.gen_u64() % 2 == 0 {
                        best = Some((*storylet, weight));
                    }
                }
            } else {
                best = Some((*storylet, weight));
            }
        }
        best.map(|(s, _)| s)
    }

    /// Construct and apply a storylet event.
    pub fn construct_event(
        &self,
        chosen: &Storylet,
        ctx: &mut SimulationContext,
    ) {
        let _roles = chosen.roles.assign(&ctx.event);
        chosen.outcomes.apply(ctx);
        // Memory entry handling would be wired here when available.
    }
}
