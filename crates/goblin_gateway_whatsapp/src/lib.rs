//! Goblin Gateway - WhatsApp Integration
//!
//! WhatsApp Business API integration for Goblin messaging.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// WhatsApp Gateway
pub struct WhatsAppGateway {
    client: Client,
    phone_number_id: String,
    access_token: String,
    api_url: String,
    webhook_secret: Option<String>,
    message_handlers: Arc<RwLock<Vec<Box<dyn WhatsAppMessageHandler>>>>,
}

impl WhatsAppGateway {
    /// Create a new WhatsApp gateway
    pub fn new(phone_number_id: String, access_token: String) -> Self {
        Self {
            client: Client::new(),
            phone_number_id,
            access_token,
            api_url: "https://graph.facebook.com/v18.0".to_string(),
            webhook_secret: None,
            message_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set webhook verification secret
    pub fn with_webhook_secret(mut self, secret: String) -> Self {
        self.webhook_secret = Some(secret);
        self
    }

    /// Register a message handler
    pub async fn on_message(&self, handler: Box<dyn WhatsAppMessageHandler>) {
        let mut handlers = self.message_handlers.write().await;
        handlers.push(handler);
    }

    /// Send a text message
    pub async fn send_message(&self, to: &str, text: &str) -> Result<MessageResponse> {
        let url = format!("{}/{}/messages", self.api_url, self.phone_number_id);
        
        let payload = SendMessageRequest {
            messaging_product: "whatsapp".to_string(),
            to: to.to_string(),
            r#type: "text".to_string(),
            text: Some(TextContent {
                preview_url: false,
                body: text.to_string(),
            }),
        };
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("WhatsApp API error: {}", error);
        }
        
        let result = response.json::<MessageResponse>().await?;
        Ok(result)
    }

    /// Send an image message
    pub async fn send_image(&self, to: &str, image_id: Option<&str>, caption: Option<&str>) -> Result<MessageResponse> {
        let url = format!("{}/{}/messages", self.api_url, self.phone_number_id);
        
        let payload = if let Some(id) = image_id {
            serde_json::json!({
                "messaging_product": "whatsapp",
                "to": to,
                "type": "image",
                "image": {
                    "id": id
                }
            })
        } else {
            serde_json::json!({
                "messaging_product": "whatsapp",
                "to": to,
                "type": "image",
                "image": {
                    "link": "https://example.com/image.jpg",
                    "caption": caption.unwrap_or("")
                }
            })
        };
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        let result = response.json::<MessageResponse>().await?;
        Ok(result)
    }

    /// Send typing indicator
    pub async fn send_typing(&self, to: &str, typing: bool) -> Result<()> {
        let url = format!("{}/{}/messages", self.api_url, self.phone_number_id);
        
        let _action = if typing { "typing_on" } else { "typing_off" };
        
        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "action",
            "action": {
                "typing": if typing { 1 } else { 0 }
            }
        });
        
        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        Ok(())
    }

    /// Mark message as read
    pub async fn mark_read(&self, message_id: &str) -> Result<()> {
        let url = format!("{}/{}/messages", self.api_url, self.phone_number_id);
        
        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "action": "mark_read",
            "message_id": message_id
        });
        
        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        Ok(())
    }

    /// Process incoming webhook
    pub async fn process_webhook(&self, payload: &str) -> Result<Vec<IncomingMessage>> {
        #[derive(Deserialize)]
        struct WebhookPayload {
            entry: Vec<Entry>,
        }
        
        #[derive(Deserialize)]
        struct Entry {
            changes: Vec<Change>,
        }
        
        #[derive(Deserialize)]
        struct Change {
            value: Value,
        }
        
        #[derive(Deserialize)]
        struct Value {
            messages: Option<Vec<Message>>,
        }
        
        #[derive(Deserialize)]
        struct Message {
            from: String,
            id: String,
            text: Option<TextBody>,
            image: Option<ImageBody>,
            timestamp: String,
            #[serde(rename = "type")]
            msg_type: String,
        }
        
        #[derive(Deserialize)]
        struct TextBody {
            body: String,
        }
        
        #[derive(Deserialize)]
        struct ImageBody {
            id: String,
        }
        
        let webhook: WebhookPayload = serde_json::from_str(payload)
            .context("Failed to parse webhook payload")?;
        
        let mut messages = Vec::new();
        
        for entry in webhook.entry {
            for change in entry.changes {
                if let Some(msgs) = change.value.messages {
                    for msg in msgs {
                        let content = if let Some(text) = msg.text {
                            MessageContent::Text(text.body)
                        } else if let Some(image) = msg.image {
                            MessageContent::Image(image.id)
                        } else {
                            MessageContent::Unknown
                        };
                        
                        messages.push(IncomingMessage {
                            from: msg.from,
                            id: msg.id,
                            content,
                            timestamp: msg.timestamp,
                            msg_type: msg.msg_type,
                        });
                    }
                }
            }
        }
        
        // Call handlers
        let handlers = self.message_handlers.read().await;
        for message in &messages {
            for handler in handlers.iter() {
                handler.handle(message).await?;
            }
        }
        
        Ok(messages)
    }

    /// Get message status
    pub async fn get_message_status(&self, message_id: &str) -> Result<MessageStatus> {
        let url = format!("{}/{}/messages/{}", self.api_url, self.phone_number_id, message_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;
        
        let result = response.json::<serde_json::Value>().await?;
        
        let status = result["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        Ok(MessageStatus {
            message_id: message_id.to_string(),
            status,
        })
    }
}

/// Message handler trait
#[async_trait::async_trait]
pub trait WhatsAppMessageHandler: Send + Sync {
    async fn handle(&self, message: &IncomingMessage) -> Result<()>;
}

/// Incoming message
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    /// Sender phone number
    pub from: String,
    /// Message ID
    pub id: String,
    /// Message content
    pub content: MessageContent,
    /// Timestamp
    pub timestamp: String,
    /// Message type
    pub msg_type: String,
}

/// Message content
#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    Image(String),
    Unknown,
}

/// Message response
#[derive(Debug, Clone, Deserialize)]
pub struct MessageResponse {
    /// Message ID
    pub messaging_product: Option<String>,
    pub contacts: Option<Vec<Contact>>,
    pub messages: Option<Vec<SentMessage>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Contact {
    pub input: String,
    pub wa_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SentMessage {
    pub id: String,
}

/// Message status
#[derive(Debug, Clone)]
pub struct MessageStatus {
    /// Message ID
    pub message_id: String,
    /// Status
    pub status: String,
}

/// Send message request
#[derive(Debug, Serialize)]
struct SendMessageRequest {
    messaging_product: String,
    to: String,
    #[serde(rename = "type")]
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextContent>,
}

#[derive(Debug, Serialize)]
struct TextContent {
    preview_url: bool,
    body: String,
}
