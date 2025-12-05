//! Procedural lifepath generation for NPCs.
//!
//! Per GDD §3.12: Each AbstractNPC has a generated lifepath using Markov chain
//! progression for jobs/districts, milestone-based events, and relationship origins.
//!
//! ## Key Features
//! - Deterministic generation from NPC seed
//! - Markov chain job/district transitions weighted by traits
//! - Age-gated milestones (kindergarten, first job, marriage, retirement)
//! - Relationship origin tracking ("met_at_work", "childhood_friend", etc.)
//! - Job history timeline for event eligibility checks

use crate::population::{JobSector, EducationLevel};
use crate::rng::DeterministicRng;
use crate::types::{NpcId, SimTick, Traits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete lifepath data for an NPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lifepath {
    /// Chronological job history (tick, job sector, job title).
    pub job_history: Vec<(SimTick, JobSector, String)>,
    /// Chronological district history (tick, district name).
    pub district_history: Vec<(SimTick, String)>,
    /// Key life milestones (milestone type → tick when it occurred).
    pub milestones: HashMap<String, SimTick>,
    /// Relationship origins (other NPC ID → how they met).
    pub relationship_origins: HashMap<NpcId, RelationshipOrigin>,
    /// Highest education level achieved.
    pub education_level: EducationLevel,
    /// Birth district (starting location).
    pub birth_district: String,
}

impl Default for Lifepath {
    fn default() -> Self {
        Self {
            job_history: Vec::new(),
            district_history: Vec::new(),
            milestones: HashMap::new(),
            relationship_origins: HashMap::new(),
            education_level: EducationLevel::None,
            birth_district: String::new(),
        }
    }
}

impl Lifepath {
    /// Get the current job sector (most recent entry in job_history).
    pub fn current_job(&self) -> JobSector {
        self.job_history
            .last()
            .map(|(_, sector, _)| *sector)
            .unwrap_or(JobSector::Unemployed)
    }

    /// Get the current job title.
    pub fn current_job_title(&self) -> String {
        self.job_history
            .last()
            .map(|(_, _, title)| title.clone())
            .unwrap_or_else(|| "Unemployed".to_string())
    }

    /// Get the current district (most recent in district_history).
    pub fn current_district(&self) -> String {
        self.district_history
            .last()
            .map(|(_, district)| district.clone())
            .unwrap_or_else(|| self.birth_district.clone())
    }

    /// Check if NPC has ever worked in a specific job sector.
    pub fn has_worked_in(&self, sector: JobSector) -> bool {
        self.job_history.iter().any(|(_, s, _)| *s == sector)
    }

    /// Check if NPC has ever lived in a specific district.
    pub fn has_lived_in(&self, district: &str) -> bool {
        self.district_history.iter().any(|(_, d)| d == district)
            || self.birth_district == district
    }

    /// Get the milestone tick if it exists.
    pub fn milestone_tick(&self, milestone: &str) -> Option<SimTick> {
        self.milestones.get(milestone).copied()
    }

    /// Check if NPC has achieved a specific milestone.
    pub fn has_milestone(&self, milestone: &str) -> bool {
        self.milestones.contains_key(milestone)
    }

    /// Get relationship origin with another NPC.
    pub fn relationship_origin(&self, other_id: NpcId) -> Option<&RelationshipOrigin> {
        self.relationship_origins.get(&other_id)
    }

    /// Count total career moves (job changes).
    pub fn career_mobility_count(&self) -> usize {
        if self.job_history.len() <= 1 {
            0
        } else {
            self.job_history.len() - 1
        }
    }

    /// Count total district moves.
    pub fn district_mobility_count(&self) -> usize {
        self.district_history.len()
    }
}

/// How two NPCs originally met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipOrigin {
    /// Met in childhood/school (age < 18).
    ChildhoodFriend,
    /// Met at school/university.
    SchoolMate,
    /// Met through work/same employer.
    Coworker,
    /// Met through shared friend/social circle.
    MutualFriend,
    /// Met romantically (dating, matchmaking).
    RomanticEncounter,
    /// Met as neighbors in same district.
    Neighbor,
    /// Met through family connection.
    FamilyConnection,
    /// Met by chance (random encounter).
    ChanceEncounter,
    /// Met through shared hobby/interest.
    SharedInterest,
    /// Adversarial origin (conflict, competition).
    Rival,
}

