//! Population simulation system.
//!
//! Per GDD §15: Implements job market supply/demand, population aging, demographic
//! cohorts, and black-swan events. The population evolves over time affecting
//! job availability, housing costs, and social dynamics.

use crate::district::DistrictId;
use crate::rng::DeterministicRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Age cohort for demographic tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgeCohort {
    /// 0-5: Infants and toddlers
    Infant,
    /// 6-12: Children
    Child,
    /// 13-17: Teenagers
    Teen,
    /// 18-24: Young adults, entering workforce
    YoungAdult,
    /// 25-34: Early career
    EarlyCareer,
    /// 35-44: Mid career
    MidCareer,
    /// 45-54: Late career
    LateCareer,
    /// 55-64: Pre-retirement
    PreRetirement,
    /// 65+: Retired
    Retired,
}

impl AgeCohort {
    /// Get cohort from age.
    pub fn from_age(age: u32) -> Self {
        match age {
            0..=5 => Self::Infant,
            6..=12 => Self::Child,
            13..=17 => Self::Teen,
            18..=24 => Self::YoungAdult,
            25..=34 => Self::EarlyCareer,
            35..=44 => Self::MidCareer,
            45..=54 => Self::LateCareer,
            55..=64 => Self::PreRetirement,
            _ => Self::Retired,
        }
    }

    /// Whether this cohort participates in the labor force.
    pub fn is_working_age(&self) -> bool {
        matches!(
            self,
            Self::YoungAdult
                | Self::EarlyCareer
                | Self::MidCareer
                | Self::LateCareer
                | Self::PreRetirement
        )
    }

    /// Base consumption multiplier for economic modeling.
    pub fn consumption_multiplier(&self) -> f32 {
        match self {
            Self::Infant => 0.3,
            Self::Child => 0.5,
            Self::Teen => 0.7,
            Self::YoungAdult => 1.0,
            Self::EarlyCareer => 1.2,
            Self::MidCareer => 1.4,
            Self::LateCareer => 1.3,
            Self::PreRetirement => 1.1,
            Self::Retired => 0.8,
        }
    }

    /// Healthcare demand multiplier.
    pub fn healthcare_demand(&self) -> f32 {
        match self {
            Self::Infant => 1.5,
            Self::Child => 0.8,
            Self::Teen => 0.6,
            Self::YoungAdult => 0.5,
            Self::EarlyCareer => 0.6,
            Self::MidCareer => 0.8,
            Self::LateCareer => 1.2,
            Self::PreRetirement => 1.5,
            Self::Retired => 2.5,
        }
    }
}

/// Job sector categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobSector {
    /// Technology, software, IT
    Technology,
    /// Healthcare, medical
    Healthcare,
    /// Finance, banking, insurance
    Finance,
    /// Manufacturing, industrial
    Manufacturing,
    /// Retail, sales
    Retail,
    /// Food service, hospitality
    Hospitality,
    /// Education, academia
    Education,
    /// Government, public sector
    Government,
    /// Construction, trades
    Construction,
    /// Arts, entertainment, media
    Entertainment,
    /// Transportation, logistics
    Transportation,
    /// Agriculture, farming
    Agriculture,
    /// Criminal enterprises (underground economy)
    Criminal,
    /// Self-employed, gig economy
    Freelance,
    /// Unemployed (not a real sector, but tracked)
    Unemployed,
}

impl JobSector {
    /// Base salary range (annual, in game currency units).
    pub fn salary_range(&self) -> (f32, f32) {
        match self {
            Self::Technology => (60000.0, 200000.0),
            Self::Healthcare => (40000.0, 300000.0),
            Self::Finance => (50000.0, 500000.0),
            Self::Manufacturing => (30000.0, 80000.0),
            Self::Retail => (20000.0, 50000.0),
            Self::Hospitality => (18000.0, 45000.0),
            Self::Education => (35000.0, 90000.0),
            Self::Government => (40000.0, 120000.0),
            Self::Construction => (35000.0, 100000.0),
            Self::Entertainment => (20000.0, 500000.0),
            Self::Transportation => (30000.0, 70000.0),
            Self::Agriculture => (25000.0, 60000.0),
            Self::Criminal => (10000.0, 1000000.0),
            Self::Freelance => (15000.0, 150000.0),
            Self::Unemployed => (0.0, 0.0),
        }
    }

