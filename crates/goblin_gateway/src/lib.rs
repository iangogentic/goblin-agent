//! Goblin Gateway - Multi-Platform Messaging
//!
//! Talk to Goblin from Telegram, Discord, Slack, and more.

use anyhow::Result;

/// A message from any platform
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub platform: Platform,
    pub user_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Supported messaging platforms
#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Telegram,
    Discord,
    Slack,
    WhatsApp,
    Signal,
    Email,
}

/// A message gateway
pub trait MessageGateway: Send + Sync {
    /// Send a message
    fn send(&self, msg: &str) -> Result<()>;

    /// Start listening for messages
    fn start(&self) -> Result<()>;
}

/// Gateway hub managing all platforms
pub struct GatewayHub {
    platforms: Vec<Box<dyn MessageGateway>>,
}

impl GatewayHub {
    pub fn new() -> Self {
        Self {
            platforms: Vec::new(),
        }
    }

    pub fn add_platform<G: MessageGateway + 'static>(&mut self, gateway: G) {
        self.platforms.push(Box::new(gateway));
    }
}

impl Default for GatewayHub {
    fn default() -> Self {
        Self::new()
    }
}
