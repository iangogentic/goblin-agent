//! Message Adapter
//!
//! Protocol adapters for different messaging systems.
//! Supports HTTP, WebSocket, and gRPC transports.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Supported protocols
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Protocol {
    /// HTTP/REST
    Http,
    /// WebSocket
    WebSocket,
    /// gRPC
    Grpc,
    /// STDIO (for local processes)
    Stdio,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Http
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http => write!(f, "http"),
            Protocol::WebSocket => write!(f, "websocket"),
            Protocol::Grpc => write!(f, "grpc"),
            Protocol::Stdio => write!(f, "stdio"),
        }
    }
}

/// ACP Message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACPMessage {
    /// Message ID
    pub id: String,
    /// Sender ID
    pub sender: String,
    /// Recipient ID (optional for broadcast)
    pub recipient: Option<String>,
    /// Message type
    pub message_type: MessageType,
    /// Protocol version
    pub version: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Payload
    pub payload: ACPPayload,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Reply-to ID
    pub reply_to: Option<String>,
    /// Correlation ID for request/response
    pub correlation_id: Option<String>,
}

impl ACPMessage {
    /// Create a new message
    pub fn new(sender: String, message_type: MessageType, payload: ACPPayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            recipient: None,
            message_type,
            version: "1.0".to_string(),
            timestamp: Utc::now(),
            payload,
            headers: HashMap::new(),
            reply_to: None,
            correlation_id: None,
        }
    }

    /// Create a request message
    pub fn request(sender: String, action: String, params: serde_json::Value) -> Self {
        Self::new(
            sender,
            MessageType::Request,
            ACPPayload::Action {
                action,
                params,
            },
        )
    }

    /// Create a response message
    pub fn response(sender: String, result: serde_json::Value, correlation_id: String) -> Self {
        let mut msg = Self::new(
            sender,
            MessageType::Response,
            ACPPayload::Result { result },
        );
        msg.correlation_id = Some(correlation_id);
        msg
    }

    /// Create an event message
    pub fn event(sender: String, event_type: String, data: serde_json::Value) -> Self {
        Self::new(
            sender,
            MessageType::Event,
            ACPPayload::Event {
                event_type,
                data,
            },
        )
    }

    /// Create a broadcast message
    pub fn broadcast(sender: String, event_type: String, data: serde_json::Value) -> Self {
        let mut msg = Self::event(sender, event_type, data);
        msg.recipient = None; // Broadcast
        msg
    }

    /// Set recipient
    pub fn to(mut self, recipient: String) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Set reply-to
    pub fn reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Add header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }
}

/// Message types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// Request (expect response)
    Request,
    /// Response (to a request)
    Response,
    /// Event (fire and forget)
    Event,
    /// Error
    Error,
    /// Handshake (connection setup)
    Handshake,
    /// Heartbeat
    Heartbeat,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Event
    }
}

/// ACP Payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ACPPayload {
    /// Action payload (request)
    Action {
        action: String,
        params: serde_json::Value,
    },
    /// Result payload (response)
    Result {
        result: serde_json::Value,
    },
    /// Error payload
    Error {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },
    /// Event payload
    Event {
        event_type: String,
        data: serde_json::Value,
    },
    /// Handshake payload
    Handshake {
        agent_id: String,
        capabilities: Vec<String>,
        protocol_version: String,
    },
    /// Heartbeat payload
    Heartbeat {
        timestamp: DateTime<Utc>,
        status: String,
    },
}

impl ACPPayload {
    /// Create an error payload
    pub fn error(code: String, message: String) -> Self {
        Self::Error {
            code,
            message,
            details: None,
        }
    }

    /// Create a handshake payload
    pub fn handshake(agent_id: String, capabilities: Vec<String>) -> Self {
        Self::Handshake {
            agent_id,
            capabilities,
            protocol_version: "1.0".to_string(),
        }
    }

    /// Create a heartbeat payload
    pub fn heartbeat(status: String) -> Self {
        Self::Heartbeat {
            timestamp: Utc::now(),
            status,
        }
    }
}

/// Message Adapter trait
#[async_trait::async_trait]
pub trait MessageAdapter: Send + Sync {
    /// Send a message
    async fn send(&self, message: &ACPMessage) -> Result<()>;

    /// Receive a message
    async fn receive(&self) -> Result<Option<ACPMessage>>;

    /// Connect to endpoint
    async fn connect(&mut self, endpoint: &str) -> Result<()>;

    /// Disconnect
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Get protocol
    fn protocol(&self) -> Protocol;
}

/// HTTP Adapter
pub struct HttpAdapter {
    base_url: String,
    client: reqwest::Client,
    connected: bool,
}