    /// Economic sensitivity (how much sector is affected by economic swings).
    pub fn economic_sensitivity(&self) -> f32 {
        match self {
            Self::Technology => 1.2,
            Self::Healthcare => 0.3,    // Recession-resistant
            Self::Finance => 1.5,
            Self::Manufacturing => 1.4,
            Self::Retail => 1.3,
            Self::Hospitality => 1.6,   // Very sensitive
            Self::Education => 0.4,     // Stable
            Self::Government => 0.2,    // Very stable
            Self::Construction => 1.5,
            Self::Entertainment => 1.4,
            Self::Transportation => 1.0,
            Self::Agriculture => 0.6,
            Self::Criminal => -0.5,     // Inversely correlated
            Self::Freelance => 1.1,
            Self::Unemployed => 0.0,
        }
    }

    /// Skill requirements (affects who can work in this sector).
    pub fn education_requirement(&self) -> EducationLevel {
        match self {
            Self::Technology => EducationLevel::Bachelors,
            Self::Healthcare => EducationLevel::Graduate,
            Self::Finance => EducationLevel::Bachelors,
            Self::Manufacturing => EducationLevel::HighSchool,
            Self::Retail => EducationLevel::None,
            Self::Hospitality => EducationLevel::None,
            Self::Education => EducationLevel::Graduate,
            Self::Government => EducationLevel::Bachelors,
            Self::Construction => EducationLevel::Vocational,
            Self::Entertainment => EducationLevel::None,
            Self::Transportation => EducationLevel::HighSchool,
            Self::Agriculture => EducationLevel::None,
            Self::Criminal => EducationLevel::None,
            Self::Freelance => EducationLevel::None,
            Self::Unemployed => EducationLevel::None,
        }
    }
}

impl Default for JobSector {
    fn default() -> Self {
        Self::Unemployed
    }
}

/// Education level for job market matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EducationLevel {
    /// No formal education.
    None,
    /// High school diploma.
    HighSchool,
    /// Vocational/trade school.
    Vocational,
    /// Associate's degree.
    Associates,
    /// Bachelor's degree.
    Bachelors,
    /// Master's degree.
    Graduate,
    /// Doctoral degree.
    Doctorate,
}

impl Default for EducationLevel {
    fn default() -> Self {
        Self::None
    }
}

/// Job market state for a sector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorMarket {
    /// Sector type
    pub sector: JobSector,
    /// Number of jobs available
    pub job_openings: u32,
    /// Number of people seeking jobs in this sector
    pub job_seekers: u32,
    /// Current average salary (affected by supply/demand)
    pub average_salary: f32,
    /// Job growth rate (-1.0 to 1.0)
    pub growth_rate: f32,
    /// Automation pressure (0.0-1.0, higher = more jobs being automated)
    pub automation_pressure: f32,
}

impl SectorMarket {
    /// Create a new sector market with default values.
    pub fn new(sector: JobSector) -> Self {
        let (min_sal, max_sal) = sector.salary_range();
        Self {
            sector,
            job_openings: 100,
            job_seekers: 80,
            average_salary: (min_sal + max_sal) / 2.0,
            growth_rate: 0.0,
            automation_pressure: 0.0,
        }
    }

    /// Calculate supply/demand ratio (>1 = more jobs than seekers, <1 = more seekers than jobs).
    pub fn supply_demand_ratio(&self) -> f32 {
        if self.job_seekers == 0 {
            return 10.0;
        }
        self.job_openings as f32 / self.job_seekers as f32
    }

    /// Get unemployment rate for this sector.
    pub fn unemployment_rate(&self) -> f32 {
        if self.job_seekers == 0 {
            return 0.0;
        }
        let employed = self.job_openings.min(self.job_seekers);
        1.0 - (employed as f32 / self.job_seekers as f32)
    }

    /// Update market based on economic conditions.
    pub fn tick(&mut self, economic_index: f32, rng: &mut DeterministicRng) {
        let sensitivity = self.sector.economic_sensitivity();
        let (min_sal, max_sal) = self.sector.salary_range();

        // Economic impact on job openings
        let econ_factor = 1.0 + (economic_index - 50.0) / 100.0 * sensitivity;
        let base_openings = 100.0 * econ_factor;
        self.job_openings = (base_openings + rng.gen_range_f32(-10.0, 10.0)).max(0.0) as u32;

        // Salary adjusts based on supply/demand
        let ratio = self.supply_demand_ratio();
        let salary_adjustment = (ratio - 1.0) * 0.02; // 2% adjustment per unit imbalance
        self.average_salary *= 1.0 + salary_adjustment;
        self.average_salary = self.average_salary.clamp(min_sal, max_sal);

        // Growth rate follows economic trends
        self.growth_rate = (economic_index - 50.0) / 100.0 * sensitivity;
        self.growth_rate += rng.gen_range_f32(-0.05, 0.05);
        self.growth_rate = self.growth_rate.clamp(-0.3, 0.3);

        // Automation gradually increases for susceptible sectors
        if matches!(
            self.sector,
            JobSector::Manufacturing | JobSector::Retail | JobSector::Transportation
        ) {
            self.automation_pressure += 0.001;
            self.automation_pressure = self.automation_pressure.min(0.5);
            // Automation reduces job openings
            self.job_openings = (self.job_openings as f32 * (1.0 - self.automation_pressure * 0.1)) as u32;
        }
    }
}