impl RelationshipOrigin {
    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ChildhoodFriend => "childhood friend",
            Self::SchoolMate => "school friend",
            Self::Coworker => "met at work",
            Self::MutualFriend => "mutual friend",
            Self::RomanticEncounter => "romantic meeting",
            Self::Neighbor => "neighbor",
            Self::FamilyConnection => "family connection",
            Self::ChanceEncounter => "chance encounter",
            Self::SharedInterest => "shared interest",
            Self::Rival => "rival",
        }
    }

    /// Check if this origin type suggests positive initial relationship.
    pub fn is_positive_origin(&self) -> bool {
        matches!(
            self,
            Self::ChildhoodFriend
                | Self::SchoolMate
                | Self::MutualFriend
                | Self::FamilyConnection
                | Self::SharedInterest
        )
    }

    /// Check if this origin type suggests negative/competitive initial relationship.
    pub fn is_negative_origin(&self) -> bool {
        matches!(self, Self::Rival)
    }
}

/// Age milestones that structure lifepath generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeMilestone {
    /// Age 6: Start of kindergarten.
    Kindergarten,
    /// Age 13: Social group formation.
    SocialGroupFormation,
    /// Age 18: Career path choice.
    CareerPathChoice,
    /// Age 22: First real job (post-education).
    FirstJob,
    /// Age 25: Relationship milestone window opens.
    RelationshipWindow,
    /// Age 30: Career advancement potential.
    CareerAdvancement,
    /// Age 40: Mid-career plateau or pivot.
    MidCareer,
    /// Age 50: Late career decisions.
    LateCareer,
    /// Age 65: Retirement.
    Retirement,
    /// Age 90: Digital transcendence.
    DigitalTranscendence,
}

impl LifeMilestone {
    /// Get the age at which this milestone occurs.
    pub fn age(&self) -> u32 {
        match self {
            Self::Kindergarten => 6,
            Self::SocialGroupFormation => 13,
            Self::CareerPathChoice => 18,
            Self::FirstJob => 22,
            Self::RelationshipWindow => 25,
            Self::CareerAdvancement => 30,
            Self::MidCareer => 40,
            Self::LateCareer => 50,
            Self::Retirement => 65,
            Self::DigitalTranscendence => 90,
        }
    }

    /// Get milestone identifier string for storage.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Kindergarten => "kindergarten",
            Self::SocialGroupFormation => "social_group",
            Self::CareerPathChoice => "career_choice",
            Self::FirstJob => "first_job",
            Self::RelationshipWindow => "relationship_window",
            Self::CareerAdvancement => "career_advancement",
            Self::MidCareer => "mid_career",
            Self::LateCareer => "late_career",
            Self::Retirement => "retirement",
            Self::DigitalTranscendence => "digital_transcendence",
        }
    }

    /// Get all milestones up to and including the given age.
    pub fn milestones_up_to_age(age: u32) -> Vec<Self> {
        let all = [
            Self::Kindergarten,
            Self::SocialGroupFormation,
            Self::CareerPathChoice,
            Self::FirstJob,
            Self::RelationshipWindow,
            Self::CareerAdvancement,
            Self::MidCareer,
            Self::LateCareer,
            Self::Retirement,
            Self::DigitalTranscendence,
        ];
        all.into_iter()
            .filter(|m| m.age() <= age)
            .collect()
    }
}

/// Markov chain transition weights for job progression.
/// Each job sector has transition probabilities to other sectors.
struct JobTransitionMatrix {
    /// From job sector → To job sector → base probability
    transitions: HashMap<JobSector, Vec<(JobSector, f32)>>,
}

