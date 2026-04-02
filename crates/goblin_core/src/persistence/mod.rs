//! Persistence Layer
//!
//! SQLite-backed storage for Goblin's brain features:
//! - Memory entries
//! - Skills
//! - DOJO reports
//! - User preferences

use crate::dojo::{Commit, Metrics, ReviewReport};
use crate::memory::MemoryEntry;
use crate::skills::Skill;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Database-backed persistence
#[derive(Clone)]
pub struct Persistence {
    conn: Arc<Mutex<Connection>>,
    home_dir: PathBuf,
}

impl Persistence {
    /// Create a new persistence layer
    pub async fn new(home_dir: &PathBuf) -> Result<Self> {
        // Ensure directories exist
        let goblin_dir = home_dir.join(".goblin");
        let db_dir = goblin_dir.join("data");
        std::fs::create_dir_all(&db_dir)?;
        
        let db_path = db_dir.join("goblin.db");
        let conn = Connection::open(&db_path)
            .context("Failed to open database")?;
        
        // Initialize schema
        conn.execute_batch(SCHEMA)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            home_dir: home_dir.clone(),
        });
    }
    
    /// Get the global memory directory
    pub fn global_memory_dir(&self) -> PathBuf {
        self.home_dir.join(".goblin/memory")
    }

    // === Memory Operations ===

    pub async fn save_memory(&self, entry: &MemoryEntry) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO memories (id, scope, key, content, created_at, updated_at, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id,
                entry.scope as i32,
                entry.key,
                entry.content,
                entry.created_at.timestamp(),
                entry.updated_at.timestamp(),
                entry.access_count,
            ],
        )?;
        Ok(())
    }

    pub async fn get_memories(&self, query: &crate::memory::MemoryQuery) -> Result<Vec<MemoryEntry>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, scope, key, content, created_at, updated_at, access_count 
             FROM memories ORDER BY access_count DESC LIMIT ?1"
        )?;
        
        let entries = stmt.query_map([query.limit], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                scope: crate::memory::MemoryScope::from_i32(row.get(1)?),
                key: row.get(2)?,
                content: row.get(3)?,
                created_at: chrono::DateTime::from_timestamp(row.get(4)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(row.get(5)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
                access_count: row.get(6)?,
            })
        })?;
        
        Ok(entries.filter_map(|e| e.ok()).collect())
    }

    // === Skill Operations ===

    pub async fn save_skill(&self, skill: &Skill) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO skills (id, name, description, prompt_template, usage_count, quality_score, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                skill.id,
                skill.name,
                skill.description,
                skill.prompt_template,
                skill.usage_count,
                skill.quality_score,
                skill.created_at.timestamp(),
                skill.updated_at.timestamp(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, prompt_template, usage_count, quality_score, created_at, updated_at
             FROM skills ORDER BY quality_score DESC"
        )?;
        
        let skills = stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                usage_count: row.get(4)?,
                quality_score: row.get(5)?,
                created_at: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(row.get(7)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
            })
        })?;
        
        Ok(skills.filter_map(|s| s.ok()).collect())
    }

    pub async fn get_skill(&self, name: &str) -> Result<Option<Skill>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, prompt_template, usage_count, quality_score, created_at, updated_at
             FROM skills WHERE name = ?1"
        )?;
        
        let skill = stmt.query_row([name], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                usage_count: row.get(4)?,
                quality_score: row.get(5)?,
                created_at: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(row.get(7)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
            })
        });
        
        Ok(skill.ok())
    }

    pub async fn get_skill_by_id(&self, id: &str) -> Result<Option<Skill>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, prompt_template, usage_count, quality_score, created_at, updated_at
             FROM skills WHERE id = ?1"
        )?;
        
        let skill = stmt.query_row([id], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt_template: row.get(3)?,
                usage_count: row.get(4)?,
                quality_score: row.get(5)?,
                created_at: chrono::DateTime::from_timestamp(row.get(6)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(row.get(7)?, 0)
                    .unwrap_or_else(chrono::Utc::now),
            })
        });
        
        Ok(skill.ok())
    }

    // === DOJO Operations ===

    pub async fn save_review_report(&self, report: &ReviewReport) -> Result<()> {
        let conn = self.conn.lock().await;
        let report_json = serde_json::to_string(report)?;
        conn.execute(
            "INSERT INTO dojo_reports (report, reviewed_at) VALUES (?1, ?2)",
            params![report_json, report.reviewed_at.timestamp()],
        )?;
        Ok(())
    }

    pub async fn get_recent_commits(&self, _count: usize) -> Result<Vec<Commit>> {
        // For now, return empty - would need git integration
        // TODO: Parse git log
        Ok(Vec::new())
    }

    pub async fn append_rule(&self, rule: &str, description: &str) -> Result<()> {
        let rules_path = self.home_dir.join(".goblin/memory/RULES.md");
        std::fs::create_dir_all(rules_path.parent().unwrap())?;
        
        let content = format!(
            "\n## {}\n{}\n",
            rule,
            description
        );
        
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rules_path)?
            .write_all(content.as_bytes())?;
        
        Ok(())
    }
}

/// Database schema
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY,
    scope INTEGER NOT NULL,
    key TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    prompt_template TEXT NOT NULL,
    usage_count INTEGER DEFAULT 0,
    quality_score REAL DEFAULT 0.5,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS dojo_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    report TEXT NOT NULL,
    reviewed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_memories_key ON memories(key);
CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope);
CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);
"#;

use crate::memory::MemoryScope;
use std::io::Write;

impl MemoryScope {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => MemoryScope::Session,
            1 => MemoryScope::Project,
            _ => MemoryScope::Global,
        }
    }
}