/// Black swan event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlackSwanEvent {
    /// Economic recession - reduces jobs and salaries.
    Recession {
        /// How severe (0.0-1.0).
        severity: f32,
        /// How long it lasts.
        duration_ticks: u64,
        /// When it started.
        started_tick: u64,
    },
    /// Economic boom - increases jobs and spending.
    Boom {
        /// How strong the boom is.
        magnitude: f32,
        /// How long it lasts.
        duration_ticks: u64,
        /// When it started.
        started_tick: u64,
    },
    /// Pandemic/health crisis - affects mortality and healthcare.
    Pandemic {
        /// Death rate modifier.
        mortality_rate: f32,
        /// Spread rate.
        infection_rate: f32,
        /// When it started.
        started_tick: u64,
        /// When it peaks.
        peak_tick: u64,
    },
    /// Natural disaster - localized damage.
    NaturalDisaster {
        /// Which districts are affected.
        affected_districts: Vec<DistrictId>,
        /// How severe (0.0-1.0).
        severity: f32,
        /// When it started.
        started_tick: u64,
    },
    /// Major employer collapse - sector-wide job losses.
    CorporateCollapse {
        /// Which sector is hit.
        sector: JobSector,
        /// Number of jobs lost.
        jobs_lost: u32,
        /// When it started.
        started_tick: u64,
    },
    /// Tech revolution - automates jobs in a sector.
    TechRevolution {
        /// Which sector is being automated.
        automating_sector: JobSector,
        /// Fraction of jobs displaced (0.0-1.0).
        job_displacement: f32,
        /// When it started.
        started_tick: u64,
    },
    /// Housing crisis - rent and home prices spike.
    HousingCrisis {
        /// How much rents increase.
        rent_spike: f32,
        /// When it started.
        started_tick: u64,
    },
    /// Crime wave - increases crime in districts.
    CrimeWave {
        /// How intense (0.0-1.0).
        intensity: f32,
        /// Which districts are affected.
        affected_districts: Vec<DistrictId>,
        /// When it started.
        started_tick: u64,
    },
}

impl BlackSwanEvent {
    /// Check if event is still active.
    pub fn is_active(&self, current_tick: u64) -> bool {
        match self {
            Self::Recession { started_tick, duration_ticks, .. } => {
                current_tick < started_tick + duration_ticks
            }
            Self::Boom { started_tick, duration_ticks, .. } => {
                current_tick < started_tick + duration_ticks
            }
            Self::Pandemic { started_tick, peak_tick, .. } => {
                // Pandemic lasts until 2x the time to peak after peak
                let duration = (peak_tick - started_tick) * 3;
                current_tick < started_tick + duration
            }
            Self::NaturalDisaster { started_tick, .. } => {
                // Disasters have immediate impact, then 30 days recovery
                current_tick < started_tick + 720
            }
            Self::CorporateCollapse { started_tick, .. } => {
                // 90 days of ripple effects
                current_tick < started_tick + 2160
            }
            Self::TechRevolution { started_tick, .. } => {
                // Permanent, always active once started
                current_tick >= *started_tick
            }
            Self::HousingCrisis { started_tick, .. } => {
                // 6 months of crisis
                current_tick < started_tick + 4320
            }
            Self::CrimeWave { started_tick, .. } => {
                // 60 days
                current_tick < started_tick + 1440
            }
        }
    }

    /// Get economic modifier from this event.
    pub fn economic_modifier(&self, current_tick: u64) -> f32 {
        if !self.is_active(current_tick) {
            return 0.0;
        }

        match self {
            Self::Recession { severity, started_tick, duration_ticks } => {
                let progress = (current_tick - started_tick) as f32 / *duration_ticks as f32;
                // V-shaped: worst at middle
                let intensity = if progress < 0.5 {
                    progress * 2.0
                } else {
                    2.0 - progress * 2.0
                };
                -severity * intensity * 30.0
            }
            Self::Boom { magnitude, started_tick, duration_ticks } => {
                let progress = (current_tick - started_tick) as f32 / *duration_ticks as f32;
                let intensity = 1.0 - (progress - 0.5).abs() * 2.0;
                magnitude * intensity * 20.0
            }
            Self::Pandemic { .. } => -15.0,
            Self::NaturalDisaster { severity, .. } => -severity * 25.0,
            Self::CorporateCollapse { .. } => -10.0,
            Self::TechRevolution { .. } => 5.0, // Net positive long-term
            Self::HousingCrisis { .. } => -8.0,
            Self::CrimeWave { .. } => -5.0,
        }
    }
}