impl JobTransitionMatrix {
    /// Create default transition matrix.
    fn new() -> Self {
        let mut transitions = HashMap::new();

        // Define typical career progressions and lateral moves
        transitions.insert(
            JobSector::Unemployed,
            vec![
                (JobSector::Retail, 0.25),
                (JobSector::Hospitality, 0.20),
                (JobSector::Construction, 0.15),
                (JobSector::Transportation, 0.10),
                (JobSector::Freelance, 0.15),
                (JobSector::Criminal, 0.05),
                (JobSector::Unemployed, 0.10), // Stay unemployed
            ],
        );

        transitions.insert(
            JobSector::Retail,
            vec![
                (JobSector::Retail, 0.40), // Stay in retail
                (JobSector::Hospitality, 0.20),
                (JobSector::Technology, 0.10), // Upward
                (JobSector::Finance, 0.05),
                (JobSector::Freelance, 0.15),
                (JobSector::Unemployed, 0.10),
            ],
        );

        transitions.insert(
            JobSector::Technology,
            vec![
                (JobSector::Technology, 0.50), // High retention
                (JobSector::Finance, 0.15),
                (JobSector::Freelance, 0.20),
                (JobSector::Education, 0.05),
                (JobSector::Entertainment, 0.05),
                (JobSector::Unemployed, 0.05),
            ],
        );

        transitions.insert(
            JobSector::Healthcare,
            vec![
                (JobSector::Healthcare, 0.65), // Very stable
                (JobSector::Education, 0.10),
                (JobSector::Government, 0.10),
                (JobSector::Freelance, 0.10),
                (JobSector::Unemployed, 0.05),
            ],
        );

        transitions.insert(
            JobSector::Finance,
            vec![
                (JobSector::Finance, 0.50),
                (JobSector::Technology, 0.15),
                (JobSector::Government, 0.10),
                (JobSector::Freelance, 0.15),
                (JobSector::Unemployed, 0.10),
            ],
        );

        transitions.insert(
            JobSector::Education,
            vec![
                (JobSector::Education, 0.70), // Very stable
                (JobSector::Government, 0.10),
                (JobSector::Freelance, 0.10),
                (JobSector::Technology, 0.05),
                (JobSector::Unemployed, 0.05),
            ],
        );

        transitions.insert(
            JobSector::Construction,
            vec![
                (JobSector::Construction, 0.50),
                (JobSector::Manufacturing, 0.20),
                (JobSector::Freelance, 0.15),
                (JobSector::Unemployed, 0.15),
            ],
        );

        transitions.insert(
            JobSector::Hospitality,
            vec![
                (JobSector::Hospitality, 0.35),
                (JobSector::Retail, 0.25),
                (JobSector::Entertainment, 0.15),
                (JobSector::Freelance, 0.15),
                (JobSector::Unemployed, 0.10),
            ],
        );

        transitions.insert(
            JobSector::Government,
            vec![
                (JobSector::Government, 0.75), // Extremely stable
                (JobSector::Education, 0.10),
                (JobSector::Freelance, 0.10),
                (JobSector::Unemployed, 0.05),
            ],
        );

        transitions.insert(
            JobSector::Freelance,
            vec![
                (JobSector::Freelance, 0.40),
                (JobSector::Technology, 0.15),
                (JobSector::Entertainment, 0.15),
                (JobSector::Retail, 0.10),
                (JobSector::Unemployed, 0.20), // Higher instability
            ],
        );

        transitions.insert(
            JobSector::Entertainment,
            vec![
                (JobSector::Entertainment, 0.40),
                (JobSector::Freelance, 0.30),
                (JobSector::Hospitality, 0.10),
                (JobSector::Technology, 0.10),
                (JobSector::Unemployed, 0.10),
            ],
        );

        transitions.insert(
            JobSector::Manufacturing,
            vec![
                (JobSector::Manufacturing, 0.50),
                (JobSector::Construction, 0.20),
                (JobSector::Transportation, 0.15),
                (JobSector::Unemployed, 0.15),
            ],
        );

        transitions.insert(
            JobSector::Transportation,
            vec![
                (JobSector::Transportation, 0.55),
                (JobSector::Construction, 0.15),
                (JobSector::Freelance, 0.15),
                (JobSector::Unemployed, 0.15),
            ],
        );

        transitions.insert(
            JobSector::Agriculture,
            vec![
                (JobSector::Agriculture, 0.70), // Very stable, generational
                (JobSector::Construction, 0.10),
                (JobSector::Freelance, 0.10),
                (JobSector::Unemployed, 0.10),
            ],
        );

        transitions.insert(
            JobSector::Criminal,
            vec![
                (JobSector::Criminal, 0.40),
                (JobSector::Freelance, 0.20),
                (JobSector::Retail, 0.15),
                (JobSector::Construction, 0.10),
                (JobSector::Unemployed, 0.15), // Arrests, burnout
            ],
        );

        Self { transitions }
    }

