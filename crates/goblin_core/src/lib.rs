//! Goblin Core - The brain of Goblin
//!
//! This crate contains the core intelligence features:
//! - **Memory System**: Tiered, persistent memory across sessions
//! - **Skills System**: Dynamic skill creation and improvement
//! - **DOJO**: Self-improvement loop with automated review
//! - **Persistence**: SQLite-backed storage

pub mod memory;
pub mod skills;
pub mod dojo;
pub mod persistence;

use anyhow::Result;

/// Initialize Goblin Core with all subsystems
pub async fn initialize(home_dir: &std::path::Path) -> Result<Core> {
    let persistence = persistence::Persistence::new(home_dir).await?;
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