/// Demographic data for population tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Demographics {
    /// Population by age cohort
    pub cohorts: HashMap<AgeCohort, u32>,
    /// Population by district
    pub by_district: HashMap<DistrictId, u32>,
    /// Employment by sector
    pub employment: HashMap<JobSector, u32>,
    /// Education level distribution
    pub education: HashMap<EducationLevel, u32>,
    /// Total population
    pub total_population: u32,
    /// Birth rate (per 1000 per year)
    pub birth_rate: f32,
    /// Death rate (per 1000 per year)
    pub death_rate: f32,
    /// Net migration rate (per 1000 per year)
    pub migration_rate: f32,
}

impl Demographics {
    /// Create new demographics with default birth/death/migration rates.
    pub fn new() -> Self {
        Self {
            birth_rate: 12.0,
            death_rate: 8.0,
            migration_rate: 2.0,
            ..Default::default()
        }
    }

    /// Get working age population.
    pub fn working_age_population(&self) -> u32 {
        self.cohorts
            .iter()
            .filter(|(cohort, _)| cohort.is_working_age())
            .map(|(_, count)| count)
            .sum()
    }

    /// Get unemployment count.
    pub fn unemployed(&self) -> u32 {
        self.employment.get(&JobSector::Unemployed).copied().unwrap_or(0)
    }

    /// Get unemployment rate.
    pub fn unemployment_rate(&self) -> f32 {
        let working = self.working_age_population();
        if working == 0 {
            return 0.0;
        }
        self.unemployed() as f32 / working as f32
    }

    /// Get dependency ratio (non-working / working).
    pub fn dependency_ratio(&self) -> f32 {
        let working = self.working_age_population();
        if working == 0 {
            return 0.0;
        }
        let dependent = self.total_population - working;
        dependent as f32 / working as f32
    }
}

/// Main population simulation system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationSimulation {
    /// Current demographics
    pub demographics: Demographics,
    /// Job market by sector
    pub job_markets: HashMap<JobSector, SectorMarket>,
    /// Active black swan events
    pub active_events: Vec<BlackSwanEvent>,
    /// Historical events (for narrative reference)
    pub event_history: Vec<(u64, String)>,
    /// Global economic index (0-100, 50 = normal)
    pub economic_index: f32,
    /// Housing affordability index (0-100, 50 = normal)
    pub housing_index: f32,
    /// Configuration
    pub config: PopulationConfig,
    /// Last tick processed
    pub last_tick: u64,
}

/// Configuration for population simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    /// Ticks between simulation updates (batch for performance).
    pub ticks_per_step: u64,
    /// Base probability of black swan event per year.
    pub black_swan_probability: f32,
    /// Multiplier for economic volatility.
    pub economic_volatility: f32,
    /// Enable birth/death/migration simulation.
    pub enable_demographics: bool,
    /// Enable job market supply/demand simulation.
    pub enable_job_market: bool,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            ticks_per_step: 24, // Once per day
            black_swan_probability: 0.05, // 5% per year
            economic_volatility: 1.0,
            enable_demographics: true,
            enable_job_market: true,
        }
    }
}

impl PopulationSimulation {
    /// Create a new population simulation with default config.
    pub fn new() -> Self {
        let mut sim = Self {
            demographics: Demographics::new(),
            economic_index: 50.0,
            housing_index: 50.0,
            ..Default::default()
        };

        // Initialize job markets
        for sector in [
            JobSector::Technology,
            JobSector::Healthcare,
            JobSector::Finance,
            JobSector::Manufacturing,
            JobSector::Retail,
            JobSector::Hospitality,
            JobSector::Education,
            JobSector::Government,
            JobSector::Construction,
            JobSector::Entertainment,
            JobSector::Transportation,
            JobSector::Freelance,
        ] {
            sim.job_markets.insert(sector, SectorMarket::new(sector));
        }

        sim
    }

