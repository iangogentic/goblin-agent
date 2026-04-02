//! Tiered Memory System
//!
//! Goblin remembers things across sessions using a tiered memory system:
//! - **Global**: ~/.goblin/memory/ - available to all projects
//! - **Project**: .goblin/memory/ - specific to a project
//! - **Session**: ephemeral during a session, synced to DB
//!
//! Memory entries are bootstrap files that get loaded into context:
//! - SOUL.md - Agent identity
//! - IDENTITY.md - Role and capabilities
//! - USER.md - User preferences
//! - MEMORY.md - Persistent learnings

use crate::persistence::Persistence;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Memory scope levels (most-specific wins)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryScope {
    Session = 0,
    Project = 1,
    Global = 2,
}

/// Bootstrap file names (loaded in priority order)
pub const BOOTSTRAP_FILES: &[&str] = &[
    "SOUL.md",
    "IDENTITY.md",
    "USER.md",
    "AGENTS.md",
    "TOOLS.md",
    "MEMORY.md",
];

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub scope: MemoryScope,
    pub key: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
}

impl MemoryEntry {
    pub fn new(scope: MemoryScope, key: String, content: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            scope,
            key,
            content,
            created_at: now,
            updated_at: now,
            access_count: 0,
        }
    }
}

/// Memory query
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub query: Option<String>,
    pub scope: Option<MemoryScope>,
    pub limit: usize,
}

/// The main memory system
pub struct Memory {
    persistence: Persistence,
    /// In-memory cache of loaded entries
    cache: HashMap<String, MemoryEntry>,
}

impl Memory {
    pub fn new(persistence: Persistence) -> Self {
        Self {
            persistence,
            cache: HashMap::new(),
        }
    }

    /// Store a memory entry
    pub async fn store(&mut self, entry: MemoryEntry) -> Result<()> {
        self.persistence.save_memory(&entry).await?;
        self.cache.insert(entry.key.clone(), entry);
        Ok(())
    }

    /// Search memories
    pub async fn search(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>> {
        let entries = self.persistence.get_memories(query).await?;
        Ok(entries)
    }

    /// Load bootstrap files for a given scope
    pub fn load_bootstrap_files(scope_dir: &PathBuf) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        for filename in BOOTSTRAP_FILES {
            let path = scope_dir.join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    results.push((filename.to_string(), content));
                }
            }
        }
        
        results
    }

    /// Resolve context for an agent, merging bootstrap files from all scopes
    pub async fn resolve_context(
        &self,
        project_dir: Option<&PathBuf>,
    ) -> Result<String> {
        let mut context_parts = Vec::new();

        // Load global memories
        let global_dir = self.persistence.global_memory_dir();
        let global_context = Self::load_bootstrap_files(&global_dir);
        for (name, content) in global_context {
            context_parts.push(format!("\n# {}\n{}\n", name, content));
        }

        // Load project memories (if applicable)
        if let Some(project) = project_dir {
            let project_dir = project.join(".goblin/memory");
            let project_context = Self::load_bootstrap_files(&project_dir);
            for (name, content) in project_context {
                context_parts.push(format!("\n# {}\n{}\n", name, content));
            }
        }

        Ok(context_parts.join("\n"))
    }

    /// Checkpoint current state
    pub async fn checkpoint(&self, label: &str) -> Result<()> {
        let checkpoint = MemoryEntry::new(
            MemoryScope::Global,
            format!("checkpoint/{}", label),
            format!("Checkpoint created at {}", chrono::Utc::now()),
        );
        self.persistence.save_memory(&checkpoint).await?;
        Ok(())
    }

    /// Compact memories for context budget
    pub async fn compact(&self, max_entries: usize) -> Result<Vec<MemoryEntry>> {
        let query = MemoryQuery {
            query: None,
            scope: None,
            limit: max_entries,
        };
        let entries = self.persistence.get_memories(&query).await?;
        
        // LLM summarization would happen here
        // For now, just return top entries by access count
        let mut sorted = entries;
        sorted.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        Ok(sorted.into_iter().take(max_entries).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry::new(
            MemoryScope::Global,
            "test".to_string(),
            "Test content".to_string(),
        );
        
        assert_eq!(entry.scope, MemoryScope::Global);
        assert_eq!(entry.key, "test");
        assert_eq!(entry.access_count, 0);
    }
}
