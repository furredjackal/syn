//! District system: procedural city districts with crime, economy, and social values.
//!
//! Per GDD §14: Districts own a micro-economy, crime potential, social clusters, and event hotspots.
//! Districts evolve over time (gentrify, decay, rebound) based on global simulation values.

use crate::rng::DeterministicRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a district.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DistrictId(pub u32);

impl DistrictId {
    /// Create a new district ID.
    pub fn new(id: u32) -> Self {
        DistrictId(id)
    }
}

impl From<u32> for DistrictId {
    fn from(id: u32) -> Self {
        DistrictId(id)
    }
}

/// Economic health level of a district.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EconomicTier {
    /// Severe poverty, high unemployment
    Depressed,
    /// Below average, struggling businesses
    Declining,
    /// Average economic activity
    Stable,
    /// Above average, growing businesses
    Growing,
    /// High prosperity, expensive area
    Prosperous,
}

impl EconomicTier {
    /// Convert from numeric value (0.0-100.0)
    pub fn from_value(value: f32) -> Self {
        match value {
            v if v < 20.0 => Self::Depressed,
            v if v < 40.0 => Self::Declining,
            v if v < 60.0 => Self::Stable,
            v if v < 80.0 => Self::Growing,
            _ => Self::Prosperous,
        }
    }

    /// Base modifier for wealth-related events
    pub fn wealth_modifier(&self) -> f32 {
        match self {
            Self::Depressed => -20.0,
            Self::Declining => -10.0,
            Self::Stable => 0.0,
            Self::Growing => 10.0,
            Self::Prosperous => 25.0,
        }
    }

    /// Job availability modifier (affects employment storylets)
    pub fn job_modifier(&self) -> f32 {
        match self {
            Self::Depressed => -30.0,
            Self::Declining => -15.0,
            Self::Stable => 0.0,
            Self::Growing => 15.0,
            Self::Prosperous => 10.0, // High competition in prosperous areas
        }
    }
}

impl Default for EconomicTier {
    fn default() -> Self {
        Self::Stable
    }
}

/// Crime severity level of a district.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrimeLevel {
    /// Very safe, rare incidents
    Minimal,
    /// Below average crime
    Low,
    /// Average crime rate
    Moderate,
    /// Above average, noticeable crime
    High,
    /// Dangerous, frequent crime
    Severe,
}

impl CrimeLevel {
    /// Convert from numeric value (0.0-100.0)
    pub fn from_value(value: f32) -> Self {
        match value {
            v if v < 15.0 => Self::Minimal,
            v if v < 35.0 => Self::Low,
            v if v < 55.0 => Self::Moderate,
            v if v < 75.0 => Self::High,
            _ => Self::Severe,
        }
    }

    /// Safety modifier (affects health/mood events)
    pub fn safety_modifier(&self) -> f32 {
        match self {
            Self::Minimal => 15.0,
            Self::Low => 5.0,
            Self::Moderate => 0.0,
            Self::High => -10.0,
            Self::Severe => -25.0,
        }
    }

    /// Risk of crime-related storylets firing
    pub fn crime_event_weight(&self) -> f32 {
        match self {
            Self::Minimal => 0.05,
            Self::Low => 0.15,
            Self::Moderate => 0.30,
            Self::High => 0.50,
            Self::Severe => 0.75,
        }
    }
}

impl Default for CrimeLevel {
    fn default() -> Self {
        Self::Moderate
    }
}

/// District archetype affects baseline generation and event flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistrictType {
    /// City center, high density
    Downtown,
    /// Family-oriented, lower density
    Suburban,
    /// Factories, warehouses
    Industrial,
    /// Shops, entertainment
    Commercial,
    /// Universities, schools
    Academic,
    /// Parks, nature areas
    Recreational,
    /// Upscale homes, gated areas
    Affluent,
    /// Low-income housing
    Projects,
}

impl DistrictType {
    /// Baseline crime modifier
    pub fn base_crime_modifier(&self) -> f32 {
        match self {
            Self::Downtown => 10.0,
            Self::Suburban => -15.0,
            Self::Industrial => 5.0,
            Self::Commercial => 5.0,
            Self::Academic => -10.0,
            Self::Recreational => -5.0,
            Self::Affluent => -20.0,
            Self::Projects => 20.0,
        }
    }

    /// Baseline economy modifier
    pub fn base_economy_modifier(&self) -> f32 {
        match self {
            Self::Downtown => 15.0,
            Self::Suburban => 5.0,
            Self::Industrial => 0.0,
            Self::Commercial => 20.0,
            Self::Academic => 10.0,
            Self::Recreational => -5.0,
            Self::Affluent => 30.0,
            Self::Projects => -25.0,
        }
    }
}

