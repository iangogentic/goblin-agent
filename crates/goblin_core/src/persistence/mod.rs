//! Persistence Layer
//!
//! JSON file-backed storage for Goblin's brain features:
//! - Memory entries
//! - Skills
//! - DOJO reports
//! - User preferences

use crate::dojo::{Commit, ReviewReport};
use crate::memory::MemoryEntry;
use crate::skills::Skill;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// JSON file-backed persistence
#[derive(Clone)]
pub struct Persistence {
    data_dir: PathBuf,
    cache: Arc<RwLock<Cache>>,
}

struct Cache {
    memories: HashMap<String, MemoryEntry>,
    skills: HashMap<String, Skill>,
}

impl Cache {
    fn new() -> Self {
        Self {
            memories: HashMap::new(),
            skills: HashMap::new(),
        }
    }
}

use std::sync::Arc;

impl Persistence {
    /// Create a new persistence layer
    pub async fn new(home_dir: &PathBuf) -> Result<Self> {
        let goblin_dir = home_dir.join(".goblin");
        let data_dir = goblin_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let persistence = Self {
            data_dir,
            cache: Arc::new(RwLock::new(Cache::new())),
        };

        // Load existing data
        persistence.load_all().await?;

        Ok(persistence)
    }

    /// Get the global memory directory
    pub fn global_memory_dir(&self) -> PathBuf {
        self.data_dir.parent().unwrap().join("memory")
    }

    /// Load all data from disk
    async fn load_all(&self) -> Result<()> {
        let mut cache = self.cache.write().await;

        // Load memories
        let memories_path = self.data_dir.join("memories.json");
        if memories_path.exists() {
            let content = fs::read_to_string(&memories_path)?;
            let memories: Vec<MemoryEntry> = serde_json::from_str(&content).unwrap_or_default();
            for entry in memories {
                cache.memories.insert(entry.id.clone(), entry);
            }
        }

        // Load skills
        let skills_path = self.data_dir.join("skills.json");
        if skills_path.exists() {
            let content = fs::read_to_string(&skills_path)?;
            let skills: Vec<Skill> = serde_json::from_str(&content).unwrap_or_default();
            for skill in skills {
                cache.skills.insert(skill.id.clone(), skill);
            }
        }

        Ok(())
    }

    /// Save all data to disk
    async fn save_all(&self) -> Result<()> {
        let cache = self.cache.read().await;

        // Save memories
        let memories: Vec<&MemoryEntry> = cache.memories.values().collect();
        let content = serde_json::to_string_pretty(&memories)?;
        fs::write(self.data_dir.join("memories.json"), content)?;

        // Save skills
        let skills: Vec<&Skill> = cache.skills.values().collect();
        let content = serde_json::to_string_pretty(&skills)?;
        fs::write(self.data_dir.join("skills.json"), content)?;

        Ok(())
    }

    // === Memory Operations ===

    pub async fn save_memory(&self, entry: &MemoryEntry) -> Result<()> {
        {
            let mut cache = self.cache.write().await;
            cache.memories.insert(entry.id.clone(), entry.clone());
        }
        self.save_all().await
    }

    pub async fn get_memories(&self, _query: &crate::memory::MemoryQuery) -> Result<Vec<MemoryEntry>> {
        let cache = self.cache.read().await;
        let mut entries: Vec<MemoryEntry> = cache.memories.values().cloned().collect();
        entries.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        Ok(entries)
    }

    // === Skill Operations ===

    pub async fn save_skill(&self, skill: &Skill) -> Result<()> {
        {
            let mut cache = self.cache.write().await;
            cache.skills.insert(skill.id.clone(), skill.clone());
        }
        self.save_all().await
    }

    pub async fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let cache = self.cache.read().await;
        let mut skills: Vec<Skill> = cache.skills.values().cloned().collect();
        skills.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());
        Ok(skills)
    }

    pub async fn get_skill(&self, name: &str) -> Result<Option<Skill>> {
        let cache = self.cache.read().await;
        let skill = cache.skills.values().find(|s| s.name == name).cloned();
        Ok(skill)
    }

    pub async fn get_skill_by_id(&self, id: &str) -> Result<Option<Skill>> {
        let cache = self.cache.read().await;
        Ok(cache.skills.get(id).cloned())
    }

    // === DOJO Operations ===

    pub async fn save_review_report(&self, report: &ReviewReport) -> Result<()> {
        let reports_dir = self.data_dir.join("dojo");
        fs::create_dir_all(&reports_dir)?;

        let filename = format!(
            "report_{}.json",
            report.reviewed_at.format("%Y%m%d_%H%M%S")
        );
        let content = serde_json::to_string_pretty(report)?;
        fs::write(reports_dir.join(filename), content)?;
        Ok(())
    }

    pub async fn get_recent_commits(&self, _count: usize) -> Result<Vec<Commit>> {
        // For now, return empty - would need git integration
        // TODO: Parse git log
        Ok(Vec::new())
    }

    pub async fn append_rule(&self, rule: &str, description: &str) -> Result<()> {
        let rules_path = self.global_memory_dir().join("RULES.md");
        fs::create_dir_all(rules_path.parent().unwrap())?;

        let content = format!(
            "\n## {}\n{}\n",
            rule,
            description
        );

        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rules_path)?
            .write_all(content.as_bytes())?;

        Ok(())
    }
}