    /// Generate initial population for a city.
    pub fn generate_initial_population(&mut self, total_pop: u32, seed: u64) {
        let mut rng = DeterministicRng::with_domain(seed, 0, "pop_gen");

        self.demographics.total_population = total_pop;

        // Distribute by age cohort (rough US-like distribution)
        let cohort_weights = [
            (AgeCohort::Infant, 0.06),
            (AgeCohort::Child, 0.08),
            (AgeCohort::Teen, 0.07),
            (AgeCohort::YoungAdult, 0.12),
            (AgeCohort::EarlyCareer, 0.15),
            (AgeCohort::MidCareer, 0.14),
            (AgeCohort::LateCareer, 0.13),
            (AgeCohort::PreRetirement, 0.10),
            (AgeCohort::Retired, 0.15),
        ];

        for (cohort, weight) in cohort_weights {
            let count = (total_pop as f32 * weight) as u32;
            self.demographics.cohorts.insert(cohort, count);
        }

        // Distribute by education
        let working_pop = self.demographics.working_age_population();
        self.demographics.education.insert(EducationLevel::None, (working_pop as f32 * 0.08) as u32);
        self.demographics.education.insert(EducationLevel::HighSchool, (working_pop as f32 * 0.27) as u32);
        self.demographics.education.insert(EducationLevel::Vocational, (working_pop as f32 * 0.10) as u32);
        self.demographics.education.insert(EducationLevel::Associates, (working_pop as f32 * 0.10) as u32);
        self.demographics.education.insert(EducationLevel::Bachelors, (working_pop as f32 * 0.30) as u32);
        self.demographics.education.insert(EducationLevel::Graduate, (working_pop as f32 * 0.12) as u32);
        self.demographics.education.insert(EducationLevel::Doctorate, (working_pop as f32 * 0.03) as u32);

        // Distribute employment
        let employed = (working_pop as f32 * 0.95) as u32; // 5% unemployment
        let sector_weights = [
            (JobSector::Healthcare, 0.14),
            (JobSector::Retail, 0.12),
            (JobSector::Education, 0.09),
            (JobSector::Hospitality, 0.10),
            (JobSector::Technology, 0.08),
            (JobSector::Finance, 0.06),
            (JobSector::Manufacturing, 0.08),
            (JobSector::Construction, 0.06),
            (JobSector::Government, 0.07),
            (JobSector::Transportation, 0.05),
            (JobSector::Entertainment, 0.04),
            (JobSector::Freelance, 0.06),
            (JobSector::Agriculture, 0.02),
            (JobSector::Criminal, 0.03),
        ];

        for (sector, weight) in sector_weights {
            let count = (employed as f32 * weight) as u32;
            self.demographics.employment.insert(sector, count);
            
            // Update job market
            if let Some(market) = self.job_markets.get_mut(&sector) {
                market.job_seekers = count + rng.gen_range_i32(0, 50) as u32;
                market.job_openings = count + rng.gen_range_i32(0, 30) as u32;
            }
        }

        self.demographics.employment.insert(JobSector::Unemployed, working_pop - employed);
    }

    /// Main tick function - advances simulation.
    pub fn tick(&mut self, current_tick: u64, seed: u64) {
        // Only process every N ticks
        if current_tick - self.last_tick < self.config.ticks_per_step {
            return;
        }
        self.last_tick = current_tick;

        let mut rng = DeterministicRng::with_domain(seed, current_tick, "pop_sim");

        // Update economic index based on events
        self.update_economic_index(current_tick);

        // Update job markets
        if self.config.enable_job_market {
            self.update_job_markets(&mut rng);
        }

        // Update demographics (births, deaths, aging)
        if self.config.enable_demographics {
            self.update_demographics(current_tick, &mut rng);
        }

        // Check for black swan events
        self.check_black_swan_events(current_tick, &mut rng);

        // Clean up expired events
        self.active_events.retain(|e| e.is_active(current_tick));
    }

    /// Update economic index based on active events and natural fluctuation.
    fn update_economic_index(&mut self, current_tick: u64) {
        let mut modifier = 0.0;

        for event in &self.active_events {
            modifier += event.economic_modifier(current_tick);
        }

        // Apply modifier with smoothing
        let target = (50.0 + modifier).clamp(10.0, 90.0);
        self.economic_index += (target - self.economic_index) * 0.1;
    }

    /// Update all job markets.
    fn update_job_markets(&mut self, rng: &mut DeterministicRng) {
        for market in self.job_markets.values_mut() {
            market.tick(self.economic_index, rng);
        }

        // Rebalance job seekers based on employment
        let total_unemployed = self.demographics.unemployed();
        for (sector, market) in &mut self.job_markets {
            if *sector != JobSector::Unemployed {
                // Some unemployed seek jobs in growing sectors
                if market.growth_rate > 0.0 {
                    market.job_seekers += (total_unemployed as f32 * market.growth_rate * 0.1) as u32;
                }
            }
        }
    }