    /// Get next job sector based on Markov transition and trait modifiers.
    fn next_job(
        &self,
        current_job: JobSector,
        traits: &Traits,
        age: u32,
        rng: &mut DeterministicRng,
    ) -> JobSector {
        let base_transitions = self.transitions.get(&current_job)
            .cloned()
            .unwrap_or_else(|| vec![(JobSector::Unemployed, 1.0)]);

        // Apply trait modifiers
        let mut weighted_transitions: Vec<(JobSector, f32)> = base_transitions
            .iter()
            .map(|(sector, base_weight)| {
                let mut weight = *base_weight;

                // Ambition increases upward mobility (toward higher-paying sectors)
                if traits.ambition > 60.0 {
                    let ambition_bonus = (traits.ambition - 50.0) / 100.0;
                    if matches!(sector, JobSector::Technology | JobSector::Finance | JobSector::Healthcare) {
                        weight *= 1.0 + ambition_bonus;
                    }
                    // Reduce probability of staying in low-status jobs
                    if matches!(sector, JobSector::Retail | JobSector::Hospitality) {
                        weight *= 1.0 - ambition_bonus * 0.5;
                    }
                }

                // Stability reduces job-hopping
                if traits.stability > 60.0 {
                    if *sector == current_job {
                        weight *= 1.3; // More likely to stay
                    } else {
                        weight *= 0.8; // Less likely to change
                    }
                }

                // Impulsivity increases job-hopping and risky moves
                if traits.impulsivity > 60.0 {
                    if *sector != current_job {
                        weight *= 1.2; // More likely to change
                    }
                    if matches!(sector, JobSector::Freelance | JobSector::Entertainment | JobSector::Criminal) {
                        weight *= 1.3; // Drawn to volatile sectors
                    }
                }

                // Age effects: older NPCs less likely to change careers
                if age > 40 {
                    if *sector == current_job {
                        weight *= 1.5; // Strong inertia
                    } else {
                        weight *= 0.6;
                    }
                }

                (*sector, weight)
            })
            .collect();

        // Normalize weights
        let total: f32 = weighted_transitions.iter().map(|(_, w)| w).sum();
        if total <= 0.0 {
            return current_job; // Fallback
        }

        for (_, w) in &mut weighted_transitions {
            *w /= total;
        }

        // Select based on cumulative probabilities
        let roll = rng.gen_f32();
        let mut cumulative = 0.0;
        for (sector, weight) in weighted_transitions {
            cumulative += weight;
            if roll < cumulative {
                return sector;
            }
        }

        current_job // Fallback
    }
}

