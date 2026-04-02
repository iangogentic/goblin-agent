//! Goblin Skills Hub Client
//!
//! Connects to the skills marketplace (agentskills.io or custom).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Skills Hub client
pub struct SkillsHubClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl SkillsHubClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    /// Search for skills
    pub async fn search(&self, query: &str) -> Result<Vec<SkillInfo>> {
        let url = format!("{}/api/skills/search", self.base_url);
        
        let mut request = self.client.get(&url).query(&[("q", query)]);
        
        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request.send().await?;
        let skills = response.json::<Vec<SkillInfo>>().await?;
        
        Ok(skills)
    }

    /// Get skill by ID
    pub async fn get(&self, skill_id: &str) -> Result<SkillInfo> {
        let url = format!("{}/api/skills/{}", self.base_url, skill_id);
        
        let mut request = self.client.get(&url);
        
        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request.send().await?;
        let skill = response.json::<SkillInfo>().await?;
        
        Ok(skill)
    }

    /// Download skill
    pub async fn download(&self, skill_id: &str) -> Result<SkillContent> {
        let url = format!("{}/api/skills/{}/download", self.base_url, skill_id);
        
        let mut request = self.client.get(&url);
        
        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request.send().await?;
        let content = response.json::<SkillContent>().await?;
        
        Ok(content)
    }

    /// Upload skill
    pub async fn publish(&self, skill: &SkillContent) -> Result<SkillInfo> {
        let url = format!("{}/api/skills/publish", self.base_url);
        
        let mut request = self.client.post(&url).json(skill);
        
        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request.send().await?;
        let info = response.json::<SkillInfo>().await?;
        
        Ok(info)
    }

    /// List popular skills
    pub async fn popular(&self) -> Result<Vec<SkillInfo>> {
        let url = format!("{}/api/skills/popular", self.base_url);
        
        let response = self.client.get(&url).send().await?;
        let skills = response.json::<Vec<SkillInfo>>().await?;
        
        Ok(skills)
    }

    /// List skills by category
    pub async fn by_category(&self, category: &str) -> Result<Vec<SkillInfo>> {
        let url = format!("{}/api/skills/category/{}", self.base_url, category);
        
        let response = self.client.get(&url).send().await?;
        let skills = response.json::<Vec<SkillInfo>>().await?;
        
        Ok(skills)
    }
}

/// Skill info (metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Unique ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Author
    pub author: String,
    /// Version
    pub version: String,
    /// Downloads count
    pub downloads: u64,
    /// Rating
    pub rating: f64,
    /// Tags
    pub tags: Vec<String>,
    /// Created at
    pub created_at: String,
    /// Updated at
    pub updated_at: String,
}

/// Skill content (full skill definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContent {
    /// Skill metadata
    pub info: SkillInfo,
    /// Template content
    pub template: String,
    /// Instructions
    pub instructions: String,
    /// Examples
    pub examples: Vec<String>,
    /// Metadata
    pub metadata: SkillMetadata,
}

/// Skill metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Language
    pub language: String,
    /// Framework
    pub framework: Option<String>,
    /// Complexity
    pub complexity: SkillComplexity,
    /// Use cases
    pub use_cases: Vec<String>,
}

/// Skill complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillComplexity {
    Beginner,
    Intermediate,
    Advanced,
}

/// Skills directory manager
pub struct SkillsDirectory {
    skills_dir: PathBuf,
}

impl SkillsDirectory {
    pub fn new(skills_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&skills_dir)?;
        Ok(Self { skills_dir })
    }

    /// List all locally installed skills
    pub fn list(&self) -> Result<Vec<SkillInfo>> {
        let mut skills = Vec::new();
        
        for entry in std::fs::read_dir(&self.skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "json" || ext == "yaml" || ext == "yml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(skill) = serde_yaml::from_str::<SkillContent>(&content)
                        .ok()
                        .or_else(|| serde_json::from_str(&content).ok())
                    {
                        skills.push(skill.info);
                    }
                }
            }
        }
        
        Ok(skills)
    }

    /// Save skill locally
    pub fn save(&self, skill: &SkillContent) -> Result<PathBuf> {
        let path = self.skills_dir.join(format!("{}.yaml", skill.info.name));
        let content = serde_yaml::to_string(skill)?;
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Delete skill
    pub fn delete(&self, name: &str) -> Result<()> {
        let path = self.skills_dir.join(format!("{}.yaml", name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Get skill content
    pub fn get(&self, name: &str) -> Result<Option<SkillContent>> {
        let path = self.skills_dir.join(format!("{}.yaml", name));
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = std::fs::read_to_string(&path)?;
        let skill = serde_yaml::from_str::<SkillContent>(&content)
            .ok()
            .or_else(|| serde_json::from_str(&content).ok())
            .context("Failed to parse skill file")?;
        
        Ok(Some(skill))
    }
}