    /// Update demographic changes.
    fn update_demographics(&mut self, current_tick: u64, _rng: &mut DeterministicRng) {
        // Approximate yearly changes per tick (assuming 8760 ticks/year)
        let tick_rate = 1.0 / 8760.0;

        // Births
        let births = (self.demographics.total_population as f32 
            * self.demographics.birth_rate / 1000.0 
            * tick_rate) as u32;
        *self.demographics.cohorts.entry(AgeCohort::Infant).or_default() += births;
        self.demographics.total_population += births;

        // Deaths (higher in older cohorts)
        let mut deaths = 0u32;
        for (cohort, count) in &mut self.demographics.cohorts {
            let cohort_death_rate = match cohort {
                AgeCohort::Infant => 0.006,
                AgeCohort::Child => 0.0002,
                AgeCohort::Teen => 0.0004,
                AgeCohort::YoungAdult => 0.001,
                AgeCohort::EarlyCareer => 0.001,
                AgeCohort::MidCareer => 0.002,
                AgeCohort::LateCareer => 0.005,
                AgeCohort::PreRetirement => 0.01,
                AgeCohort::Retired => 0.04,
            };
            let cohort_deaths = (*count as f32 * cohort_death_rate * tick_rate) as u32;
            *count = count.saturating_sub(cohort_deaths);
            deaths += cohort_deaths;
        }
        self.demographics.total_population = self.demographics.total_population.saturating_sub(deaths);

        // Migration (affected by economic conditions)
        let migration_modifier = (self.economic_index - 50.0) / 50.0; // -1 to +1
        let net_migration = (self.demographics.total_population as f32 
            * (self.demographics.migration_rate + migration_modifier * 2.0) / 1000.0 
            * tick_rate) as i32;

        if net_migration > 0 {
            // Immigrants tend to be working age
            *self.demographics.cohorts.entry(AgeCohort::EarlyCareer).or_default() += net_migration as u32 / 2;
            *self.demographics.cohorts.entry(AgeCohort::YoungAdult).or_default() += net_migration as u32 / 2;
            self.demographics.total_population += net_migration as u32;
        } else if net_migration < 0 {
            // Emigrants also working age
            let emigrants = (-net_migration) as u32;
            *self.demographics.cohorts.entry(AgeCohort::EarlyCareer).or_default() = 
                self.demographics.cohorts.get(&AgeCohort::EarlyCareer).unwrap_or(&0).saturating_sub(emigrants / 2);
            *self.demographics.cohorts.entry(AgeCohort::YoungAdult).or_default() = 
                self.demographics.cohorts.get(&AgeCohort::YoungAdult).unwrap_or(&0).saturating_sub(emigrants / 2);
            self.demographics.total_population = self.demographics.total_population.saturating_sub(emigrants);
        }

        // Aging (once per year, simplified - every 365 ticks)
        if current_tick % 8760 == 0 {
            self.age_population();
        }
    }

    /// Age population by one year (shift cohorts).
    fn age_population(&mut self) {
        let _cohort_order = [
            AgeCohort::Infant,
            AgeCohort::Child,
            AgeCohort::Teen,
            AgeCohort::YoungAdult,
            AgeCohort::EarlyCareer,
            AgeCohort::MidCareer,
            AgeCohort::LateCareer,
            AgeCohort::PreRetirement,
            AgeCohort::Retired,
        ];

        // Calculate transition rates (what fraction moves to next cohort)
        let transition_rates = [
            (AgeCohort::Infant, AgeCohort::Child, 1.0 / 6.0),       // 6 years in infant
            (AgeCohort::Child, AgeCohort::Teen, 1.0 / 7.0),         // 7 years
            (AgeCohort::Teen, AgeCohort::YoungAdult, 1.0 / 5.0),    // 5 years
            (AgeCohort::YoungAdult, AgeCohort::EarlyCareer, 1.0 / 7.0),
            (AgeCohort::EarlyCareer, AgeCohort::MidCareer, 1.0 / 10.0),
            (AgeCohort::MidCareer, AgeCohort::LateCareer, 1.0 / 10.0),
            (AgeCohort::LateCareer, AgeCohort::PreRetirement, 1.0 / 10.0),
            (AgeCohort::PreRetirement, AgeCohort::Retired, 1.0 / 10.0),
        ];

        let mut transitions: Vec<(AgeCohort, AgeCohort, u32)> = Vec::new();
        
        for (from, to, rate) in transition_rates {
            let count = self.demographics.cohorts.get(&from).unwrap_or(&0);
            let moving = (*count as f32 * rate) as u32;
            transitions.push((from, to, moving));
        }

        for (from, to, moving) in transitions {
            *self.demographics.cohorts.entry(from).or_default() = 
                self.demographics.cohorts.get(&from).unwrap_or(&0).saturating_sub(moving);
            *self.demographics.cohorts.entry(to).or_default() += moving;
        }
    }

