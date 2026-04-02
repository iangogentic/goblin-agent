//! Atropos RL Environment
//!
//! Reinforcement learning environments for training agents.
//! Atropos is a multi-task learning framework for coding agents.

use anyhow::Result;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RL State representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLState {
    /// Current code context
    pub code: String,
    /// Available tools/actions
    pub available_actions: Vec<String>,
    /// Task description
    pub task: String,
    /// Execution history
    pub history: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// File system state
    pub files: HashMap<String, String>,
    /// Token budget remaining
    pub token_budget: usize,
    /// Step number
    pub step: usize,
}

impl RLState {
    /// Create a new state
    pub fn new(task: String, initial_code: String) -> Self {
        Self {
            code: initial_code,
            available_actions: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "run_command".to_string(),
                "search".to_string(),
                "edit".to_string(),
                "test".to_string(),
                "commit".to_string(),
            ],
            task,
            history: Vec::new(),
            env: HashMap::new(),
            files: HashMap::new(),
            token_budget: 128_000,
            step: 0,
        }
    }

    /// Encode state for LLM input
    pub fn encode(&self) -> String {
        let mut output = format!("Task: {}\n\n", self.task);
        output.push_str(&format!("Current code:\n{}\n\n", self.code));
        output.push_str("Available actions:\n");
        for action in &self.available_actions {
            output.push_str(&format!("- {}\n", action));
        }
        if !self.history.is_empty() {
            output.push_str("\nHistory:\n");
            for h in self.history.iter().rev().take(5) {
                output.push_str(&format!("- {}\n", h));
            }
        }
        output
    }
}

/// RL Action representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAction {
    /// Action type
    pub action_type: String,
    /// Action arguments
    pub args: HashMap<String, String>,
    /// Reasoning/explanation
    pub reasoning: String,
}

impl RLAction {
    /// Create a new action
    pub fn new(action_type: String, args: HashMap<String, String>, reasoning: String) -> Self {
        Self {
            action_type,
            args,
            reasoning,
        }
    }

    /// Execute the action and return result
    pub async fn execute(&self, state: &mut RLState) -> Result<String> {
        state.history.push(format!("{:?}: {:?}", self.action_type, self.args));
        state.step += 1;
        
        match self.action_type.as_str() {
            "read_file" => {
                let path = self.args.get("path").map(|s| s.as_str()).unwrap_or("");
                Ok(format!("Read file: {} - (mock content)", path))
            }
            "write_file" => {
                let path = self.args.get("path").map(|s| s.as_str()).unwrap_or("");
                let content = self.args.get("content").map(|s| s.as_str()).unwrap_or("");
                state.files.insert(path.to_string(), content.to_string());
                Ok(format!("Wrote {} bytes to {}", content.len(), path))
            }
            "run_command" => {
                let cmd = self.args.get("command").map(|s| s.as_str()).unwrap_or("");
                Ok(format!("Executed: {} (mock)", cmd))
            }
            "search" => {
                let query = self.args.get("query").map(|s| s.as_str()).unwrap_or("");
                Ok(format!("Search results for '{}' (mock)", query))
            }
            "edit" => {
                let edit = self.args.get("edit").map(|s| s.as_str()).unwrap_or("");
                state.code.push_str(&format!("\n// Edit: {}", edit));
                Ok(format!("Applied edit: {}", edit))
            }
            "test" => {
                Ok("Tests: 3 passed, 0 failed (mock)".to_string())
            }
            "commit" => {
                Ok("Committed changes (mock)".to_string())
            }
            _ => {
                Ok(format!("Unknown action: {}", self.action_type))
            }
        }
    }

    /// Calculate action cost (in tokens)
    pub fn cost(&self) -> usize {
        let args_str = serde_json::to_string(&self.args).unwrap_or_default();
        self.action_type.len() + args_str.len() + self.reasoning.len()
    }
}