impl Default for DistrictType {
    fn default() -> Self {
        Self::Suburban
    }
}

/// Full district state containing all simulation values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct District {
    /// Unique district identifier
    pub id: DistrictId,
    /// Human-readable name
    pub name: String,
    /// District archetype
    pub district_type: DistrictType,

    // === Economic Values ===
    /// Raw economy value (0.0-100.0)
    pub economy: f32,
    /// Unemployment rate (0.0-1.0)
    pub unemployment: f32,
    /// Average rent level (affects housing costs)
    pub rent_index: f32,
    /// Business growth rate (-1.0 to 1.0)
    pub business_growth: f32,

    // === Crime Values ===
    /// Raw crime value (0.0-100.0)
    pub crime: f32,
    /// Police presence (0.0-100.0, affects crime suppression)
    pub police_presence: f32,
    /// Gang activity level (0.0-100.0)
    pub gang_activity: f32,

    // === Social Values ===
    /// Community cohesion (0.0-100.0, affects social support)
    pub community_cohesion: f32,
    /// Cultural vibrancy (0.0-100.0, affects entertainment/art events)
    pub cultural_index: f32,
    /// Education quality (0.0-100.0)
    pub education_quality: f32,

    // === Environmental ===
    /// Pollution level (0.0-100.0, affects health)
    pub pollution: f32,
    /// Green space coverage (0.0-100.0, affects mood)
    pub green_space: f32,

    // === Population ===
    /// Rough population count
    pub population: u32,
    /// Population density (people per unit area)
    pub density: f32,

    // === Trend Tracking ===
    /// Economic momentum (-1.0 declining to 1.0 growing)
    pub economy_trend: f32,
    /// Crime momentum (-1.0 decreasing to 1.0 increasing)
    pub crime_trend: f32,
    /// Gentrification pressure (0.0-1.0)
    pub gentrification: f32,
}

impl District {
    /// Create a new district with default values.
    pub fn new(id: DistrictId, name: String, district_type: DistrictType) -> Self {
        let base_economy = 50.0 + district_type.base_economy_modifier();
        let base_crime = 40.0 + district_type.base_crime_modifier();

        Self {
            id,
            name,
            district_type,
            economy: base_economy.clamp(0.0, 100.0),
            unemployment: 0.05,
            rent_index: base_economy / 50.0,
            business_growth: 0.0,
            crime: base_crime.clamp(0.0, 100.0),
            police_presence: 50.0,
            gang_activity: (base_crime * 0.3).clamp(0.0, 100.0),
            community_cohesion: 50.0,
            cultural_index: 50.0,
            education_quality: 50.0,
            pollution: 30.0,
            green_space: 40.0,
            population: 10000,
            density: 1.0,
            economy_trend: 0.0,
            crime_trend: 0.0,
            gentrification: 0.0,
        }
    }

    /// Generate a district procedurally from a seed.
    pub fn generate(id: DistrictId, name: String, district_type: DistrictType, seed: u64) -> Self {
        let mut rng = DeterministicRng::with_domain(seed, id.0 as u64, "district_gen");

        let base_economy = 50.0 + district_type.base_economy_modifier();
        let base_crime = 40.0 + district_type.base_crime_modifier();

        // Add random variation
        let economy = (base_economy + rng.gen_range_f32(-15.0, 15.0)).clamp(0.0, 100.0);
        let crime = (base_crime + rng.gen_range_f32(-15.0, 15.0)).clamp(0.0, 100.0);

        Self {
            id,
            name,
            district_type,
            economy,
            unemployment: rng.gen_range_f32(0.02, 0.15),
            rent_index: economy / 50.0 * rng.gen_range_f32(0.8, 1.2),
            business_growth: rng.gen_range_f32(-0.1, 0.1),
            crime,
            police_presence: rng.gen_range_f32(30.0, 70.0),
            gang_activity: (crime * rng.gen_range_f32(0.2, 0.5)).clamp(0.0, 100.0),
            community_cohesion: rng.gen_range_f32(30.0, 70.0),
            cultural_index: rng.gen_range_f32(20.0, 80.0),
            education_quality: rng.gen_range_f32(30.0, 70.0),
            pollution: rng.gen_range_f32(10.0, 60.0),
            green_space: rng.gen_range_f32(20.0, 60.0),
            population: rng.gen_range_f32(5000.0, 50000.0) as u32,
            density: rng.gen_range_f32(0.5, 3.0),
            economy_trend: rng.gen_range_f32(-0.1, 0.1),
            crime_trend: rng.gen_range_f32(-0.1, 0.1),
            gentrification: rng.gen_range_f32(0.0, 0.3),
        }
    }