    /// Check and potentially trigger black swan events.
    fn check_black_swan_events(&mut self, current_tick: u64, rng: &mut DeterministicRng) {
        // Check probability per year (assuming 8760 ticks/year)
        let check_prob = self.config.black_swan_probability / 8760.0;

        if rng.gen_f32() < check_prob {
            // Roll for event type
            let roll = rng.gen_range_i32(0, 100);
            let event = match roll {
                0..=20 => self.generate_recession(current_tick, rng),
                21..=35 => self.generate_boom(current_tick, rng),
                36..=45 => self.generate_pandemic(current_tick, rng),
                46..=55 => self.generate_corporate_collapse(current_tick, rng),
                56..=65 => self.generate_housing_crisis(current_tick, rng),
                66..=80 => self.generate_crime_wave(current_tick, rng),
                _ => self.generate_tech_revolution(current_tick, rng),
            };

            if let Some(event) = event {
                let description = self.describe_event(&event);
                self.event_history.push((current_tick, description));
                self.active_events.push(event);
            }
        }
    }

    fn generate_recession(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        // Don't stack recessions
        if self.active_events.iter().any(|e| matches!(e, BlackSwanEvent::Recession { .. })) {
            return None;
        }

        Some(BlackSwanEvent::Recession {
            severity: rng.gen_range_f32(0.3, 0.9),
            duration_ticks: rng.gen_range_i32(2000, 6000) as u64, // 3-9 months
            started_tick: current_tick,
        })
    }

    fn generate_boom(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        if self.active_events.iter().any(|e| matches!(e, BlackSwanEvent::Boom { .. })) {
            return None;
        }

        Some(BlackSwanEvent::Boom {
            magnitude: rng.gen_range_f32(0.3, 0.7),
            duration_ticks: rng.gen_range_i32(1500, 4000) as u64,
            started_tick: current_tick,
        })
    }

    fn generate_pandemic(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        Some(BlackSwanEvent::Pandemic {
            mortality_rate: rng.gen_range_f32(0.001, 0.02),
            infection_rate: rng.gen_range_f32(0.1, 0.4),
            started_tick: current_tick,
            peak_tick: current_tick + rng.gen_range_i32(500, 1500) as u64,
        })
    }

    fn generate_corporate_collapse(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        let sectors = [
            JobSector::Technology,
            JobSector::Finance,
            JobSector::Manufacturing,
            JobSector::Retail,
        ];
        let sector = sectors[rng.gen_range_i32(0, sectors.len() as i32) as usize];

        Some(BlackSwanEvent::CorporateCollapse {
            sector,
            jobs_lost: rng.gen_range_i32(500, 5000) as u32,
            started_tick: current_tick,
        })
    }

    fn generate_housing_crisis(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        Some(BlackSwanEvent::HousingCrisis {
            rent_spike: rng.gen_range_f32(0.2, 0.5),
            started_tick: current_tick,
        })
    }

    fn generate_crime_wave(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        // Affect 2-4 districts
        let num_districts = rng.gen_range_i32(2, 5);
        let districts: Vec<DistrictId> = (0..num_districts)
            .map(|_| DistrictId(rng.gen_range_i32(0, 10) as u32))
            .collect();

        Some(BlackSwanEvent::CrimeWave {
            intensity: rng.gen_range_f32(0.3, 0.8),
            affected_districts: districts,
            started_tick: current_tick,
        })
    }

    fn generate_tech_revolution(&self, current_tick: u64, rng: &mut DeterministicRng) -> Option<BlackSwanEvent> {
        let automatable = [
            JobSector::Manufacturing,
            JobSector::Retail,
            JobSector::Transportation,
            JobSector::Hospitality,
        ];
        let sector = automatable[rng.gen_range_i32(0, automatable.len() as i32) as usize];

        Some(BlackSwanEvent::TechRevolution {
            automating_sector: sector,
            job_displacement: rng.gen_range_f32(0.1, 0.3),
            started_tick: current_tick,
        })
    }

    fn describe_event(&self, event: &BlackSwanEvent) -> String {
        match event {
            BlackSwanEvent::Recession { severity, .. } => {
                format!("Economic recession begins (severity: {:.0}%)", severity * 100.0)
            }
            BlackSwanEvent::Boom { magnitude, .. } => {
                format!("Economic boom begins (magnitude: {:.0}%)", magnitude * 100.0)
            }
            BlackSwanEvent::Pandemic { .. } => {
                "Health crisis declared".to_string()
            }
            BlackSwanEvent::CorporateCollapse { sector, jobs_lost, .. } => {
                format!("Major {:?} employer collapses, {} jobs lost", sector, jobs_lost)
            }
            BlackSwanEvent::HousingCrisis { rent_spike, .. } => {
                format!("Housing crisis: rents spike {:.0}%", rent_spike * 100.0)
            }
            BlackSwanEvent::CrimeWave { intensity, .. } => {
                format!("Crime wave sweeps city (intensity: {:.0}%)", intensity * 100.0)
            }
            BlackSwanEvent::TechRevolution { automating_sector, .. } => {
                format!("Automation wave hits {:?} sector", automating_sector)
            }
            BlackSwanEvent::NaturalDisaster { severity, .. } => {
                format!("Natural disaster strikes (severity: {:.0}%)", severity * 100.0)
            }
        }
    }