/// RL Reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLReward {
    /// Primary reward value
    pub value: f32,
    /// Reward breakdown
    pub breakdown: HashMap<String, f32>,
    /// Success indicator
    pub success: bool,
    /// Termination reason
    pub termination: Option<String>,
}

impl RLReward {
    /// Calculate reward from state and action result
    pub fn from_execution(state: &RLState, action_result: &str, action: &RLAction) -> Self {
        let mut breakdown = HashMap::new();
        let mut total = 0.0f32;
        
        // Efficiency reward (prefer fewer steps)
        let efficiency = 1.0 - (state.step as f32 / 50.0).min(1.0);
        breakdown.insert("efficiency".to_string(), efficiency * 0.2);
        total += efficiency * 0.2;
        
        // Tool usage reward
        if action.action_type == "commit" || action.action_type == "test" {
            breakdown.insert("productive".to_string(), 0.3);
            total += 0.3;
        }
        
        // Progress reward (code is changing)
        if action.action_type == "edit" || action.action_type == "write_file" {
            breakdown.insert("progress".to_string(), 0.1);
            total += 0.1;
        }
        
        // Success reward
        let success = action_result.contains("passed") || 
                      action_result.contains("success") ||
                      action_result.contains("committed");
        if success {
            breakdown.insert("success".to_string(), 0.5);
            total += 0.5;
        }
        
        // Penalty for errors
        if action_result.contains("error") || action_result.contains("failed") {
            breakdown.insert("error".to_string(), -0.3);
            total -= 0.3;
        }
        
        let termination = if state.step >= 50 {
            Some("Max steps reached".to_string())
        } else {
            None
        };
        
        Self {
            value: total,
            breakdown,
            success,
            termination,
        }
    }
    
    /// Terminal reward (end of episode)
    pub fn terminal(&self, _state: &RLState) -> Self {
        if self.success {
            Self {
                value: self.value + 1.0,
                breakdown: {
                    let mut b = self.breakdown.clone();
                    b.insert("completion_bonus".to_string(), 1.0);
                    b
                },
                success: true,
                termination: self.termination.clone(),
            }
        } else {
            self.clone()
        }
    }
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,
    /// Max steps per episode
    pub max_steps: usize,
    /// Token budget
    pub token_budget: usize,
    /// Success criteria
    pub success_criteria: Vec<String>,
    /// Available tools
    pub tools: Vec<String>,
    /// Initial code template
    pub initial_template: String,
    /// Reward weights
    pub reward_weights: HashMap<String, f32>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            name: "coding".to_string(),
            max_steps: 50,
            token_budget: 128_000,
            success_criteria: vec![
                "tests pass".to_string(),
                "code compiles".to_string(),
                "commits successful".to_string(),
            ],
            tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "run_command".to_string(),
                "search".to_string(),
                "edit".to_string(),
                "test".to_string(),
                "commit".to_string(),
            ],
            initial_template: "# Start here\n".to_string(),
            reward_weights: HashMap::new(),
        }
    }
}

/// RL Environment
pub struct RLEnvironment {
    config: EnvironmentConfig,
    state: Option<RLState>,
    episode_reward: f32,
    episode_count: usize,
    rng: StdRng,
}

impl RLEnvironment {
    /// Create a new environment
    pub fn new(config: EnvironmentConfig) -> Self {
        Self {
            config,
            state: None,
            episode_reward: 0.0,
            episode_count: 0,
            rng: StdRng::from_entropy(),
        }
    }
    
    /// Reset the environment
    pub fn reset(&mut self, task: String) -> RLState {
        self.state = Some(RLState::new(task, self.config.initial_template.clone()));
        self.episode_reward = 0.0;
        RLState::clone(self.state.as_ref().unwrap())
    }
    
    /// Get current state
    pub fn state(&self) -> Option<&RLState> {
        self.state.as_ref()
    }
    