    /// Get the economic tier band.
    pub fn economic_tier(&self) -> EconomicTier {
        EconomicTier::from_value(self.economy)
    }

    /// Get the crime level band.
    pub fn crime_level(&self) -> CrimeLevel {
        CrimeLevel::from_value(self.crime)
    }

    /// Get overall safety score (inverse of crime, modified by police).
    pub fn safety(&self) -> f32 {
        let base_safety = 100.0 - self.crime;
        let police_bonus = (self.police_presence - 50.0) * 0.2;
        (base_safety + police_bonus).clamp(0.0, 100.0)
    }

    /// Get livability score (composite of multiple factors).
    pub fn livability(&self) -> f32 {
        let safety = self.safety();
        let economy = self.economy;
        let environment = (self.green_space - self.pollution).clamp(0.0, 100.0);
        let community = self.community_cohesion;

        // Weighted average
        safety * 0.3 + economy * 0.25 + environment * 0.2 + community * 0.25
    }

    /// Get desirability for housing (affects rent and move decisions).
    pub fn desirability(&self) -> f32 {
        let livability = self.livability();
        let culture_bonus = self.cultural_index * 0.1;
        let education_bonus = self.education_quality * 0.1;
        (livability + culture_bonus + education_bonus).clamp(0.0, 100.0)
    }

    /// Tick the district simulation forward (called each world tick).
    pub fn tick(&mut self, rng: &mut DeterministicRng) {
        // Apply trends with dampening
        self.economy += self.economy_trend * 0.1;
        self.crime += self.crime_trend * 0.1;

        // Natural decay toward baseline
        self.economy_trend *= 0.99;
        self.crime_trend *= 0.99;

        // Crime responds to economy inversely
        if self.economy < 30.0 {
            self.crime_trend += 0.01;
        } else if self.economy > 70.0 {
            self.crime_trend -= 0.005;
        }

        // Police presence suppresses crime trend
        if self.police_presence > 60.0 && self.crime > 50.0 {
            self.crime_trend -= 0.01;
        }

        // Gentrification effects
        if self.gentrification > 0.5 {
            self.rent_index += 0.001;
            self.economy_trend += 0.005;
            self.community_cohesion -= 0.01; // Displacement effect
        }

        // Small random fluctuations
        self.economy += rng.gen_range_f32(-0.5, 0.5);
        self.crime += rng.gen_range_f32(-0.3, 0.3);

        // Clamp all values
        self.clamp();
    }

    /// Apply an economic shock (positive or negative).
    pub fn apply_economic_event(&mut self, delta: f32) {
        self.economy += delta;
        self.economy_trend += delta * 0.1;

        // Ripple effects
        if delta < -10.0 {
            self.unemployment += 0.02;
            self.crime_trend += 0.05;
        } else if delta > 10.0 {
            self.unemployment -= 0.01;
            self.business_growth += 0.05;
        }

        self.clamp();
    }

    /// Apply a crime event (spike or crackdown).
    pub fn apply_crime_event(&mut self, delta: f32) {
        self.crime += delta;
        self.crime_trend += delta * 0.1;

        // Ripple effects
        if delta > 15.0 {
            self.community_cohesion -= 5.0;
            self.economy_trend -= 0.02;
        } else if delta < -15.0 {
            self.community_cohesion += 2.0;
        }

        self.clamp();
    }

    /// Clamp all values to valid ranges.
    fn clamp(&mut self) {
        self.economy = self.economy.clamp(0.0, 100.0);
        self.crime = self.crime.clamp(0.0, 100.0);
        self.unemployment = self.unemployment.clamp(0.0, 1.0);
        self.rent_index = self.rent_index.clamp(0.1, 5.0);
        self.business_growth = self.business_growth.clamp(-1.0, 1.0);
        self.police_presence = self.police_presence.clamp(0.0, 100.0);
        self.gang_activity = self.gang_activity.clamp(0.0, 100.0);
        self.community_cohesion = self.community_cohesion.clamp(0.0, 100.0);
        self.cultural_index = self.cultural_index.clamp(0.0, 100.0);
        self.education_quality = self.education_quality.clamp(0.0, 100.0);
        self.pollution = self.pollution.clamp(0.0, 100.0);
        self.green_space = self.green_space.clamp(0.0, 100.0);
        self.economy_trend = self.economy_trend.clamp(-1.0, 1.0);
        self.crime_trend = self.crime_trend.clamp(-1.0, 1.0);
        self.gentrification = self.gentrification.clamp(0.0, 1.0);
    }
}

