//! Honcho API Client
//!
//! Client for interacting with the Honcho API for user modeling and memory.
//!
//! Usage:
//! ```rust,no_run
//! use goblin_honcho::{HonchoClient, HonchoConfig, Message, MessageRole};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = HonchoConfig {
//!     workspace_id: "my-app".to_string(),
//!     api_key: "your-api-key".to_string(),
//!     ..Default::default()
//! };
//!
//! let client = HonchoClient::new(config);
//!
//! // Create a session
//! let session = client.create_session("conversation-1").await?;
//!
//! // Add messages
//! client.add_message(
//!     &session.id,
//!     "user",
//!     "Can you help me with my Rust code?"
//! ).await?;
//!
//! // Get context for LLM
//! let context = client.get_context(&session.id, 10000).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{HonchoConfig, LearnedProfile, Message};

/// Main Honcho API client
#[derive(Clone)]
pub struct HonchoClient {
    workspace_id: String,
    api_key: String,
    base_url: String,
    http: Client,
    /// Cached profile for the current user
    profile: Arc<RwLock<Option<LearnedProfile>>>,
}

impl HonchoClient {
    /// Create a new Honcho client
    pub fn new(config: HonchoConfig) -> Self {
        let base_url = if config.local {
            config.local_url.unwrap_or_else(|| "http://localhost:8000".to_string())
        } else {
            config.base_url.unwrap_or_else(|| "https://api.honcho.dev".to_string())
        };

        Self {
            workspace_id: config.workspace_id,
            api_key: config.api_key,
            base_url,
            http: Client::new(),
            profile: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if the client is configured (has API key)
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Get the base URL for API requests
    fn api_base(&self) -> String {
        format!("{}/v1/{}", self.base_url, self.workspace_id)
    }

    /// Create authorization header
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    // ========================================================================
    // Session Management
    // ========================================================================

    /// Create a new session
    pub async fn create_session(&self, session_id: &str) -> Result<Session> {
        #[derive(Serialize)]
        struct CreateSessionRequest {
            session_id: String,
        }

        let url = format!("{}/sessions", self.api_base());
        let response = self.http
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&CreateSessionRequest {
                session_id: session_id.to_string(),
            })
            .send()
            .await
            .context("Failed to create session")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create session: {}", response.status());
        }

        let session: Session = response.json().await?;
        Ok(session)
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let url = format!("{}/sessions", self.api_base());
        let response = self.http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to list sessions")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list sessions: {}", response.status());
        }

        let sessions: Vec<Session> = response.json().await?;
        Ok(sessions)
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/sessions/{}", self.api_base(), session_id);
        let response = self.http
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to delete session")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete session: {}", response.status());
        }

        Ok(())
    }

    // ========================================================================
    // Message Management
    // ========================================================================

    /// Add a message to a session
    pub async fn add_message(
        &self,
        session_id: &str,
        peer: &str,
        content: &str,
    ) -> Result<MessageResponse> {
        #[derive(Serialize)]
        struct AddMessageRequest {
            content: String,
            role: String,
        }

        let url = format!(
            "{}/sessions/{}/peers/{}/messages",
            self.api_base(),
            session_id,
            peer
        );

        let response = self.http
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&AddMessageRequest {
                content: content.to_string(),
                role: peer.to_string(),
            })
            .send()
            .await
            .context("Failed to add message")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to add message: {}", response.status());
        }

        let msg_response: MessageResponse = response.json().await?;
        Ok(msg_response)
    }

    /// Add a user message
    pub async fn add_user_message(&self, session_id: &str, content: &str) -> Result<MessageResponse> {
        self.add_message(session_id, "user", content).await
    }

    /// Add an agent message
    pub async fn add_agent_message(&self, session_id: &str, content: &str) -> Result<MessageResponse> {
        self.add_message(session_id, "agent", content).await
    }

    /// Get messages from a session
    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let url = format!("{}/sessions/{}/messages", self.api_base(), session_id);
        let response = self.http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to get messages")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get messages: {}", response.status());
        }

        let messages: Vec<Message> = response.json().await?;
        Ok(messages)
    }

    // ========================================================================
    // Context Retrieval
    // ========================================================================

    /// Get context for LLM usage (includes summary if needed)
    pub async fn get_context(&self, session_id: &str, max_tokens: usize) -> Result<ContextResponse> {
        let url = format!(
            "{}/sessions/{}/context?max_tokens={}",
            self.api_base(),
            session_id,
            max_tokens
        );

        let response = self.http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to get context")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get context: {}", response.status());
        }

        let context: ContextResponse = response.json().await?;
        Ok(context)
    }

    /// Get context as OpenAI-compatible format
    pub async fn get_openai_context(
        &self,
        session_id: &str,
        assistant: Option<&str>,
        max_tokens: usize,
    ) -> Result<OpenAIContext> {
        let url = format!(
            "{}/sessions/{}/context/to_openai?max_tokens={}&assistant={}",
            self.api_base(),
            session_id,
            max_tokens,
            assistant.unwrap_or("agent")
        );

        let response = self.http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to get OpenAI context")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get OpenAI context: {}", response.status());
        }

        let context: OpenAIContext = response.json().await?;
        Ok(context)
    }

    // ========================================================================
    // Peer (User) Management
    // ========================================================================

    /// Create or get a peer (user)
    pub async fn get_or_create_peer(&self, peer_id: &str) -> Result<Peer> {
        let url = format!("{}/peers/{}", self.api_base(), peer_id);
        
        let response = self.http
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to create peer")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create peer: {}", response.status());
        }

        let peer: Peer = response.json().await?;
        Ok(peer)
    }

    /// Chat with a peer about themselves (for insights)
    pub async fn peer_chat(&self, peer_id: &str, question: &str) -> Result<String> {
        #[derive(Serialize)]
        struct ChatRequest {
            question: String,
        }

        let url = format!("{}/peers/{}/chat", self.api_base(), peer_id);
        let response = self.http
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&ChatRequest {
                question: question.to_string(),
            })
            .send()
            .await
            .context("Failed to chat with peer")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to chat with peer: {}", response.status());
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            response: String,
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response.response)
    }

    /// Search peer messages
    pub async fn peer_search(&self, peer_id: &str, query: &str) -> Result<Vec<SearchResult>> {
        #[derive(Serialize)]
        struct SearchRequest {
            query: String,
        }

        let url = format!("{}/peers/{}/search", self.api_base(), peer_id);
        let response = self.http
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&SearchRequest {
                query: query.to_string(),
            })
            .send()
            .await
            .context("Failed to search peer messages")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to search peer messages: {}", response.status());
        }

        let results: Vec<SearchResult> = response.json().await?;
        Ok(results)
    }

    /// Get peer representation (learned profile)
    pub async fn get_peer_representation(
        &self,
        peer_id: &str,
        session_id: &str,
    ) -> Result<LearnedProfile> {
        let url = format!(
            "{}/sessions/{}/peers/{}/representation",
            self.api_base(),
            session_id,
            peer_id
        );

        let response = self.http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to get peer representation")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get peer representation: {}", response.status());
        }

        let profile: LearnedProfile = response.json().await?;
        
        // Cache the profile
        {
            let mut cached = self.profile.write().await;
            *cached = Some(profile.clone());
        }
        
        Ok(profile)
    }

    /// Get cached profile (if available)
    pub async fn get_cached_profile(&self) -> Option<LearnedProfile> {
        let profile = self.profile.read().await;
        profile.clone()
    }

    // ========================================================================
    // Workspace Management
    // ========================================================================

    /// Get workspace info
    pub async fn get_workspace(&self) -> Result<Workspace> {
        let url = format!("{}/workspace", self.api_base());
        let response = self.http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to get workspace")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get workspace: {}", response.status());
        }

        let workspace: Workspace = response.json().await?;
        Ok(workspace)
    }

    /// Check if Honcho is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.api_base());
        let response = self.http
            .get(&url)
            .send()
            .await
            .context("Failed to health check")?;

        Ok(response.status().is_success())
    }
}

// =============================================================================
// API Response Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub workspace_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub peer: String,
    pub content: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResponse {
    pub messages: Vec<ContextMessage>,
    pub summary: Option<String>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIContext {
    pub messages: Vec<OpenAIMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub workspace_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub message_id: String,
    pub content: String,
    pub score: f32,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub created_at: String,
}

// =============================================================================
// Builder Pattern for Easy Configuration
// =============================================================================

impl HonchoClient {
    /// Create a client with just workspace ID (for local development)
    pub fn local(workspace_id: &str) -> Self {
        Self::new(HonchoConfig {
            workspace_id: workspace_id.to_string(),
            api_key: String::new(),
            local: true,
            ..Default::default()
        })
    }

    /// Create a client with API key (for cloud service)
    pub fn cloud(workspace_id: &str, api_key: &str) -> Self {
        Self::new(HonchoConfig {
            workspace_id: workspace_id.to_string(),
            api_key: api_key.to_string(),
            local: false,
            ..Default::default()
        })
    }
}