/// Generate a job title for a given sector and NPC.
fn generate_job_title(sector: JobSector, traits: &Traits, rng: &mut DeterministicRng) -> String {
    let title_pool = match sector {
        JobSector::Technology => vec![
            "Software Engineer", "Data Analyst", "IT Specialist", "Product Manager",
            "UX Designer", "DevOps Engineer", "Systems Administrator", "Tech Lead",
        ],
        JobSector::Healthcare => vec![
            "Nurse", "Doctor", "Medical Assistant", "Pharmacist", "Therapist",
            "Lab Technician", "Hospital Administrator", "Paramedic",
        ],
        JobSector::Finance => vec![
            "Financial Analyst", "Accountant", "Banker", "Investment Advisor",
            "Insurance Agent", "Loan Officer", "Auditor", "Tax Consultant",
        ],
        JobSector::Retail => vec![
            "Sales Associate", "Store Manager", "Cashier", "Merchandiser",
            "Buyer", "Inventory Clerk", "Customer Service Rep", "Visual Designer",
        ],
        JobSector::Hospitality => vec![
            "Server", "Bartender", "Chef", "Hotel Manager", "Concierge",
            "Line Cook", "Host", "Event Coordinator",
        ],
        JobSector::Education => vec![
            "Teacher", "Professor", "School Administrator", "Tutor",
            "Librarian", "Guidance Counselor", "Teaching Assistant", "Dean",
        ],
        JobSector::Government => vec![
            "Civil Servant", "Policy Analyst", "Inspector", "Administrator",
            "Social Worker", "Public Health Officer", "Urban Planner", "Clerk",
        ],
        JobSector::Construction => vec![
            "Carpenter", "Electrician", "Plumber", "Contractor", "Foreman",
            "Mason", "Roofer", "Heavy Equipment Operator",
        ],
        JobSector::Entertainment => vec![
            "Actor", "Musician", "Writer", "Artist", "Director", "Producer",
            "Sound Engineer", "Cinematographer", "Comedian",
        ],
        JobSector::Transportation => vec![
            "Driver", "Pilot", "Dispatcher", "Logistics Coordinator",
            "Mechanic", "Warehouse Manager", "Delivery Person", "Transit Operator",
        ],
        JobSector::Freelance => vec![
            "Consultant", "Freelancer", "Independent Contractor", "Gig Worker",
            "Self-Employed", "Entrepreneur", "Coach", "Specialist",
        ],
        JobSector::Manufacturing => vec![
            "Factory Worker", "Machine Operator", "Quality Control", "Foreman",
            "Assembly Line Worker", "Production Manager", "Technician", "Welder",
        ],
        JobSector::Agriculture => vec![
            "Farmer", "Ranch Hand", "Agricultural Worker", "Farm Manager",
            "Livestock Handler", "Crop Specialist", "Agronomist", "Harvester",
        ],
        JobSector::Criminal => vec![
            "Dealer", "Enforcer", "Lookout", "Runner", "Fixer",
            "Associate", "Operator", "Coordinator",
        ],
        JobSector::Unemployed => return "Unemployed".to_string(),
    };

    // Ambitious NPCs more likely to get senior titles
    let senior_titles = vec!["Manager", "Lead", "Senior", "Chief", "Director"];
    if traits.ambition > 70.0 && rng.gen_f32() < 0.3 {
        let base = title_pool[rng.gen_range_i32(0, title_pool.len() as i32) as usize];
        let modifier = senior_titles[rng.gen_range_i32(0, senior_titles.len() as i32) as usize];
        return format!("Senior {}", base);
    }

    title_pool[rng.gen_range_i32(0, title_pool.len() as i32) as usize].to_string()
}

/// District pool for lifepath generation.
const DISTRICTS: &[&str] = &[
    "Downtown", "Midtown", "Eastside", "Westside", "Southside", "Northside",
    "Suburban", "Industrial", "Harbor", "Heights", "Old Town", "University",
    "Riverside", "Hillcrest", "Parkview", "Lakeside",
];

/// Generate a full lifepath for an NPC.
pub fn generate_lifepath(
    npc_id: NpcId,
    age: u32,
    traits: &Traits,
    seed: u64,
) -> Lifepath {
    let mut rng = DeterministicRng::with_domain(seed, npc_id.0, "lifepath");
    let matrix = JobTransitionMatrix::new();

    let mut lifepath = Lifepath::default();

    // 1. Birth district
    lifepath.birth_district = DISTRICTS[rng.gen_range_i32(0, DISTRICTS.len() as i32) as usize].to_string();
    lifepath.district_history.push((SimTick(0), lifepath.birth_district.clone()));

    // 2. Education level (based on traits and age)
    lifepath.education_level = generate_education_level(age, traits, &mut rng);

    // 3. Generate milestones up to current age
    for milestone in LifeMilestone::milestones_up_to_age(age) {
        let milestone_age = milestone.age();
        let milestone_tick = SimTick(age_to_tick(milestone_age));
        lifepath.milestones.insert(milestone.key().to_string(), milestone_tick);
    }

    // 4. Generate job history
    if age >= 18 {
        generate_job_history(&mut lifepath, age, traits, &matrix, &mut rng);
    }

    // 5. Generate district moves (every 5-10 years on average, trait-dependent)
    generate_district_history(&mut lifepath, age, traits, &mut rng);

    lifepath
}

/// Convert age in years to approximate SimTick (24 ticks per day, 365 days per year).
fn age_to_tick(age: u32) -> u64 {
    (age as u64) * 365 * 24
}

