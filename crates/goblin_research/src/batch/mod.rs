//! Batch Trajectory Generation
//!
//! Runs multiple agent trajectories in parallel for dataset collection.
//! Used for training new models or evaluating agent performance.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A single step in a trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    /// Step index
    pub step: usize,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Agent's observation (state)
    pub observation: String,
    /// Agent's action
    pub action: String,
    /// Environment's response
    pub response: String,
    /// Cumulative reward
    pub cumulative_reward: f32,
    /// Whether this step is terminal
    pub terminal: bool,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// A complete trajectory (episode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    /// Unique trajectory ID
    pub id: String,
    /// Task/prompt for this trajectory
    pub task: String,
    /// All steps in the trajectory
    pub steps: Vec<TrajectoryStep>,
    /// Final cumulative reward
    pub total_reward: f32,
    /// Whether the trajectory was successful
    pub success: bool,
    /// Number of tokens used
    pub tokens_used: usize,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Environment configuration
    pub environment: String,
    /// Model used
    pub model: String,
}

impl Trajectory {
    /// Create a new trajectory
    pub fn new(task: String, environment: String, model: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task,
            steps: Vec::new(),
            total_reward: 0.0,
            success: false,
            tokens_used: 0,
            start_time: Utc::now(),
            end_time: Utc::now(),
            environment,
            model,
        }
    }

    /// Add a step to the trajectory
    pub fn add_step(&mut self, observation: String, action: String, response: String, reward: f32, terminal: bool) {
        self.total_reward += reward;
        self.steps.push(TrajectoryStep {
            step: self.steps.len(),
            timestamp: Utc::now(),
            observation,
            action,
            response,
            cumulative_reward: self.total_reward,
            terminal,
            metadata: HashMap::new(),
        });
        
        if terminal {
            self.success = reward > 0.0;
            self.end_time = Utc::now();
        }
    }

    /// Calculate success metrics
    pub fn success_rate(&self) -> f32 {
        if self.success { 1.0 } else { 0.0 }
    }

    /// Get average step length
    pub fn avg_step_length(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }
        let total: usize = self.steps.iter()
            .map(|s| s.observation.len() + s.action.len())
            .sum();
        total as f32 / self.steps.len() as f32
    }

    /// Export to JSON for training
    pub fn to_training_sample(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "task": self.task,
            "steps": self.steps,
            "success": self.success,
            "total_reward": self.total_reward,
            "tokens_used": self.tokens_used,
            "duration_ms": (self.end_time - self.start_time).num_milliseconds(),
        })
    }
}

/// Batch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Number of trajectories to generate
    pub num_trajectories: usize,
    /// Maximum steps per trajectory
    pub max_steps: usize,
    /// Model to use
    pub model: String,
    /// Temperature for sampling
    pub temperature: f32,
    /// Parallel workers
    pub workers: usize,
    /// Output directory
    pub output_dir: String,
    /// Environment type
    pub environment: String,
    /// System prompt
    pub system_prompt: String,
    /// Tasks/prompts to use
    pub tasks: Vec<String>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            num_trajectories: 100,
            max_steps: 50,
            model: "gpt-4".to_string(),
            temperature: 0.7,
            workers: 10,
            output_dir: "./trajectories".to_string(),
            environment: "coding".to_string(),
            system_prompt: "You are a helpful coding assistant.".to_string(),
            tasks: Vec::new(),
        }
    }
}

/// Batch runner for generating multiple trajectories
pub struct BatchRunner {
    config: BatchConfig,
    http_client: reqwest::Client,
    rng: StdRng,
}