impl Default for District {
    fn default() -> Self {
        Self::new(DistrictId(0), "Unknown".to_string(), DistrictType::Suburban)
    }
}

/// Registry of all districts in the world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistrictRegistry {
    /// All districts by ID
    pub districts: HashMap<DistrictId, District>,
    /// Name to ID lookup
    pub name_index: HashMap<String, DistrictId>,
    /// Next available ID
    next_id: u32,
}

impl DistrictRegistry {
    /// Create a new empty district registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a default city with standard districts.
    pub fn generate_default_city(seed: u64) -> Self {
        let mut registry = Self::new();

        let district_defs = [
            ("Downtown", DistrictType::Downtown),
            ("Westside", DistrictType::Suburban),
            ("Industrial Park", DistrictType::Industrial),
            ("The Strip", DistrictType::Commercial),
            ("University District", DistrictType::Academic),
            ("Riverside Park", DistrictType::Recreational),
            ("Highland Heights", DistrictType::Affluent),
            ("Eastside Projects", DistrictType::Projects),
            ("Midtown", DistrictType::Commercial),
            ("Old Town", DistrictType::Suburban),
        ];

        for (name, district_type) in district_defs {
            registry.add_generated(name.to_string(), district_type, seed);
        }

        registry
    }

    /// Add a new district with default values.
    pub fn add(&mut self, name: String, district_type: DistrictType) -> DistrictId {
        let id = DistrictId(self.next_id);
        self.next_id += 1;

        let district = District::new(id, name.clone(), district_type);
        self.districts.insert(id, district);
        self.name_index.insert(name, id);

        id
    }

    /// Add a procedurally generated district.
    pub fn add_generated(&mut self, name: String, district_type: DistrictType, seed: u64) -> DistrictId {
        let id = DistrictId(self.next_id);
        self.next_id += 1;

        let district = District::generate(id, name.clone(), district_type, seed);
        self.districts.insert(id, district);
        self.name_index.insert(name, id);

        id
    }

    /// Get district by ID.
    pub fn get(&self, id: DistrictId) -> Option<&District> {
        self.districts.get(&id)
    }

    /// Get mutable district by ID.
    pub fn get_mut(&mut self, id: DistrictId) -> Option<&mut District> {
        self.districts.get_mut(&id)
    }

    /// Get district by name.
    pub fn get_by_name(&self, name: &str) -> Option<&District> {
        self.name_index.get(name).and_then(|id| self.districts.get(id))
    }

    /// Get mutable district by name.
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut District> {
        if let Some(&id) = self.name_index.get(name) {
            self.districts.get_mut(&id)
        } else {
            None
        }
    }

    /// Get ID for a district name.
    pub fn id_for_name(&self, name: &str) -> Option<DistrictId> {
        self.name_index.get(name).copied()
    }

    /// List all district names.
    pub fn list_names(&self) -> Vec<&String> {
        self.name_index.keys().collect()
    }

    /// List all district IDs.
    pub fn list_ids(&self) -> Vec<DistrictId> {
        self.districts.keys().copied().collect()
    }

    /// Get count of districts.
    pub fn len(&self) -> usize {
        self.districts.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.districts.is_empty()
    }

    /// Tick all districts forward.
    pub fn tick_all(&mut self, rng: &mut DeterministicRng) {
        for district in self.districts.values_mut() {
            district.tick(rng);
        }
    }