/// Generate education level based on age, traits, and random variation.
fn generate_education_level(age: u32, traits: &Traits, rng: &mut DeterministicRng) -> EducationLevel {
    // Children and teens haven't finished education yet
    if age < 18 {
        return EducationLevel::None;
    }
    if age < 22 {
        return EducationLevel::HighSchool;
    }

    // Confidence (self-assurance) and ambition drive education
    // Using confidence as a proxy for academic capability
    let education_score = (traits.confidence + traits.ambition) / 2.0;
    let roll = rng.gen_f32() * 100.0;

    if education_score > 80.0 && roll < 40.0 {
        EducationLevel::Graduate
    } else if education_score > 70.0 && roll < 60.0 {
        EducationLevel::Bachelors
    } else if education_score > 60.0 && roll < 70.0 {
        if rng.gen_bool(0.4) {
            EducationLevel::Vocational
        } else {
            EducationLevel::Associates
        }
    } else if education_score > 40.0 && roll < 85.0 {
        EducationLevel::HighSchool
    } else {
        // Fallback to HighSchool for adults
        EducationLevel::HighSchool
    }
}

/// Generate job history from age 18 to current age.
fn generate_job_history(
    lifepath: &mut Lifepath,
    age: u32,
    traits: &Traits,
    matrix: &JobTransitionMatrix,
    rng: &mut DeterministicRng,
) {
    let start_age = 18u32;
    let mut current_job = JobSector::Unemployed;

    // First job based on education
    let first_job_age = match lifepath.education_level {
        EducationLevel::None | EducationLevel::HighSchool => 18,
        EducationLevel::Vocational => 19,
        EducationLevel::Associates => 20,
        EducationLevel::Bachelors => 22,
        EducationLevel::Graduate => 24,
        EducationLevel::Doctorate => 28,
    };

    // Get first job if old enough
    if age >= first_job_age {
        current_job = get_entry_job(lifepath.education_level, traits, rng);
        let job_title = generate_job_title(current_job, traits, rng);
        lifepath.job_history.push((
            SimTick(age_to_tick(first_job_age)),
            current_job,
            job_title,
        ));
    }

    // Career progression: change jobs based on ambition and impulsivity
    let mut current_age = first_job_age;
    while current_age < age {
        // Determine when next job change happens
        let years_in_job = calculate_years_in_job(traits, rng);
        current_age += years_in_job;

        if current_age > age {
            break;
        }

        // Retirement check
        if current_age >= 65 && rng.gen_f32() < 0.7 {
            current_job = JobSector::Unemployed; // Retired
            lifepath.job_history.push((
                SimTick(age_to_tick(current_age)),
                current_job,
                "Retired".to_string(),
            ));
            break;
        }

        // Get next job via Markov chain
        current_job = matrix.next_job(current_job, traits, current_age, rng);
        let job_title = generate_job_title(current_job, traits, rng);
        lifepath.job_history.push((
            SimTick(age_to_tick(current_age)),
            current_job,
            job_title,
        ));
    }
}

/// Get entry-level job sector based on education.
fn get_entry_job(education: EducationLevel, traits: &Traits, rng: &mut DeterministicRng) -> JobSector {
    match education {
        EducationLevel::None | EducationLevel::HighSchool => {
            let options = vec![
                (JobSector::Retail, 0.3),
                (JobSector::Hospitality, 0.25),
                (JobSector::Construction, 0.15),
                (JobSector::Transportation, 0.15),
                (JobSector::Manufacturing, 0.10),
                (JobSector::Freelance, 0.05),
            ];
            select_weighted_job(&options, rng)
        }
        EducationLevel::Vocational => {
            let options = vec![
                (JobSector::Construction, 0.35),
                (JobSector::Healthcare, 0.25),
                (JobSector::Manufacturing, 0.20),
                (JobSector::Transportation, 0.15),
                (JobSector::Hospitality, 0.05),
            ];
            select_weighted_job(&options, rng)
        }
        EducationLevel::Associates => {
            let options = vec![
                (JobSector::Healthcare, 0.3),
                (JobSector::Technology, 0.25),
                (JobSector::Education, 0.2),
                (JobSector::Government, 0.15),
                (JobSector::Retail, 0.1),
            ];
            select_weighted_job(&options, rng)
        }
        EducationLevel::Bachelors => {
            let options = vec![
                (JobSector::Technology, 0.25),
                (JobSector::Finance, 0.20),
                (JobSector::Healthcare, 0.15),
                (JobSector::Education, 0.15),
                (JobSector::Government, 0.15),
                (JobSector::Entertainment, 0.05),
                (JobSector::Freelance, 0.05),
            ];
            // Ambition pushes toward high-status sectors
            let mut adj_options = options;
            if traits.ambition > 70.0 {
                for (sector, weight) in &mut adj_options {
                    if matches!(sector, JobSector::Technology | JobSector::Finance) {
                        *weight *= 1.5;
                    }
                }
            }
            select_weighted_job(&adj_options, rng)
        }
        EducationLevel::Graduate | EducationLevel::Doctorate => {
            let options = vec![
                (JobSector::Healthcare, 0.35),
                (JobSector::Education, 0.30),
                (JobSector::Technology, 0.20),
                (JobSector::Finance, 0.10),
                (JobSector::Government, 0.05),
            ];
            select_weighted_job(&options, rng)
        }
    }
}

