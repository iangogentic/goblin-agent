//! Goblin Gateway - Email Integration
//!
//! Email integration for Goblin messaging.

use anyhow::{Context, Result};
use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Email Gateway
pub struct EmailGateway {
    smtp_host: String,
    smtp_port: u16,
    credentials: Credentials,
    from_address: String,
    from_name: String,
    message_handlers: Arc<RwLock<Vec<Box<dyn EmailMessageHandler>>>>,
}

impl EmailGateway {
    /// Create a new Email gateway with SMTP settings
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        username: String,
        password: String,
        from_address: String,
        from_name: String,
    ) -> Self {
        let credentials = Credentials::new(username, password);
        
        Self {
            smtp_host,
            smtp_port,
            credentials,
            from_address,
            from_name,
            message_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let smtp_host = std::env::var("SMTP_HOST")
            .unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let smtp_port = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let username = std::env::var("SMTP_USERNAME")
            .context("SMTP_USERNAME not set")?;
        let password = std::env::var("SMTP_PASSWORD")
            .context("SMTP_PASSWORD not set")?;
        let from_address = std::env::var("EMAIL_FROM")
            .unwrap_or_else(|_| username.clone());
        let from_name = std::env::var("EMAIL_FROM_NAME")
            .unwrap_or_else(|_| "Goblin".to_string());
        
        Ok(Self::new(
            smtp_host,
            smtp_port,
            username,
            password,
            from_address,
            from_name,
        ))
    }

    /// Register a message handler
    pub async fn on_message(&self, handler: Box<dyn EmailMessageHandler>) {
        let mut handlers = self.message_handlers.write().await;
        handlers.push(handler);
    }

    /// Send an email
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<EmailResponse> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_address).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body)?;
        
        let mailer = SmtpTransport::relay(&self.smtp_host)?
            .port(self.smtp_port)
            .credentials(self.credentials.clone())
            .build();
        
        let result = mailer.send(&email)
            .context("Failed to send email")?;
        
        Ok(EmailResponse {
            message_id: result.message_id().to_string(),
            accepted: result.accepted_recipients().to_vec(),
            rejected: result.rejected_recipients().to_vec(),
        })
    }

    /// Send HTML email
    pub async fn send_html_email(&self, to: &str, subject: &str, html_body: &str, text_body: Option<&str>) -> Result<EmailResponse> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_address).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(html_body)?;
        
        let mailer = SmtpTransport::relay(&self.smtp_host)?
            .port(self.smtp_port)
            .credentials(self.credentials.clone())
            .build();
        
        let result = mailer.send(&email)
            .context("Failed to send HTML email")?;
        
        Ok(EmailResponse {
            message_id: result.message_id().to_string(),
            accepted: result.accepted_recipients().to_vec(),
            rejected: result.rejected_recipients().to_vec(),
        })
    }

    /// Send email with attachment
    pub async fn send_email_with_attachment(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        attachments: Vec<Attachment>,
    ) -> Result<EmailResponse> {
        let mut email_builder = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_address).parse()?)
            .to(to.parse()?)
            .subject(subject);
        
        // Add attachments
        for attachment in &attachments {
            let mime_type = attachment.mime_type.parse()?;
            email_builder = email_builder
                .attachment(mime_type, attachment.filename.as_str(), attachment.data.as_str().into())?;
        }
        
        let email = email_builder.body(body)?;
        
        let mailer = SmtpTransport::relay(&self.smtp_host)?
            .port(self.smtp_port)
            .credentials(self.credentials.clone())
            .build();
        
        let result = mailer.send(&email)
            .context("Failed to send email with attachment")?;
        
        Ok(EmailResponse {
            message_id: result.message_id().to_string(),
            accepted: result.accepted_recipients().to_vec(),
            rejected: result.rejected_recipients().to_vec(),
        })
    }

    /// Process incoming email (via IMAP polling)
    pub async fn check_inbox(&self, host: &str, port: imap::types::Port, username: &str, password: &str) -> Result<Vec<IncomingEmail>> {
        let client = imap::Client::connect((host, port), username, password)?;
        let inbox = client.select("INBOX")?;
        
        let mut messages = Vec::new();
        
        // Fetch last 10 emails
        let seq = inbox.exists.saturating_sub(9)..=inbox.exists;
        let emails = client.fetch(seq, "BODY[]")?;
        
        for email in emails {
            if let Some(body) = email.body() {
                if let Ok(parsed) = mailparse::parse_mail(body) {
                    messages.push(IncomingEmail {
                        message_id: parsed.headers.iter()
                            .find(|h| h.get_key() == "Message-ID")
                            .map(|h| h.get_value()),
                        from: parsed.headers.iter()
                            .find(|h| h.get_key() == "From")
                            .map(|h| h.get_value()),
                        to: parsed.headers.iter()
                            .find(|h| h.get_key() == "To")
                            .map(|h| h.get_value()),
                        subject: parsed.headers.iter()
                            .find(|h| h.get_key() == "Subject")
                            .map(|h| h.get_value()),
                        body: parsed.get_body().ok(),
                        date: parsed.headers.iter()
                            .find(|h| h.get_key() == "Date")
                            .map(|h| h.get_value()),
                    });
                }
            }
        }
        
        Ok(messages)
    }
}

/// Email message handler
#[async_trait]
pub trait EmailMessageHandler: Send + Sync {
    async fn handle(&self, email: &IncomingEmail) -> Result<()>;
}

/// Incoming email
#[derive(Debug, Clone)]
pub struct IncomingEmail {
    /// Message ID
    pub message_id: Option<String>,
    /// From address
    pub from: Option<String>,
    /// To address
    pub to: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Body
    pub body: Option<String>,
    /// Date
    pub date: Option<String>,
}

/// Email response
#[derive(Debug, Clone)]
pub struct EmailResponse {
    /// Message ID
    pub message_id: String,
    /// Accepted recipients
    pub accepted: Vec<String>,
    /// Rejected recipients
    pub rejected: Vec<String>,
}

/// Email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// Data (base64 encoded)
    pub data: String,
}