    /// Get the safest district.
    pub fn safest(&self) -> Option<&District> {
        self.districts.values().max_by(|a, b| {
            a.safety().partial_cmp(&b.safety()).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the most dangerous district.
    pub fn most_dangerous(&self) -> Option<&District> {
        self.districts.values().max_by(|a, b| {
            a.crime.partial_cmp(&b.crime).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the wealthiest district.
    pub fn wealthiest(&self) -> Option<&District> {
        self.districts.values().max_by(|a, b| {
            a.economy.partial_cmp(&b.economy).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get average crime across all districts.
    pub fn average_crime(&self) -> f32 {
        if self.districts.is_empty() {
            return 0.0;
        }
        let total: f32 = self.districts.values().map(|d| d.crime).sum();
        total / self.districts.len() as f32
    }

    /// Get average economy across all districts.
    pub fn average_economy(&self) -> f32 {
        if self.districts.is_empty() {
            return 0.0;
        }
        let total: f32 = self.districts.values().map(|d| d.economy).sum();
        total / self.districts.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_district_creation() {
        let district = District::new(
            DistrictId(1),
            "TestDistrict".to_string(),
            DistrictType::Downtown,
        );
        assert_eq!(district.name, "TestDistrict");
        assert_eq!(district.district_type, DistrictType::Downtown);
        assert!(district.economy > 50.0); // Downtown gets economy bonus
    }

    #[test]
    fn test_district_generation_deterministic() {
        let d1 = District::generate(DistrictId(1), "Test".to_string(), DistrictType::Suburban, 12345);
        let d2 = District::generate(DistrictId(1), "Test".to_string(), DistrictType::Suburban, 12345);

        assert_eq!(d1.economy, d2.economy);
        assert_eq!(d1.crime, d2.crime);
        assert_eq!(d1.population, d2.population);
    }

    #[test]
    fn test_district_generation_varies_with_seed() {
        let d1 = District::generate(DistrictId(1), "Test".to_string(), DistrictType::Suburban, 12345);
        let d2 = District::generate(DistrictId(1), "Test".to_string(), DistrictType::Suburban, 54321);

        // Different seeds should produce different values
        assert_ne!(d1.economy, d2.economy);
    }

    #[test]
    fn test_economic_tier_bands() {
        assert_eq!(EconomicTier::from_value(10.0), EconomicTier::Depressed);
        assert_eq!(EconomicTier::from_value(30.0), EconomicTier::Declining);
        assert_eq!(EconomicTier::from_value(50.0), EconomicTier::Stable);
        assert_eq!(EconomicTier::from_value(70.0), EconomicTier::Growing);
        assert_eq!(EconomicTier::from_value(90.0), EconomicTier::Prosperous);
    }

    #[test]
    fn test_crime_level_bands() {
        assert_eq!(CrimeLevel::from_value(10.0), CrimeLevel::Minimal);
        assert_eq!(CrimeLevel::from_value(25.0), CrimeLevel::Low);
        assert_eq!(CrimeLevel::from_value(45.0), CrimeLevel::Moderate);
        assert_eq!(CrimeLevel::from_value(65.0), CrimeLevel::High);
        assert_eq!(CrimeLevel::from_value(85.0), CrimeLevel::Severe);
    }

    #[test]
    fn test_district_registry() {
        let mut registry = DistrictRegistry::new();
        let id1 = registry.add("Downtown".to_string(), DistrictType::Downtown);
        let id2 = registry.add("Suburbs".to_string(), DistrictType::Suburban);

        assert_eq!(registry.len(), 2);
        assert!(registry.get(id1).is_some());
        assert!(registry.get_by_name("Suburbs").is_some());
        assert_eq!(registry.id_for_name("Downtown"), Some(id1));
    }

    #[test]
    fn test_default_city_generation() {
        let registry = DistrictRegistry::generate_default_city(42);
        assert_eq!(registry.len(), 10);
        assert!(registry.get_by_name("Downtown").is_some());
        assert!(registry.get_by_name("Highland Heights").is_some());
    }

    #[test]
    fn test_district_safety_calculation() {
        let mut district = District::new(DistrictId(1), "Test".to_string(), DistrictType::Affluent);
        district.crime = 20.0;
        district.police_presence = 70.0;

        let safety = district.safety();
        assert!(safety > 80.0); // Low crime + high police = high safety
    }

    #[test]
    fn test_district_tick_simulation() {
        let mut district = District::new(DistrictId(1), "Test".to_string(), DistrictType::Suburban);
        let initial_economy = district.economy;
        district.economy_trend = 0.5; // Strong growth

        // Tick multiple times to overcome random noise
        let mut rng = DeterministicRng::with_domain(42, 0, "test");
        for _ in 0..10 {
            district.tick(&mut rng);
        }

        assert!(district.economy > initial_economy); // Should have grown overall
    }

    #[test]
    fn test_economic_event_ripple() {
        let mut district = District::new(DistrictId(1), "Test".to_string(), DistrictType::Industrial);
        let initial_crime_trend = district.crime_trend;

        district.apply_economic_event(-20.0); // Economic crash

        assert!(district.unemployment > 0.05); // Unemployment increased
        assert!(district.crime_trend > initial_crime_trend); // Crime trend increased
    }
}
