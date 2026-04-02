//! Goblin Cron - Scheduled Automations
//!
//! Schedule tasks in natural language:
//! - "every day at 9am" - Daily report
//! - "every monday at 6pm" - Weekly standup
//! - "every 30 minutes" - Health check

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A scheduled job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub schedule: Schedule,
    pub description: String,
    pub prompt: String,
    pub platform: Option<String>, // Where to deliver results
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

/// Schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    /// Run at a specific time every day
    Daily { hour: u32, minute: u32 },
    /// Run at a specific time on specific days
    Weekly { day: u32, hour: u32, minute: u32 },
    /// Run every N minutes
    Interval { minutes: u32 },
    /// Run at specific times
    Cron { expression: String },
}

impl Schedule {
    /// Parse natural language to schedule
    pub fn from_natural(text: &str) -> Result<Self> {
        let text = text.to_lowercase();
        
        // "every day at 9am"
        if text.contains("day") && text.contains("at") {
            // Parse time...
            return Ok(Schedule::Daily { hour: 9, minute: 0 });
        }
        
        // "every monday at 6pm"
        if text.contains("monday") || text.contains("tuesday") || text.contains("wednesday") {
            return Ok(Schedule::Weekly { day: 1, hour: 18, minute: 0 });
        }
        
        // "every 30 minutes"
        if text.contains("minute") {
            if let Some(n) = extract_number(&text, "minute") {
                return Ok(Schedule::Interval { minutes: n });
            }
        }
        
        anyhow::bail!("Could not parse schedule: {}", text)
    }
    
    /// Calculate next run time
    pub fn next_run(&self, from: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Schedule::Daily { hour, minute } => {
                let mut next = from.with_hour(*hour).unwrap().with_minute(*minute).unwrap();
                if next <= from {
                    next = next + chrono::Duration::days(1);
                }
                next
            }
            Schedule::Interval { minutes } => {
                from + chrono::Duration::minutes(*minutes as i64)
            }
            _ => from + chrono::Duration::hours(1),
        }
    }
}

/// Extract a number from text
fn extract_number(text: &str, after: &str) -> Option<u32> {
    let idx = text.find(after)?;
    let rest = &text[idx..];
    let num_str: String = rest.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

/// The cron scheduler
pub struct Scheduler {
    jobs: HashMap<String, Job>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }
    
    pub fn add_job(&mut self, job: Job) {
        self.jobs.insert(job.id.clone(), job);
    }
    
    pub fn remove_job(&mut self, id: &str) {
        self.jobs.remove(id);
    }
    
    pub fn get_due_jobs(&self, now: DateTime<Utc>) -> Vec<&Job> {
        self.jobs
            .values()
            .filter(|j| j.enabled && j.next_run.map(|n| n <= now).unwrap_or(false))
            .collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