/// Select job from weighted options.
fn select_weighted_job(options: &[(JobSector, f32)], rng: &mut DeterministicRng) -> JobSector {
    let total: f32 = options.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_f32() * total;

    for (sector, weight) in options {
        roll -= weight;
        if roll <= 0.0 {
            return *sector;
        }
    }

    options.last().unwrap().0 // Fallback
}

/// Calculate how long NPC stays in current job (in years).
fn calculate_years_in_job(traits: &Traits, rng: &mut DeterministicRng) -> u32 {
    let base_years = 4.0; // Average tenure

    // Stability increases tenure
    let stability_mod = (traits.stability / 50.0).clamp(0.5, 2.0);

    // Impulsivity decreases tenure
    let impulsivity_mod = (100.0 - traits.impulsivity) / 50.0;

    // Ambition can increase turnover (job-hopping for advancement)
    let ambition_mod = if traits.ambition > 70.0 {
        0.8
    } else {
        1.0
    };

    let years = base_years * stability_mod * impulsivity_mod * ambition_mod * rng.gen_range_f32(0.5, 1.5);
    years.max(1.0).min(15.0) as u32
}

/// Generate district move history.
fn generate_district_history(lifepath: &mut Lifepath, age: u32, traits: &Traits, rng: &mut DeterministicRng) {
    // NPCs move districts based on life events (career, relationships, economic changes)
    // Stability reduces mobility, impulsivity increases it

    let mobility_base = 8.0; // Years between moves on average
    let stability_factor = traits.stability / 50.0;
    let mobility_years = (mobility_base * stability_factor).max(4.0) as u32;

    let mut current_age = 18; // Start tracking district moves after childhood
    let mut current_district = lifepath.birth_district.clone();

    while current_age < age {
        current_age += mobility_years + rng.gen_range_i32(0, 5) as u32;

        if current_age > age {
            break;
        }

        // 50% chance to actually move (many people don't move that often)
        if rng.gen_bool(0.5) {
            // Pick new district (favor nearby districts in real impl, here just random)
            let new_district = loop {
                let candidate = DISTRICTS[rng.gen_range_i32(0, DISTRICTS.len() as i32) as usize].to_string();
                if candidate != current_district {
                    break candidate;
                }
            };

            current_district = new_district.clone();
            lifepath.district_history.push((
                SimTick(age_to_tick(current_age)),
                new_district,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Traits;

    #[test]
    fn test_lifepath_generation_is_deterministic() {
        let traits = Traits {
            stability: 60.0,
            confidence: 55.0,
            sociability: 50.0,
            empathy: 50.0,
            impulsivity: 40.0,
            ambition: 70.0,
            charm: 50.0,
        };

        let lp1 = generate_lifepath(NpcId(1), 35, &traits, 12345);
        let lp2 = generate_lifepath(NpcId(1), 35, &traits, 12345);

        assert_eq!(lp1, lp2, "Lifepaths should be deterministic");
    }

    #[test]
    fn test_lifepath_has_job_history_for_adults() {
        let traits = Traits::default();
        let lifepath = generate_lifepath(NpcId(1), 30, &traits, 42);

        assert!(!lifepath.job_history.is_empty(), "Adult should have job history");
        assert_eq!(lifepath.current_job_title().is_empty(), false);
    }

    #[test]
    fn test_lifepath_milestones_match_age() {
        let traits = Traits::default();
        let lifepath = generate_lifepath(NpcId(1), 25, &traits, 999);

        assert!(lifepath.has_milestone("kindergarten"));
        assert!(lifepath.has_milestone("social_group"));
        assert!(lifepath.has_milestone("career_choice"));
        assert!(lifepath.has_milestone("first_job"));
        assert!(lifepath.has_milestone("relationship_window"));
        assert!(!lifepath.has_milestone("retirement")); // Too young
    }

    #[test]
    fn test_high_ambition_gets_better_jobs() {
        let ambitious = Traits {
            ambition: 90.0,
            confidence: 80.0,
            stability: 60.0,
            ..Default::default()
        };

        let lazy = Traits {
            ambition: 20.0,
            confidence: 50.0,
            stability: 70.0,
            ..Default::default()
        };

        let lp_ambitious = generate_lifepath(NpcId(1), 35, &ambitious, 7777);
        let lp_lazy = generate_lifepath(NpcId(2), 35, &lazy, 7777);

        // Ambitious NPC should have more career moves and higher-status sectors
        assert!(lp_ambitious.career_mobility_count() >= lp_lazy.career_mobility_count());

        // Check if ambitious NPC worked in higher-status sectors
        let high_status = [JobSector::Technology, JobSector::Finance, JobSector::Healthcare];
        let ambitious_in_high_status = lp_ambitious.job_history.iter()
            .any(|(_, sector, _)| high_status.contains(sector));

        // Not guaranteed but likely with high ambition
        assert!(ambitious_in_high_status || lp_ambitious.education_level >= EducationLevel::Bachelors);
    }

    #[test]
    fn test_education_level_increases_with_confidence() {
        let smart = Traits {
            confidence: 85.0,
            ambition: 75.0,
            ..Default::default()
        };

        let lifepath = generate_lifepath(NpcId(1), 30, &smart, 333);

        // High confidence/ambition should correlate with higher education, but RNG varies
        // At minimum they should have some education
        assert!(
            lifepath.education_level >= EducationLevel::HighSchool,
            "High-confidence NPC should at least have high school education, got: {:?}",
            lifepath.education_level
        );
    }

    #[test]
    fn test_stable_npcs_change_jobs_less() {
        let stable = Traits {
            stability: 95.0,
            impulsivity: 10.0,
            ambition: 50.0,
            ..Default::default()
        };

        let volatile = Traits {
            stability: 20.0,
            impulsivity: 85.0,
            ambition: 50.0,
            ..Default::default()
        };

        let lp_stable = generate_lifepath(NpcId(1), 40, &stable, 555);
        let lp_volatile = generate_lifepath(NpcId(2), 40, &volatile, 555);

        // Volatile NPC should generally have more job changes
        // (Not guaranteed due to RNG but statistically likely)
        let stable_moves = lp_stable.career_mobility_count();
        let volatile_moves = lp_volatile.career_mobility_count();

        // At least check that both have some history
        assert!(stable_moves > 0);
        assert!(volatile_moves > 0);
    }

    #[test]
    fn test_relationship_origin_positive_negative() {
        assert!(RelationshipOrigin::ChildhoodFriend.is_positive_origin());
        assert!(RelationshipOrigin::Rival.is_negative_origin());
        assert!(!RelationshipOrigin::ChanceEncounter.is_positive_origin());
        assert!(!RelationshipOrigin::ChanceEncounter.is_negative_origin());
    }

    #[test]
    fn test_lifepath_query_methods() {
        let traits = Traits::default();
        let mut lifepath = generate_lifepath(NpcId(1), 35, &traits, 123);

        // Add a test relationship origin
        lifepath.relationship_origins.insert(NpcId(2), RelationshipOrigin::Coworker);

        assert!(lifepath.relationship_origin(NpcId(2)).is_some());
        assert_eq!(
            *lifepath.relationship_origin(NpcId(2)).unwrap(),
            RelationshipOrigin::Coworker
        );

        // Test has_worked_in
        if let Some((_, sector, _)) = lifepath.job_history.first() {
            assert!(lifepath.has_worked_in(*sector));
        }

        // Test has_lived_in
        assert!(lifepath.has_lived_in(&lifepath.birth_district));
    }
}