    /// Get current job openings in a sector.
    pub fn get_job_openings(&self, sector: JobSector) -> u32 {
        self.job_markets
            .get(&sector)
            .map(|m| m.job_openings)
            .unwrap_or(0)
    }

    /// Get average salary in a sector.
    pub fn get_average_salary(&self, sector: JobSector) -> f32 {
        self.job_markets
            .get(&sector)
            .map(|m| m.average_salary)
            .unwrap_or(0.0)
    }

    /// Get overall unemployment rate.
    pub fn get_unemployment_rate(&self) -> f32 {
        self.demographics.unemployment_rate()
    }

    /// Check if any crisis is active.
    pub fn has_active_crisis(&self) -> bool {
        self.active_events.iter().any(|e| {
            matches!(
                e,
                BlackSwanEvent::Recession { .. }
                    | BlackSwanEvent::Pandemic { .. }
                    | BlackSwanEvent::HousingCrisis { .. }
                    | BlackSwanEvent::CrimeWave { .. }
            )
        })
    }

    /// Get list of active event descriptions.
    pub fn active_event_descriptions(&self, current_tick: u64) -> Vec<String> {
        self.active_events
            .iter()
            .filter(|e| e.is_active(current_tick))
            .map(|e| self.describe_event(e))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_cohort_from_age() {
        assert_eq!(AgeCohort::from_age(3), AgeCohort::Infant);
        assert_eq!(AgeCohort::from_age(10), AgeCohort::Child);
        assert_eq!(AgeCohort::from_age(16), AgeCohort::Teen);
        assert_eq!(AgeCohort::from_age(22), AgeCohort::YoungAdult);
        assert_eq!(AgeCohort::from_age(30), AgeCohort::EarlyCareer);
        assert_eq!(AgeCohort::from_age(40), AgeCohort::MidCareer);
        assert_eq!(AgeCohort::from_age(50), AgeCohort::LateCareer);
        assert_eq!(AgeCohort::from_age(60), AgeCohort::PreRetirement);
        assert_eq!(AgeCohort::from_age(70), AgeCohort::Retired);
    }

    #[test]
    fn test_working_age() {
        assert!(!AgeCohort::Child.is_working_age());
        assert!(!AgeCohort::Teen.is_working_age());
        assert!(AgeCohort::YoungAdult.is_working_age());
        assert!(AgeCohort::EarlyCareer.is_working_age());
        assert!(!AgeCohort::Retired.is_working_age());
    }

    #[test]
    fn test_sector_supply_demand() {
        let mut market = SectorMarket::new(JobSector::Technology);
        market.job_openings = 100;
        market.job_seekers = 50;

        assert!((market.supply_demand_ratio() - 2.0).abs() < 0.01);
        assert!((market.unemployment_rate() - 0.0).abs() < 0.01);

        market.job_openings = 30;
        market.job_seekers = 100;
        assert!((market.unemployment_rate() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_population_generation() {
        let mut sim = PopulationSimulation::new();
        sim.generate_initial_population(100000, 42);

        assert_eq!(sim.demographics.total_population, 100000);
        assert!(sim.demographics.working_age_population() > 50000);
        assert!(sim.demographics.unemployment_rate() < 0.1);
    }

    #[test]
    fn test_recession_event() {
        let event = BlackSwanEvent::Recession {
            severity: 0.5,
            duration_ticks: 1000,
            started_tick: 0,
        };

        assert!(event.is_active(500));
        assert!(!event.is_active(1500));
        assert!(event.economic_modifier(500) < 0.0);
    }

    #[test]
    fn test_economic_index_response() {
        let mut sim = PopulationSimulation::new();
        sim.generate_initial_population(10000, 42);

        let initial_index = sim.economic_index;

        // Add recession
        sim.active_events.push(BlackSwanEvent::Recession {
            severity: 0.8,
            duration_ticks: 1000,
            started_tick: 0,
        });

        // Tick to update
        sim.update_economic_index(500);

        assert!(sim.economic_index < initial_index);
    }

    #[test]
    fn test_job_market_tick() {
        let mut sim = PopulationSimulation::new();
        let mut rng = DeterministicRng::new(42);

        let initial_salary = sim.job_markets.get(&JobSector::Technology).unwrap().average_salary;

        // High economy should increase salaries
        sim.economic_index = 80.0;
        sim.update_job_markets(&mut rng);

        // Market should respond to economic conditions
        let market = sim.job_markets.get(&JobSector::Technology).unwrap();
        assert!(market.growth_rate > 0.0);
    }
}
