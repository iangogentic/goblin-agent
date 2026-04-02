//! DOJO - Self-Improvement Loop
//!
//! Daily review sessions where Goblin checks its own work:
//! - Did I follow my own rules?
//! - Any commits without proper documentation?
//! - Ops-heavy vs code-heavy balance?
//!
//! Goblin patches its own rules when it finds patterns to fix.

use crate::persistence::Persistence;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A review finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: FindingSeverity,
    pub rule: String,
    pub description: String,
    pub suggestion: String,
}

/// Finding severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Warning,
    Critical,
}

/// A correction rule to add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub rule: String,
    pub description: String,
    pub source: String,
}

/// The DOJO review report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub findings: Vec<Finding>,
    pub corrections: Vec<Correction>,
    pub metrics: Metrics,
    pub reviewed_at: chrono::DateTime<chrono::Utc>,
}

/// Metrics from the review period
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metrics {
    pub commits_count: u64,
    pub code_commits: u64,
    pub ops_commits: u64,
    pub avg_commit_size: f64,
    pub consecutive_ops: u64,
}

/// The DOJO self-review system
pub struct Dojo {
    persistence: Persistence,
}

impl Dojo {
    pub fn new(persistence: Persistence) -> Self {
        Self { persistence }
    }

    /// Run a morning review
    pub async fn review(&self) -> Result<ReviewReport> {
        let mut findings = Vec::new();
        let mut corrections = Vec::new();

        // Get recent activity
        let metrics = self.compute_metrics().await?;
        
        // Check OPS/CODE balance
        if metrics.consecutive_ops >= 3 {
            findings.push(Finding {
                severity: FindingSeverity::Warning,
                rule: "OPS-CODE-BALANCE".to_string(),
                description: format!(
                    "{} consecutive ops-only commits without code commits",
                    metrics.consecutive_ops
                ),
                suggestion: "Time to write some code! Find a bug to fix or feature to implement.".to_string(),
            });
            
            corrections.push(Correction {
                rule: "OPS-CODE-BALANCE-RULE".to_string(),
                description: "After 3 consecutive ops commits, must do one code commit".to_string(),
                source: "DOJO correction".to_string(),
            });
        }

        // Check commit size
        if metrics.avg_commit_size > 500.0 {
            findings.push(Finding {
                severity: FindingSeverity::Info,
                rule: "COMMIT-SIZE".to_string(),
                description: format!(
                    "Average commit size is {} lines - consider smaller commits",
                    metrics.avg_commit_size as u64
                ),
                suggestion: "Break large changes into smaller, focused commits.".to_string(),
            });
        }

        let report = ReviewReport {
            findings,
            corrections,
            metrics,
            reviewed_at: chrono::Utc::now(),
        };

        // Save the report
        self.persistence.save_review_report(&report).await?;

        Ok(report)
    }

    /// Compute metrics from recent activity
    async fn compute_metrics(&self) -> Result<Metrics> {
        let commits = self.persistence.get_recent_commits(20).await?;
        
        let commits_count = commits.len() as u64;
        let (code_commits, ops_commits, avg_commit_size) = 
            commits.iter().fold((0u64, 0u64, 0u64), |acc, c| {
                let is_ops = c.is_ops();
                (
                    acc.0 + (!is_ops) as u64,
                    acc.1 + is_ops as u64,
                    acc.2 + c.additions as u64,
                )
            });
        
        let avg_commit_size = if commits_count > 0 {
            avg_commit_size as f64 / commits_count as f64
        } else {
            0.0
        };
        
        // Count consecutive ops commits
        let consecutive_ops = commits.iter()
            .take_while(|c| c.is_ops())
            .count() as u64;

        Ok(Metrics {
            commits_count,
            code_commits,
            ops_commits,
            avg_commit_size,
            consecutive_ops,
        })
    }

    /// Apply corrections to the rules
    pub async fn apply_corrections(&self, corrections: &[Correction]) -> Result<()> {
        for correction in corrections {
            self.persistence.append_rule(&correction.rule, &correction.description).await?;
        }
        Ok(())
    }
}

/// A commit for analysis
#[derive(Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub additions: i32,
    pub deletions: i32,
}

impl Commit {
    /// Heuristic: ops commits touch config/scripts/docs, not source code
    pub fn is_ops(&self) -> bool {
        let ops_patterns = ["README", "docs/", ".github/", "config", "CI", "docker", "script"];
        let code_patterns = [".rs", ".ts", ".js", ".py", ".go"];
        
        let msg_lower = self.message.to_lowercase();
        
        let is_ops = ops_patterns.iter().any(|p| msg_lower.contains(&p.to_lowercase()));
        let is_code = code_patterns.iter().any(|p| msg_lower.contains(p));
        
        is_ops && !is_code
    }
}
