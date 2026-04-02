//! Service Registry
//!
//! Central registry for agent services. Allows agents to discover
//! and communicate with each other across distributed systems.

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A registered service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Unique endpoint ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Version
    pub version: String,
    /// URL or address
    pub url: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Health check endpoint
    pub health_url: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
}

impl ServiceEndpoint {
    /// Create a new endpoint
    pub fn new(name: String, url: String, version: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            url,
            version,
            capabilities: Vec::new(),
            health_url: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Set health check URL
    pub fn with_health_url(mut self, url: String) -> Self {
        self.health_url = Some(url);
        self
    }

    /// Check if healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(health_url) = &self.health_url {
            let client = reqwest::Client::new();
            match client.get(health_url).send().await {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            }
        } else {
            true // No health check means healthy
        }
    }
}

/// Service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service ID
    pub id: String,
    /// Service name (unique per registry)
    pub name: String,
    /// Service type
    pub service_type: String,
    /// Endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Owner/creator
    pub owner: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last heartbeat
    pub last_heartbeat: DateTime<Utc>,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

impl Service {
    /// Create a new service registration
    pub fn new(name: String, service_type: String, owner: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            service_type,
            endpoints: Vec::new(),
            owner,
            tags: Vec::new(),
            created_at: Utc::now(),
            last_heartbeat: Utc::now(),
            ttl_seconds: 300, // 5 minutes default
        }
    }

    /// Add an endpoint
    pub fn add_endpoint(mut self, endpoint: ServiceEndpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now() - self.last_heartbeat;
        elapsed.num_seconds() as u64 > self.ttl_seconds
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
    }
}

/// Registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationConfig {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: String,
    /// Owner identifier
    pub owner: String,
    /// Endpoints to register
    pub endpoints: Vec<ServiceEndpoint>,
    /// TTL in seconds
    pub ttl_seconds: u64,
    /// Tags
    pub tags: Vec<String>,
}

impl Default for RegistrationConfig {
    fn default() -> Self {
        Self {
            name: "goblin".to_string(),
            service_type: "agent".to_string(),
            owner: "default".to_string(),
            endpoints: Vec::new(),
            ttl_seconds: 300,
            tags: vec!["goblin".to_string()],
        }
    }
}

/// Service Registry
pub struct ServiceRegistry {
    /// Registered services
    services: Arc<DashMap<String, Service>>,
    /// Name to ID mapping
    by_name: Arc<DashMap<String, String>>,
    /// Type to IDs mapping
    by_type: Arc<DashMap<String, Vec<String>>>,
    /// Lock for atomic operations
    lock: Arc<RwLock<()>>,
}

