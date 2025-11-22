use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use syn_core::{Persistence, StoryletRecord};
use syn_director::Storylet;
pub mod storylet;
pub use storylet::*;

/// Load all storylets stored inside the SQLite database at `db_path`.
pub fn load_storylets_from_db(db_path: &str) -> Result<Vec<Storylet>> {
    let mut persistence = Persistence::new(db_path)?;
    let records = persistence.load_storylet_records()?;
    let mut storylets = Vec::new();
    for record in records {
        let storylet: Storylet = serde_json::from_str(&record.json_data)?;
        storylets.push(storylet);
    }
    Ok(storylets)
}

/// Import every JSON storylet inside `directory` into the SQLite database, overwriting existing entries.
pub fn import_storylets_from_dir(db_path: &str, directory: &Path) -> Result<usize> {
    let mut persistence = Persistence::new(db_path)?;
    let mut imported = 0;
    for path in iter_json_files(directory)? {
        let data = std::fs::read_to_string(&path)?;
        let storylet: Storylet = serde_json::from_str(&data)?;
        let json_data = serde_json::to_string_pretty(&storylet)?;
        let record = StoryletRecord {
            id: storylet.id.clone(),
            name: storylet.name.clone(),
            json_data,
        };
        persistence.upsert_storylet_record(&record)?;
        imported += 1;
    }
    Ok(imported)
}

/// Helper for tooling: structure stored alongside JSON for validation output.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoryletSummary {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
}

/// Preview all storylets in a directory without touching the database.
pub fn summarize_storylets(directory: &Path) -> Result<Vec<StoryletSummary>> {
    let mut summaries = Vec::new();
    for path in iter_json_files(directory)? {
        let data = std::fs::read_to_string(&path)?;
        let storylet: Storylet = serde_json::from_str(&data)?;
        summaries.push(StoryletSummary {
            id: storylet.id.clone(),
            name: storylet.name.clone(),
            tags: storylet.tags.clone(),
        });
    }
    Ok(summaries)
}

fn iter_json_files(directory: &Path) -> Result<Vec<PathBuf>> {
    fn recurse(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                recurse(&path, files)?;
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    if directory.exists() {
        recurse(directory, &mut files)?;
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};
    use syn_core::RelationshipState;
    use syn_director::StoryletPrerequisites;

    fn sample_storylet() -> Storylet {
        Storylet {
            id: "test_storylet".to_string(),
            name: "Test".to_string(),
            tags: vec!["test".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: Some(1.0),
                min_relationship_resentment: None,
                stat_conditions: Default::default(),
                life_stages: vec!["Adult".to_string()],
                tags: vec![],
                relationship_states: vec![RelationshipState::Friend],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 40.0,
            weight: 0.5,
            cooldown_ticks: 24,
            roles: vec![],
        }
    }

    #[test]
    fn test_import_and_load_storylets() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let temp_base = std::env::temp_dir().join(format!("syn_storylet_test_{}", unique));
        fs::create_dir_all(&temp_base).unwrap();
        let db_path = temp_base.join("storylets.sqlite");
        let json_dir = temp_base.join("json");
        fs::create_dir(&json_dir).unwrap();

        let storylet = sample_storylet();
        let storylet_path = json_dir.join("test_storylet.json");
        fs::write(&storylet_path, serde_json::to_string_pretty(&storylet).unwrap()).unwrap();

        let imported = import_storylets_from_dir(db_path.to_str().unwrap(), &json_dir).unwrap();
        assert_eq!(imported, 1);

        let loaded = load_storylets_from_db(db_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test_storylet");

        let _ = fs::remove_dir_all(temp_base);
    }
}
