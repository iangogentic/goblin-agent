//! Goblin CLI - Setup Wizard and Commands
//!
//! Interactive CLI setup and command interface for Goblin.
//!
//! Features:
//! - Interactive setup wizard
//! - Model selection
//! - Provider configuration
//! - Tool management

use anyhow::Result;
use std::path::PathBuf;

pub mod setup;
pub mod commands;

pub use setup::SetupWizard;
pub use commands::{CliCommand, CommandRegistry};

/// CLI configuration
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Config directory
    pub config_dir: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
}

impl Default for CliConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            config_dir: home.join(".config").join("goblin"),
            data_dir: home.join(".local").join("share").join("goblin"),
        }
    }
}

/// Setup wizard state
#[derive(Debug, Clone)]
pub struct SetupState {
    /// Current step
    pub step: SetupStep,
    /// Selected provider
    pub provider: Option<String>,
    /// Selected model
    pub model: Option<String>,
    /// API key (if provided)
    pub api_key: Option<String>,
    /// Enabled features
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetupStep {
    Welcome,
    Provider,
    Model,
    ApiKey,
    Features,
    Complete,
}

impl SetupState {
    pub fn new() -> Self {
        Self {
            step: SetupStep::Welcome,
            provider: None,
            model: None,
            api_key: None,
            features: Vec::new(),
        }
    }

    pub fn next_step(&mut self) {
        self.step = match self.step {
            SetupStep::Welcome => SetupStep::Provider,
            SetupStep::Provider => SetupStep::Model,
            SetupStep::Model => SetupStep::ApiKey,
            SetupStep::ApiKey => SetupStep::Features,
            SetupStep::Features => SetupStep::Complete,
            SetupStep::Complete => SetupStep::Complete,
        };
    }

    pub fn is_complete(&self) -> bool {
        self.step == SetupStep::Complete
    }
}

impl Default for SetupState {
    fn default() -> Self {
        Self::new()
    }
}