    /// Step the environment
    pub fn step(&mut self, action: RLAction) -> Result<(RLState, RLReward)> {
        let state = self.state.as_mut().expect("Environment not reset");
        
        // Execute action
        let result = tokio::runtime::Handle::current()
            .block_on(action.execute(state))?;
        
        // Calculate reward
        let reward = RLReward::from_execution(state, &result, &action);
        
        // Check termination
        let terminal = state.step >= self.config.max_steps || reward.termination.is_some();
        let final_reward = if terminal { reward.terminal(state) } else { reward.clone() };
        
        self.episode_reward += final_reward.value;
        
        Ok((RLState::clone(state), final_reward))
    }
    
    /// Get episode statistics
    pub fn stats(&self) -> EpisodeStats {
        EpisodeStats {
            episode: self.episode_count,
            total_reward: self.episode_reward,
            avg_reward: self.episode_reward / self.state.as_ref().map(|s| s.step.max(1) as f32).unwrap_or(1.0),
        }
    }
    
    /// Sample a random task
    pub fn sample_task(&mut self) -> String {
        let tasks = vec![
            "Implement a binary search tree".to_string(),
            "Write a REST API client library".to_string(),
            "Create a caching mechanism".to_string(),
            "Build a simple web server".to_string(),
            "Implement authentication middleware".to_string(),
        ];
        tasks[self.rng.gen_range(0..tasks.len())].clone()
    }
    
    /// List available actions
    pub fn available_actions(&self) -> &[String] {
        &self.config.tools
    }
}

/// Episode statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeStats {
    pub episode: usize,
    pub total_reward: f32,
    pub avg_reward: f32,
}

/// Run PPO-style training loop
pub async fn run_training_loop(
    env: &mut RLEnvironment,
    num_episodes: usize,
) -> Result<Vec<EpisodeStats>> {
    let mut all_stats = Vec::new();
    
    for _episode in 0..num_episodes {
        let task = env.sample_task();
        let mut state = env.reset(task);
        
        let mut trajectory = Vec::new();
        let mut _total_reward = 0.0f32;
        
        // Collect trajectory
        for _step in 0..env.config.max_steps {
            // In production, this would use a policy network
            // For now, generate random actions
            let action = generate_random_action(env.available_actions());
            
            let (new_state, reward) = env.step(action)?;
            trajectory.push((state.clone(), new_state.clone(), reward.clone()));
            _total_reward += reward.value;
            state = new_state;
            
            if reward.termination.is_some() {
                break;
            }
        }
        
        // Calculate and store stats
        all_stats.push(env.stats());
        
        // In production: update policy using collected trajectory
        // update_policy(&trajectory)?;
    }
    
    Ok(all_stats)
}

/// Generate a random action for exploration
fn generate_random_action(available: &[String]) -> RLAction {
    let mut rng = StdRng::from_entropy();
    let action_type = available[rng.gen_range(0..available.len())].clone();
    
    let mut args = HashMap::new();
    match action_type.as_str() {
        "read_file" => { args.insert("path".to_string(), "main.rs".to_string()); }
        "write_file" => {
            args.insert("path".to_string(), "new_file.rs".to_string());
            args.insert("content".to_string(), "// New code".to_string());
        }
        "run_command" => { args.insert("command".to_string(), "cargo build".to_string()); }
        "search" => { args.insert("query".to_string(), "TODO".to_string()); }
        "edit" => { args.insert("edit".to_string(), "fix bug".to_string()); }
        "test" => {}
        "commit" => {}
        _ => {}
    }
    
    RLAction::new(
        action_type,
        args,
        "Random action for exploration".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() {
        let config = EnvironmentConfig::default();
        let env = RLEnvironment::new(config);
        assert_eq!(env.config.max_steps, 50);
    }

    #[test]
    fn test_reset_and_step() {
        let config = EnvironmentConfig::default();
        let mut env = RLEnvironment::new(config);
        
        let state = env.reset("test task".to_string());
        assert_eq!(state.task, "test task");
        assert_eq!(state.step, 0);
    }
}
