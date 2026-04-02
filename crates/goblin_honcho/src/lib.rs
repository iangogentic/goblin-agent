//! Goblin Honcho - User Modeling and Dialectic System
//!
//! Honcho is a memory library for building stateful agents. This crate provides
//! integration with Honcho for user modeling, dialectic tracking, and conversation
//! analysis.
//!
//! Key features:
//! - User profile building from conversation style
//! - Dialectic conversation tracking
//! - Context injection for personalized responses
//! - Preference inference from behavior

use serde::{Deserialize, Serialize};

pub mod client;
pub mod dialectic;
pub mod profile;
pub mod analyzer;
pub mod context;

// Re-export main types
pub use client::HonchoClient;
pub use dialectic::DialecticTracker;
pub use profile::UserProfile;
pub use analyzer::ConversationAnalyzer;
pub use context::ContextInjector;

/// Configuration for Honcho integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonchoConfig {
    /// Workspace ID (your app identifier)
    pub workspace_id: String,
    /// API key for Honcho service
    pub api_key: String,
    /// Base URL (defaults to https://api.honcho.dev)
    pub base_url: Option<String>,
    /// Whether to use local Honcho server
    pub local: bool,
    /// Local server URL if using local mode
    pub local_url: Option<String>,
}

impl Default for HonchoConfig {
    fn default() -> Self {
        Self {
            workspace_id: "goblin".to_string(),
            api_key: String::new(),
            base_url: Some("https://api.honcho.dev".to_string()),
            local: false,
            local_url: Some("http://localhost:8000".to_string()),
        }
    }
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message content
    pub content: String,
    /// Who sent the message (user or agent)
    pub role: MessageRole,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Agent,
}

/// Dialectic analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialecticAnalysis {
    /// Communication style (formal/casual/technical)
    pub style: CommunicationStyle,
    /// Tone indicators
    pub tone: Vec<ToneIndicator>,
    /// Learning patterns
    pub learning_patterns: Vec<String>,
    /// Preferences inferred
    pub inferred_preferences: Vec<String>,
}

/// Communication style
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommunicationStyle {
    Formal,
    Casual,
    Technical,
    Mixed,
}

/// Tone indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToneIndicator {
    Patient,
    Impatient,
    Curious,
    Skeptical,
    Enthusiastic,
    Direct,
    Verbose,
    Concise,
}

/// User profile built from dialectic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedProfile {
    /// Primary communication style
    pub communication_style: CommunicationStyle,
    /// Technical level (1-10)
    pub technical_level: u8,
    /// Preferred explanation depth
    pub explanation_depth: ExplanationDepth,
    /// Response format preferences
    pub format_preferences: FormatPreferences,
    /// Tracked preferences
    pub preferences: Vec<Preference>,
    /// Interaction patterns
    pub patterns: Vec<InteractionPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExplanationDepth {
    Brief,
    Standard,
    Detailed,
    Comprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatPreferences {
    pub use_code_blocks: bool,
    pub use_bullets: bool,
    pub use_headers: bool,
    pub include_examples: bool,
    pub technical_jargon: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub key: String,
    pub value: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPattern {
    pub trigger: String,
    pub response_style: String,
    pub frequency: u32,
}
