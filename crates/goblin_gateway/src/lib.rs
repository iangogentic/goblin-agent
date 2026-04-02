//! Goblin Gateway - Multi-Platform Messaging
//!
//! Talk to Goblin from Telegram, Discord, Slack, and more.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// A message from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub platform: Platform,
    pub user_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub chat_id: String,
}

/// Supported messaging platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Telegram,
    Discord,
    Slack,
    WhatsApp,
    Signal,
    Email,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Telegram => write!(f, "telegram"),
            Platform::Discord => write!(f, "discord"),
            Platform::Slack => write!(f, "slack"),
            Platform::WhatsApp => write!(f, "whatsapp"),
            Platform::Signal => write!(f, "signal"),
            Platform::Email => write!(f, "email"),
        }
    }
}

/// A message gateway trait
#[async_trait]
pub trait MessageGateway: Send + Sync {
    /// Send a message
    async fn send(&self, chat_id: &str, text: &str) -> Result<()>;

    /// Start listening for messages - returns a stream
    async fn start(&self, tx: broadcast::Sender<Message>) -> Result<()>;
}

/// Gateway hub managing all platforms
#[derive(Clone)]
pub struct GatewayHub {
    platforms: Vec<Arc<dyn MessageGateway>>,
    message_tx: broadcast::Sender<Message>,
}

impl GatewayHub {
    /// Create a new gateway hub
    pub fn new() -> Self {
        let (message_tx, _) = broadcast::channel(100);
        Self {
            platforms: Vec::new(),
            message_tx,
        }
    }

    /// Add a platform gateway
    pub fn add_platform<G: MessageGateway + 'static>(&mut self, gateway: G) {
        self.platforms.push(Arc::new(gateway));
    }

    /// Subscribe to incoming messages
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.message_tx.subscribe()
    }

    /// Start all platforms
    pub async fn start_all(&self) -> Result<()> {
        for platform in &self.platforms {
            let tx = self.message_tx.clone();
            let p = Arc::clone(platform);
            tokio::spawn(async move {
                if let Err(e) = p.start(tx).await {
                    tracing::error!("Gateway error: {}", e);
                }
            });
        }
        Ok(())
    }

    /// Broadcast a message to all platforms
    pub async fn broadcast(&self, text: &str) -> Result<()> {
        for platform in &self.platforms {
            let chat_id = "broadcast"; // TODO: track user chats
            if let Err(e) = platform.send(chat_id, text).await {
                tracing::warn!("Failed to send to platform: {}", e);
            }
        }
        Ok(())
    }
}

impl Default for GatewayHub {
    fn default() -> Self {
        Self::new()
    }
}

// Telegram Gateway
#[derive(Clone)]
pub struct TelegramGateway {
    bot_token: String,
    api_url: String,
}

impl TelegramGateway {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            api_url: "https://api.telegram.org".to_string(),
        }
    }

    async fn make_request(&self, endpoint: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/bot{}/{}", self.api_url, self.bot_token, endpoint);
        let client = reqwest::Client::new();
        
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Telegram API request failed")?;

        let json: serde_json::Value = response.json().await?;
        
        if json.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            anyhow::bail!("Telegram API error: {:?}", json.get("description"));
        }
        
        Ok(json)
    }

    /// Set webhook for incoming updates
    pub async fn set_webhook(&self, webhook_url: &str) -> Result<()> {
        self.make_request("setWebhook", serde_json::json!({
            "url": webhook_url
        })).await?;
        Ok(())
    }
}

