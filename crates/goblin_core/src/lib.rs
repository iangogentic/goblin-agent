//! Goblin Core - The brain of Goblin
//!
//! This crate contains the core intelligence features:
//! - **Memory System**: Tiered, persistent memory across sessions
//! - **Skills System**: Dynamic skill creation and improvement
//! - **DOJO**: Self-improvement loop with automated review
//! - **Persistence**: JSON file-backed storage

use std::path::{Path, PathBuf};
use anyhow::Result;

pub mod memory;
pub mod skills;
pub mod dojo;
pub mod persistence;

// ============================================================================
// Tool Input/Output Types (used by goblin_app)
// ============================================================================

/// Input for memory checkpoint operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryCheckpointInput {
    pub label: String,
}

/// Output for memory checkpoint operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryCheckpointOutput {
    pub checkpoint_id: String,
    pub timestamp: i64,
    pub message: String,
}

/// Input for memory compact operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryCompactInput {
    pub max_entries: Option<usize>,
}

/// Output for memory compact operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryCompactOutput {
    pub entries_compacted: usize,
    pub new_size_bytes: usize,
    pub message: String,
}

/// Input for memory search operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySearchInput {
    pub query: String,
    pub scope: Option<String>,
}

/// Output for memory search operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySearchOutput {
    pub results: Vec<MemorySearchResult>,
    pub total: usize,
    pub message: String,
}

/// A single memory search result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySearchResult {
    pub key: String,
    pub value: String,
    pub score: f32,
    pub scope: String,
}

/// Input for memory summarize operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySummarizeInput {
    pub scope: Option<String>,
}

/// Output for memory summarize operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySummarizeOutput {
    pub summary: String,
    pub tokens_saved: usize,
}

/// Input for skill creation operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillCreateInput {
    pub name: String,
    pub prompt: Option<String>,
}

/// Output for skill creation operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillCreateOutput {
    pub skill_id: String,
    pub name: String,
    pub message: String,
}

/// Input for skill improvement operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillImproveInput {
    pub name: String,
    pub success: bool,
}

/// Output for skill improvement operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillImproveOutput {
    pub name: String,
    pub quality_delta: f32,
    pub message: String,
}

/// Output for skill list operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillListOutput {
    pub skills: Vec<SkillInfo>,
    pub total: usize,
}

/// Information about a skill
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub quality_score: f32,
    pub usage_count: usize,
}

/// Input for schedule creation operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleCreateInput {
    pub prompt: String,
    pub schedule: Option<String>,
}

/// Output for schedule creation operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleCreateOutput {
    pub job_id: String,
    pub next_run: Option<i64>,
    pub message: String,
}

/// Input for schedule cancellation operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleCancelInput {
    pub job_id: String,
}

/// Output for schedule cancellation operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleCancelOutput {
    pub success: bool,
    pub message: String,
}

/// Output for schedule list operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleListOutput {
    pub jobs: Vec<ScheduledJob>,
    pub total: usize,
}

/// Information about a scheduled job
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub prompt: String,
    pub schedule: String,
    pub next_run: Option<i64>,
    pub last_run: Option<i64>,
}

/// Initialize Goblin Core with all subsystems
pub async fn initialize(home_dir: &Path) -> Result<Core> {
    let home: PathBuf = home_dir.into();
    let persistence = persistence::Persistence::new(&home).await?;
    let memory = memory::Memory::new(persistence.clone());
    let skills = skills::Skills::new(persistence.clone());
    let dojo = dojo::Dojo::new(persistence.clone());

    Ok(Core {
        memory,
        skills,
        dojo,
    })
}

/// The main Goblin Core instance
pub struct Core {
    pub memory: memory::Memory,
    pub skills: skills::Skills,
    pub dojo: dojo::Dojo,
}

impl Core {
    /// Run a self-improvement cycle
    pub async fn improve(&self) -> Result<dojo::ReviewReport> {
        self.dojo.review().await
    }
}
