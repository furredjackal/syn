//! Deterministic RNG using seeded ChaCha8 for reproducible simulation.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize, Serializer, Deserializer};

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

    /// Generate a random u32.
    pub fn gen_u32(&mut self) -> u32 {
        self.inner.gen()
    }

    /// Generate a random u64.
    pub fn gen_u64(&mut self) -> u64 {
        self.inner.gen()
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
}