impl ServiceRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(DashMap::new()),
            by_name: Arc::new(DashMap::new()),
            by_type: Arc::new(DashMap::new()),
            lock: Arc::new(RwLock::new(())),
        }
    }

    /// Register a service
    pub async fn register(&self, config: RegistrationConfig) -> Result<Service> {
        let _guard = self.lock.write().await;

        let mut service = Service::new(
            config.name.clone(),
            config.service_type.clone(),
            config.owner,
        );
        service.endpoints = config.endpoints;
        service.ttl_seconds = config.ttl_seconds;
        service.tags = config.tags.clone();

        // Add to type index
        self.by_type
            .entry(config.service_type.clone())
            .or_insert_with(Vec::new)
            .push(service.id.clone());

        // Add to name index
        self.by_name.insert(config.name.clone(), service.id.clone());

        // Store service
        self.services.insert(service.id.clone(), service.clone());

        Ok(service)
    }

    /// Unregister a service
    pub async fn unregister(&self, service_id: &str) -> Result<bool> {
        let _guard = self.lock.write().await;

        if let Some((_, service)) = self.services.remove(service_id) {
            // Remove from name index
            self.by_name.remove(&service.name);

            // Remove from type index
            if let Some(mut ids) = self.by_type.get_mut(&service.service_type) {
                ids.retain(|id| id != service_id);
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Get a service by ID
    pub async fn get(&self, service_id: &str) -> Option<Service> {
        self.services.get(service_id).map(|s| s.clone())
    }

    /// Get a service by name
    pub async fn get_by_name(&self, name: &str) -> Option<Service> {
        let _guard = self.lock.read().await;
        self.by_name.get(name).and_then(|id_ref| {
            let id_str: String = id_ref.clone();
            self.services.get(&id_str).map(|s| s.clone())
        })
    }

    /// Find services by type
    pub async fn find_by_type(&self, service_type: &str) -> Vec<Service> {
        let _guard = self.lock.read().await;
        self.by_type
            .get(service_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.services.get(id).map(|s| s.clone()))
                    .filter(|s| !s.is_expired())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find services by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<Service> {
        let _guard = self.lock.read().await;
        self.services
            .iter()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .filter(|s| !s.is_expired())
            .map(|s| s.clone())
            .collect()
    }

    /// Find services with query
    pub async fn find<F>(&self, predicate: F) -> Vec<Service>
    where
        F: Fn(&Service) -> bool,
    {
        let _guard = self.lock.read().await;
        self.services
            .iter()
            .filter(|s| !s.is_expired())
            .filter(|s| predicate(&s))
            .map(|s| s.clone())
            .collect()
    }

    /// Update heartbeat for a service
    pub async fn heartbeat(&self, service_id: &str) -> Result<bool> {
        let _guard = self.lock.write().await;
        if let Some(mut service) = self.services.get_mut(service_id) {
            service.heartbeat();
            return Ok(true);
        }
        Ok(false)
    }

    /// Clean up expired services
    pub async fn cleanup(&self) -> usize {
        let _guard = self.lock.write().await;
        let expired: Vec<String> = self
            .services
            .iter()
            .filter(|s| s.is_expired())
            .map(|s| s.id.clone())
            .collect();

        for id in &expired {
            self.services.remove(id);
        }

        expired.len()
    }

    /// List all services
    pub async fn list(&self) -> Vec<Service> {
        let _guard = self.lock.read().await;
        self.services
            .iter()
            .filter(|s| !s.is_expired())
            .map(|s| s.clone())
            .collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> RegistryStats {
        let _guard = self.lock.read().await;
        RegistryStats {
            total_services: self.services.len(),
            by_type: self
                .by_type
                .iter()
                .map(|e| (e.key().clone(), e.value().len()))
                .collect(),
            expired_count: self.services.iter().filter(|s| s.is_expired()).count(),
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_services: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub expired_count: usize,
}

/// In-memory registry for testing
pub mod memory {
    use super::*;

    /// Create a new in-memory registry
    pub fn new() -> ServiceRegistry {
        ServiceRegistry::new()
    }
}

/// HTTP registry client
pub mod client {
    use super::*;

    /// HTTP registry client
    pub struct RegistryClient {
        base_url: String,
        client: reqwest::Client,
    }

    impl RegistryClient {
        /// Create a new client
        pub fn new(base_url: String) -> Self {
            Self {
                base_url,
                client: reqwest::Client::new(),
            }
        }

        /// Register a service
        pub async fn register(&self, config: &RegistrationConfig) -> Result<Service> {
            let resp = self
                .client
                .post(&format!("{}/services", self.base_url))
                .json(config)
                .send()
                .await?;
            Ok(resp.json().await?)
        }

        /// Unregister a service
        pub async fn unregister(&self, service_id: &str) -> Result<bool> {
            let resp = self
                .client
                .delete(&format!("{}/services/{}", self.base_url, service_id))
                .send()
                .await?;
            Ok(resp.status().is_success())
        }

        /// Get a service
        pub async fn get(&self, service_id: &str) -> Result<Option<Service>> {
            let resp = self
                .client
                .get(&format!("{}/services/{}", self.base_url, service_id))
                .send()
                .await?;
            if resp.status().is_success() {
                Ok(Some(resp.json().await?))
            } else {
                Ok(None)
            }
        }

        /// Find services by type
        pub async fn find_by_type(&self, service_type: &str) -> Result<Vec<Service>> {
            let resp = self
                .client
                .get(&format!("{}/services?type={}", self.base_url, service_type))
                .send()
                .await?;
            Ok(resp.json().await?)
        }

        /// Send heartbeat
        pub async fn heartbeat(&self, service_id: &str) -> Result<bool> {
            let resp = self
                .client
                .post(&format!("{}/services/{}/heartbeat", self.base_url, service_id))
                .send()
                .await?;
            Ok(resp.status().is_success())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get() {
        let registry = ServiceRegistry::new();

        let config = RegistrationConfig {
            name: "test-agent".to_string(),
            service_type: "agent".to_string(),
            owner: "test".to_string(),
            endpoints: vec![ServiceEndpoint::new(
                "api".to_string(),
                "http://localhost:8080".to_string(),
                "1.0".to_string(),
            )],
            ..Default::default()
        };

        let service = registry.register(config).await.unwrap();
        assert_eq!(service.name, "test-agent");

        let retrieved = registry.get(&service.id).await;
        assert!(retrieved.is_some());

        let by_name = registry.get_by_name("test-agent").await;
        assert!(by_name.is_some());
    }

    #[tokio::test]
    async fn test_expiration() {
        let registry = ServiceRegistry::new();

        let config = RegistrationConfig {
            name: "expiring".to_string(),
            service_type: "test".to_string(),
            owner: "test".to_string(),
            ttl_seconds: 1, // 1 second TTL
            ..Default::default()
        };

        registry.register(config).await.unwrap();

        // Should be present
        let found = registry.find_by_type("test").await;
        assert_eq!(found.len(), 1);

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be expired
        let found = registry.find_by_type("test").await;
        assert_eq!(found.len(), 0);
    }
}
