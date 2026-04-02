//! Skills System
//!
//! Goblin creates and improves skills from experience. Skills are prompt templates
//! that capture patterns and solutions for reuse.

use crate::persistence::Persistence;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A skill is a reusable prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_template: String,
    pub usage_count: u64,
    pub quality_score: f64, // 0.0 - 1.0
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Skill {
    pub fn new(name: String, description: String, prompt_template: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            prompt_template,
            usage_count: 0,
            quality_score: 0.5, // Start neutral
            created_at: now,
            updated_at: now,
        }
    }

    /// Update quality based on feedback
    pub fn update_quality(&mut self, success: bool) {
        let delta = if success { 0.1 } else { -0.1 };
        self.quality_score = (self.quality_score + delta).clamp(0.0, 1.0);
        self.usage_count += 1;
        self.updated_at = chrono::Utc::now();
    }

    /// Render the skill template with variables
    pub fn render(&self, variables: &std::collections::HashMap<String, String>) -> String {
        let mut result = self.prompt_template.clone();
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

/// The skills registry
pub struct Skills {
    persistence: Persistence,
}

impl Skills {
    pub fn new(persistence: Persistence) -> Self {
        Self { persistence }
    }

    /// Create a new skill from experience
    pub async fn create(&self, skill: Skill) -> Result<()> {
        self.persistence.save_skill(&skill).await
    }

    /// List all skills
    pub async fn list(&self) -> Result<Vec<Skill>> {
        self.persistence.get_all_skills().await
    }

    /// Get a skill by name
    pub async fn get(&self, name: &str) -> Result<Option<Skill>> {
        self.persistence.get_skill(name).await
    }

    /// Update a skill's quality score
    pub async fn record_usage(&self, skill_id: &str, success: bool) -> Result<()> {
        if let Some(mut skill) = self.persistence.get_skill_by_id(skill_id).await? {
            skill.update_quality(success);
            self.persistence.save_skill(&skill).await?;
        }
        Ok(())
    }

    /// Suggest relevant skills based on context
    pub async fn suggest(&self, context: &str, limit: usize) -> Result<Vec<Skill>> {
        let all = self.list().await?;
        
        // Simple keyword matching for now
        // TODO: Use embeddings for semantic matching
        let scored: Vec<_> = all
            .into_iter()
            .map(|s| {
                let score = s.name.to_lowercase().contains(&context.to_lowercase()) as u64 as f64
                    + s.description.to_lowercase().contains(&context.to_lowercase()) as u64 as f64 * 0.5
                    + s.quality_score;
                (s, score)
            })
            .collect();
        
        let mut sorted = scored;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(sorted.into_iter().take(limit).map(|(s, _)| s).collect())
    }

    /// Create skill from experience (the magic part!)
    pub async fn create_from_experience(
        &self,
        task: &str,
        solution: &str,
        context: &str,
    ) -> Result<Skill> {
        let name = format!("skill/{}", slugify(task));
        let description = format!("Learned from solving: {}", task);
        
        let prompt_template = format!(
            r#"## Task: {}

## Context:
{}

## Solution Pattern:
{}

## Usage:
Use this skill when you encounter a similar task."#,
            task, context, solution
        );
        
        let skill = Skill::new(name, description, prompt_template);
        self.create(skill.clone()).await?;
        
        Ok(skill)
    }
}

/// Convert a string to a URL-safe slug
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect::<String>()
        .split_whitespace()
        .take(5)
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Fix memory leak in Rust"), "fix-memory-leak-in-rust");
    }

    #[test]
    fn test_skill_quality_update() {
        let mut skill = Skill::new(
            "test".to_string(),
            "Test skill".to_string(),
            "{{prompt}}".to_string(),
        );
        
        assert_eq!(skill.quality_score, 0.5);
        
        skill.update_quality(true);
        assert_eq!(skill.quality_score, 0.6);
        
        skill.update_quality(false);
        assert_eq!(skill.quality_score, 0.5);
    }
}
