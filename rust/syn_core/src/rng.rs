//! Deterministic RNG using seeded ChaCha8 for reproducible simulation.

use crate::WorldState;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Wrapper around ChaCha8Rng for deterministic randomness.
/// All randomness in SYN derives from seeded instances of this generator.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    inner: ChaCha8Rng,
    seed: u64,
}

impl Serialize for DeterministicRng {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.seed)
    }
}

impl<'de> Deserialize<'de> for DeterministicRng {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seed = u64::deserialize(deserializer)?;
        Ok(DeterministicRng::new(seed))
    }
}

impl DeterministicRng {
    /// Create a new RNG seeded with the given value.
    pub fn new(seed: u64) -> Self {
        DeterministicRng {
            inner: ChaCha8Rng::seed_from_u64(seed),
            seed,
        }
    }

    /// Create a new RNG with domain separation for deterministic, uncorrelated streams.
    ///
    /// Different domains produce completely different sequences even with the same
    /// seed and tick, ensuring systems like tier management, NPC updates, and
    /// the event director don't accidentally correlate.
    ///
    /// # Example
    /// ```ignore
    /// let rng_tiers = DeterministicRng::with_domain(world_seed, tick, "tiers");
    /// let rng_updates = DeterministicRng::with_domain(world_seed, tick, "npc_updates");
    /// let rng_director = DeterministicRng::with_domain(world_seed, tick, "director");
    /// ```
    pub fn with_domain(world_seed: u64, tick: u64, domain: &str) -> Self {
        // Hash the domain string to a u64
        let domain_hash = domain
            .bytes()
            .fold(0x517cc1b727220a95u64, |acc, b| {
                acc.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(b as u64)
            });

        // Combine seed, tick, and domain hash using a mixing function
        let mixed = world_seed
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add(tick.wrapping_mul(0x85ebca6b))
            .wrapping_add(domain_hash);

        Self::new(mixed)
    }

    /// Generate a random u32.
    pub fn gen_u32(&mut self) -> u32 {
        use rand::Rng;
        self.inner.r#gen()
    }

    /// Generate a random u64.
    pub fn gen_u64(&mut self) -> u64 {
        use rand::Rng;
        self.inner.r#gen()
    }

    /// Generate a random f32 in range [0.0..1.0).
    pub fn gen_f32(&mut self) -> f32 {
        self.inner.gen_range(0.0..1.0)
    }

    /// Generate a random value in range [min..max).
    pub fn gen_range_i32(&mut self, min: i32, max: i32) -> i32 {
        self.inner.gen_range(min..max)
    }

    /// Generate a random value in range [min..max).
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        self.inner.gen_range(min..max)
    }

    /// Generate a random boolean with given probability (0.0..1.0).
    pub fn gen_bool(&mut self, probability: f32) -> bool {
        self.gen_f32() < probability
    }

    /// Reseed the RNG (useful for generating sub-deterministic sequences).
    pub fn reseed(&mut self, seed: u64) {
        self.seed = seed;
        self.inner = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Generate a seed suitable for creating sub-generators.
    pub fn derive_seed(&mut self) -> u64 {
        self.gen_u64()
    }
}

/// Build a deterministic RNG seeded from world seed + time so selection is reproducible.
pub fn deterministic_rng_from_world(world: &WorldState) -> DeterministicRng {
    let mix = world
        .game_time
        .tick_index
        .wrapping_mul(0x9E37_79B9_7F4A_7C15);
    DeterministicRng::new(world.seed.0 ^ mix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_sequence() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.gen_u32(), rng2.gen_u32());
        }
    }

    #[test]
    fn test_different_seeds_differ() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(43);

        let val1 = rng1.gen_u32();
        let val2 = rng2.gen_u32();

        assert_ne!(val1, val2);
    }

    #[test]
    fn test_gen_range() {
        let mut rng = DeterministicRng::new(42);
        let val = rng.gen_range_i32(0, 10);
        assert!(val < 10);
    }

    #[test]
    fn test_gen_bool() {
        let mut rng = DeterministicRng::new(42);
        let _ = rng.gen_bool(0.5); // Should not panic
    }

    #[test]
    fn test_with_domain_is_deterministic() {
        let mut rng1 = DeterministicRng::with_domain(12345, 100, "tiers");
        let mut rng2 = DeterministicRng::with_domain(12345, 100, "tiers");

        for _ in 0..50 {
            assert_eq!(rng1.gen_u32(), rng2.gen_u32());
        }
    }

    #[test]
    fn test_different_domains_differ() {
        let mut rng1 = DeterministicRng::with_domain(12345, 100, "tiers");
        let mut rng2 = DeterministicRng::with_domain(12345, 100, "npc_updates");
        let mut rng3 = DeterministicRng::with_domain(12345, 100, "director");

        let val1 = rng1.gen_u32();
        let val2 = rng2.gen_u32();
        let val3 = rng3.gen_u32();

        // All three should produce different sequences
        assert_ne!(val1, val2);
        assert_ne!(val2, val3);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_different_ticks_differ() {
        let mut rng1 = DeterministicRng::with_domain(12345, 100, "tiers");
        let mut rng2 = DeterministicRng::with_domain(12345, 101, "tiers");

        assert_ne!(rng1.gen_u32(), rng2.gen_u32());
    }
}