impl BatchRunner {
    /// Create a new batch runner
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            rng: StdRng::from_entropy(),
        }
    }

    /// Run a batch of trajectories
    pub async fn run(&mut self, tasks: Vec<String>) -> Result<Vec<Trajectory>> {
        let tasks = if tasks.is_empty() {
            self.generate_default_tasks()?
        } else {
            tasks
        };

        let mut all_trajectories = Vec::new();
        
        // Process in batches
        for batch in tasks.chunks(self.config.workers) {
            let mut handles = Vec::new();
            
            for task in batch {
                let mut runner = Self::new(self.config.clone());
                let task = task.clone();
                handles.push(tokio::spawn(async move {
                    runner.run_single(task).await
                }));
            }
            
            // Collect results
            for handle in handles {
                if let Ok(trajectory) = handle.await {
                    if let Ok(t) = trajectory {
                        all_trajectories.push(t);
                    }
                }
            }
        }

        Ok(all_trajectories)
    }

    /// Run a single trajectory
    pub async fn run_single(&mut self, task: String) -> Result<Trajectory> {
        let mut trajectory = Trajectory::new(
            task.clone(),
            self.config.environment.clone(),
            self.config.model.clone(),
        );

        let mut current_observation = task;
        let mut steps = 0;

        while steps < self.config.max_steps {
            // Generate action using LLM
            let action = self.generate_action(&current_observation).await?;
            
            // Execute action and get response
            let response = self.execute_action(&action).await?;
            
            // Calculate reward
            let reward = self.calculate_reward(&response);
            
            // Check if terminal
            let terminal = steps >= self.config.max_steps - 1 || self.is_success(&response);
            
            trajectory.add_step(
                current_observation.clone(),
                action.clone(),
                response.clone(),
                reward,
                terminal,
            );
            
            if terminal {
                break;
            }

            current_observation = response;
            steps += 1;
        }

        Ok(trajectory)
    }

    /// Generate an action based on observation
    async fn generate_action(&self, observation: &str) -> Result<String> {
        // In production, this would call the LLM API
        // For now, return a placeholder
        Ok(format!("Action for: {}", &observation[..observation.len().min(50)]))
    }

    /// Execute an action and get response
    async fn execute_action(&self, action: &str) -> Result<String> {
        // In production, this would execute the action
        // For now, return a placeholder
        Ok(format!("Response to: {}", action))
    }

    /// Calculate reward for a response
    fn calculate_reward(&self, response: &str) -> f32 {
        // Simple reward based on response characteristics
        let mut reward = 0.0;
        
        if response.contains("```") {
            reward += 0.2; // Contains code
        }
        if response.len() > 100 {
            reward += 0.1; // Substantial response
        }
        if response.contains("error") || response.contains("fail") {
            reward -= 0.5; // Contains errors
        }
        
        reward
    }

    /// Check if response indicates success
    fn is_success(&self, response: &str) -> bool {
        response.contains("success") ||
        response.contains("complete") ||
        response.contains("done") ||
        response.contains("solved")
    }

    /// Generate default tasks for training
    fn generate_default_tasks(&mut self) -> Result<Vec<String>> {
        let tasks = vec![
            "Write a function to reverse a string in Python".to_string(),
            "Implement a binary search algorithm".to_string(),
            "Create a REST API endpoint for user registration".to_string(),
            "Write unit tests for a calculator module".to_string(),
            "Debug this code and explain the fix".to_string(),
            "Optimize this SQL query for better performance".to_string(),
            "Refactor this class to follow SOLID principles".to_string(),
            "Add authentication to this API endpoint".to_string(),
            "Create a Docker container for this application".to_string(),
            "Write documentation for this function".to_string(),
        ];
        
        // Repeat tasks to reach num_trajectories
        let mut all_tasks = Vec::new();
        while all_tasks.len() < self.config.num_trajectories {
            for task in &tasks {
                all_tasks.push(task.clone());
                if all_tasks.len() >= self.config.num_trajectories {
                    break;
                }
            }
        }
        
        Ok(all_tasks)
    }

    /// Save trajectories to disk
    pub async fn save_trajectories(&self, trajectories: &[Trajectory]) -> Result<()> {
        use tokio::fs;
        
        fs::create_dir_all(&self.config.output_dir).await?;
        
        for trajectory in trajectories {
            let filename = format!("{}/{}.json", self.config.output_dir, trajectory.id);
            let json = serde_json::to_string_pretty(trajectory)?;
            fs::write(&filename, json).await?;
        }
        
        // Save summary
        let summary = serde_json::json!({
            "num_trajectories": trajectories.len(),
            "success_rate": trajectories.iter().map(|t| t.success_rate()).sum::<f32>() / trajectories.len() as f32,
            "avg_steps": trajectories.iter().map(|t| t.steps.len()).sum::<usize>() as f32 / trajectories.len() as f32,
            "total_tokens": trajectories.iter().map(|t| t.tokens_used).sum::<usize>(),
        });
        
        fs::write(
            format!("{}/summary.json", self.config.output_dir),
            serde_json::to_string_pretty(&summary)?,
        ).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_trajectory() {
        let config = BatchConfig {
            num_trajectories: 1,
            max_steps: 5,
            ..Default::default()
        };
        
        let mut runner = BatchRunner::new(config);
        let trajectory = runner.run_single("Test task".to_string()).await.unwrap();
        
        assert!(!trajectory.id.is_empty());
        assert_eq!(trajectory.task, "Test task");
    }

    #[test]
    fn test_trajectory_reward() {
        let mut trajectory = Trajectory::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
        );
        
        trajectory.add_step("obs1".to_string(), "action1".to_string(), "resp1".to_string(), 0.5, false);
        trajectory.add_step("obs2".to_string(), "action2".to_string(), "resp2".to_string(), 0.5, true);
        
        assert_eq!(trajectory.total_reward, 1.0);
        assert_eq!(trajectory.steps.len(), 2);
    }
}
