use serde::Deserialize;
use std::path::Path;

use crate::{
    storylet_library::tags_to_bitset, Storylet, StoryletCooldown, StoryletOutcomeSet,
    StoryletPrerequisites, StoryletRole, StoryletRoles, StoryletTrigger,
};

#[derive(Debug, Deserialize)]
pub(crate) struct StoryletSerde {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub prerequisites: StoryletPrerequisites,
    #[serde(default)]
    pub roles: Vec<StoryletRole>,
    pub heat: i32,
    #[serde(default)]
    pub triggers: StoryletTrigger,
    #[serde(default)]
    pub outcomes: StoryletOutcomeSet,
    #[serde(default)]
    pub cooldown: StoryletCooldown,
    pub weight: f32,
}

impl From<StoryletSerde> for Storylet {
    fn from(src: StoryletSerde) -> Self {
        let mut storylet = Storylet::new(
            src.id,
            tags_to_bitset(&src.tags),
            src.prerequisites,
            StoryletRoles::from(src.roles),
            src.heat,
            src.triggers,
            src.outcomes,
            src.cooldown,
            src.weight,
        );
        storylet.name = src.name;
        storylet
    }
}

pub fn parse_storylet_str(raw: &str) -> Result<Storylet, serde_json::Error> {
    let intermediate: StoryletSerde = serde_json::from_str(raw)?;
    Ok(intermediate.into())
}

/// Load the compiled storylet library from the binary file.
/// 
/// This function loads from `rust/syn_director/data/storylets.bin` which is
/// generated at build time by the storyletc compiler.
pub fn load_compiled_library() -> Result<syn_storylets::library::StoryletLibrary, String> {
    let binary_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("storylets.bin");
    
    if !binary_path.exists() {
        return Err(format!(
            "Compiled storylet library not found at {:?}. Run build to compile storylets.",
            binary_path
        ));
    }
    
    syn_storylets::library::StoryletLibrary::read_from_file(&binary_path)
        .map_err(|e| format!("Failed to load compiled storylet library: {:?}", e))
}