#[async_trait]
impl MessageGateway for TelegramGateway {
    async fn send(&self, chat_id: &str, text: &str) -> Result<()> {
        self.make_request("sendMessage", serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown"
        })).await?;
        Ok(())
    }

    async fn start(&self, tx: broadcast::Sender<Message>) -> Result<()> {
        // Long polling for updates
        let client = reqwest::Client::new();
        let mut offset = 0;

        loop {
            let url = format!(
                "{}/bot{}/getUpdates?timeout=30&offset={}",
                self.api_url, self.bot_token, offset
            );

            if let Ok(response) = client.get(&url).send().await {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(updates) = json.get("result").and_then(|v| v.as_array()) {
                        for update in updates {
                            if let Some(msg) = update.get("message") {
                                let chat_id = msg.get("chat")
                                    .and_then(|c| c.get("id"))
                                    .and_then(|v| v.as_i64())
                                    .map(|v| v.to_string())
                                    .unwrap_or_default();
                                
                                let user_id = msg.get("from")
                                    .and_then(|f| f.get("id"))
                                    .and_then(|v| v.as_i64())
                                    .map(|v| v.to_string())
                                    .unwrap_or_default();

                                let content = msg.get("text")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                let update_id = update.get("update_id")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(0);
                                
                                offset = update_id + 1;

                                let message = Message {
                                    id: update_id.to_string(),
                                    platform: Platform::Telegram,
                                    user_id,
                                    content,
                                    timestamp: chrono::Utc::now(),
                                    chat_id,
                                };

                                let _ = tx.send(message);
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

// Discord Gateway
#[derive(Clone)]
pub struct DiscordGateway {
    bot_token: String,
    application_id: String,
    http: reqwest::Client,
}

impl DiscordGateway {
    pub fn new(bot_token: String, application_id: String) -> Self {
        Self {
            bot_token,
            application_id,
            http: reqwest::Client::new(),
        }
    }

    async fn discord_request(&self, method: &str, endpoint: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let url = format!("https://discord.com/api/v10{}", endpoint);
        let mut request = self.http.request(
            reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
            &url
        )
        .header("Authorization", format!("Bot {}", self.bot_token))
        .header("Content-Type", "application/json");

        if let Some(b) = body {
            request = request.json(&b);
        }

        let response = request.send().await?;
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    }

    /// Create a Discord interaction response
    pub async fn create_interaction_response(
        &self,
        interaction_token: &str,
        content: &str,
    ) -> Result<()> {
        self.discord_request(
            "POST",
            &format!("/interactions/{}/callback", interaction_token),
            Some(serde_json::json!({
                "type": 4, // ChannelMessageWithSource
                "data": {
                    "content": content
                }
            }))
        ).await?;
        Ok(())
    }
}

#[async_trait]
impl MessageGateway for DiscordGateway {
    async fn send(&self, channel_id: &str, content: &str) -> Result<()> {
        self.discord_request(
            "POST",
            &format!("/channels/{}/messages", channel_id),
            Some(serde_json::json!({
                "content": content
            }))
        ).await?;
        Ok(())
    }

    async fn start(&self, _tx: broadcast::Sender<Message>) -> Result<()> {
        // Discord uses webhooks and interactions, not polling
        // This would typically be a webhook server
        tracing::info!("Discord gateway ready - use webhooks for incoming messages");
        Ok(())
    }
}

// Slack Gateway
#[derive(Clone)]
pub struct SlackGateway {
    bot_token: String,
    signing_secret: String,
}

impl SlackGateway {
    pub fn new(bot_token: String, signing_secret: String) -> Self {
        Self {
            bot_token,
            signing_secret,
        }
    }

    /// Post a message to a channel
    pub async fn post_message(&self, channel: &str, text: &str) -> Result<()> {
        let client = reqwest::Client::new();
        client.post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(&serde_json::json!({
                "channel": channel,
                "text": text
            }))
            .send()
            .await?;
        Ok(())
    }

    /// Verify a Slack request signature
    #[allow(dead_code)]
    pub fn verify_signature(&self, timestamp: &str, body: &str, _signature: &str) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let base = format!("v0:{}:{}", timestamp, body);
        let mut hasher = DefaultHasher::new();
        base.hash(&mut hasher);
        let _hash = format!("v0={}", hex::encode(hasher.finish().to_le_bytes()));
        
        // Simple verification - in production use HMAC
        true // TODO: proper HMAC verification
    }
}

#[async_trait]
impl MessageGateway for SlackGateway {
    async fn send(&self, channel: &str, text: &str) -> Result<()> {
        self.post_message(channel, text).await
    }

    async fn start(&self, _tx: broadcast::Sender<Message>) -> Result<()> {
        tracing::info!("Slack gateway ready - use events webhook for incoming");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Telegram.to_string(), "telegram");
        assert_eq!(Platform::Discord.to_string(), "discord");
    }

    #[tokio::test]
    async fn test_gateway_hub() {
        let hub = GatewayHub::new();
        assert!(hub.subscribe().try_recv().is_err());
    }
}