impl HttpAdapter {
    /// Create a new HTTP adapter
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl MessageAdapter for HttpAdapter {
    async fn send(&self, message: &ACPMessage) -> Result<()> {
        let url = match &message.recipient {
            Some(r) => format!("{}/messages/{}", self.base_url, r),
            None => format!("{}/messages/broadcast", self.base_url),
        };

        self.client
            .post(&url)
            .json(message)
            .send()
            .await?;

        Ok(())
    }

    async fn receive(&self) -> Result<Option<ACPMessage>> {
        // HTTP is request/response, so this would need polling
        // In production, use Server-Sent Events or WebSocket
        Ok(None)
    }

    async fn connect(&mut self, endpoint: &str) -> Result<()> {
        self.base_url = endpoint.to_string();
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn protocol(&self) -> Protocol {
        Protocol::Http
    }
}

/// WebSocket Adapter
pub struct WebSocketAdapter {
    url: String,
    connected: bool,
}

impl WebSocketAdapter {
    /// Create a new WebSocket adapter
    pub fn new(url: String) -> Self {
        Self {
            url,
            connected: false,
        }
    }
}

#[async_trait::async_trait]
impl MessageAdapter for WebSocketAdapter {
    async fn send(&self, _message: &ACPMessage) -> Result<()> {
        // In production, use tokio-tungstenite
        Ok(())
    }

    async fn receive(&self) -> Result<Option<ACPMessage>> {
        // In production, receive from WebSocket stream
        Ok(None)
    }

    async fn connect(&mut self, endpoint: &str) -> Result<()> {
        self.url = endpoint.to_string();
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn protocol(&self) -> Protocol {
        Protocol::WebSocket
    }
}

/// STDIO Adapter (for local communication)
pub struct StdioAdapter {
    connected: bool,
}

impl StdioAdapter {
    /// Create a new STDIO adapter
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for StdioAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MessageAdapter for StdioAdapter {
    async fn send(&self, message: &ACPMessage) -> Result<()> {
        let json = serde_json::to_string(message)?;
        println!("{}", json);
        Ok(())
    }

    async fn receive(&self) -> Result<Option<ACPMessage>> {
        // In production, read from stdin
        Ok(None)
    }

    async fn connect(&mut self, _endpoint: &str) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn protocol(&self) -> Protocol {
        Protocol::Stdio
    }
}

/// Message builder for fluent API
pub struct MessageBuilder {
    sender: String,
    recipient: Option<String>,
    message_type: MessageType,
    payload: Option<ACPPayload>,
    headers: HashMap<String, String>,
    correlation_id: Option<String>,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new(sender: String) -> Self {
        Self {
            sender,
            recipient: None,
            message_type: MessageType::Event,
            payload: None,
            headers: HashMap::new(),
            correlation_id: None,
        }
    }

    /// Set recipient
    pub fn to(mut self, recipient: String) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Set message type
    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    /// Set action payload
    pub fn action(mut self, action: String, params: serde_json::Value) -> Self {
        self.payload = Some(ACPPayload::Action { action, params });
        self
    }

    /// Set result payload
    pub fn result(mut self, result: serde_json::Value) -> Self {
        self.payload = Some(ACPPayload::Result { result });
        self
    }

    /// Set error payload
    pub fn error(mut self, code: String, message: String) -> Self {
        self.payload = Some(ACPPayload::Error {
            code,
            message,
            details: None,
        });
        self
    }

    /// Set event payload
    pub fn event(mut self, event_type: String, data: serde_json::Value) -> Self {
        self.payload = Some(ACPPayload::Event { event_type, data });
        self
    }

    /// Add header
    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set correlation ID
    pub fn correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Build the message
    pub fn build(self) -> ACPMessage {
        ACPMessage {
            id: Uuid::new_v4().to_string(),
            sender: self.sender,
            recipient: self.recipient,
            message_type: self.message_type,
            version: "1.0".to_string(),
            timestamp: Utc::now(),
            payload: self.payload.unwrap_or_else(|| ACPPayload::Event {
                event_type: "unknown".to_string(),
                data: serde_json::json!({}),
            }),
            headers: self.headers,
            reply_to: None,
            correlation_id: self.correlation_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = ACPMessage::request(
            "agent1".to_string(),
            "echo".to_string(),
            serde_json::json!({"text": "hello"}),
        );

        assert_eq!(msg.sender, "agent1");
        assert_eq!(msg.message_type, MessageType::Request);
    }

    #[test]
    fn test_message_builder() {
        let msg = MessageBuilder::new("sender".to_string())
            .to("receiver".to_string())
            .action("test".to_string(), serde_json::json!({}))
            .header("x-custom".to_string(), "value".to_string())
            .build();

        assert_eq!(msg.sender, "sender");
        assert_eq!(msg.recipient, Some("receiver".to_string()));
    }
}
