//! Service Discovery
//!
//! Service discovery for finding and connecting to agents.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{registry::{Service, ServiceRegistry}, adapter::ACPMessage};

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Discovery method
    pub method: DiscoveryMethod,
    /// Registry URL (for centralized discovery)
    pub registry_url: Option<String>,
    /// Broadcast interval in seconds
    pub broadcast_interval: u64,
    /// Discovery timeout in seconds
    pub timeout: u64,
    /// Local network discovery enabled
    pub local_discovery: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            method: DiscoveryMethod::Centralized,
            registry_url: Some("http://localhost:8080".to_string()),
            broadcast_interval: 30,
            timeout: 60,
            local_discovery: false,
        }
    }
}

/// Discovery method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Centralized registry
    Centralized,
    /// Broadcast discovery
    Broadcast,
    /// DNS-based discovery
    Dns,
    /// Local network multicast
    Multicast,
}

/// Service query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQuery {
    /// Service type filter
    pub service_type: Option<String>,
    /// Name pattern (supports wildcards)
    pub name_pattern: Option<String>,
    /// Required capabilities
    pub capabilities: Vec<String>,
    /// Tags filter
    pub tags: Vec<String>,
    /// Maximum results
    pub limit: usize,
    /// Include only healthy services
    pub healthy_only: bool,
}

impl Default for ServiceQuery {
    fn default() -> Self {
        Self {
            service_type: None,
            name_pattern: None,
            capabilities: Vec::new(),
            tags: Vec::new(),
            limit: 10,
            healthy_only: true,
        }
    }
}

impl ServiceQuery {
    /// Create query for agents
    pub fn agents() -> Self {
        Self {
            service_type: Some("agent".to_string()),
            ..Default::default()
        }
    }

    /// Create query for tools
    pub fn tools() -> Self {
        Self {
            service_type: Some("tool".to_string()),
            ..Default::default()
        }
    }

    /// Filter by name pattern
    pub fn name(mut self, pattern: String) -> Self {
        self.name_pattern = Some(pattern);
        self
    }

    /// Filter by capabilities
    pub fn capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }

    /// Filter by tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    /// Found services
    pub services: Vec<DiscoveredService>,
    /// Query used
    pub query: ServiceQuery,
    /// Discovery duration in ms
    pub duration_ms: u64,
    /// Total services matching (before limit)
    pub total_matches: usize,
}

/// A discovered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service info
    pub service: Service,
    /// Distance/latency (if available)
    pub latency_ms: Option<u64>,
    /// Health status
    pub healthy: bool,
    /// Score (relevance)
    pub score: f32,
}

impl DiscoveredService {
    /// Get best endpoint URL
    pub fn best_endpoint(&self) -> Option<&str> {
        self.service.endpoints.first().map(|e| e.url.as_str())
    }
}

/// Service Discovery
pub struct ServiceDiscovery {
    config: DiscoveryConfig,
    registry: Option<ServiceRegistry>,
    cache: HashMap<String, CachedDiscovery>,
}

impl ServiceDiscovery {
    /// Create a new discovery instance
    pub fn new(config: DiscoveryConfig) -> Self {
        let registry = config.registry_url.as_ref().map(|_| ServiceRegistry::new());
        Self {
            config,
            registry,
            cache: HashMap::new(),
        }
    }

    /// Create with registry
    pub fn with_registry(config: DiscoveryConfig, registry: ServiceRegistry) -> Self {
        Self {
            config,
            registry: Some(registry),
            cache: HashMap::new(),
        }
    }

    /// Discover services matching query
    pub async fn discover(&mut self, query: ServiceQuery) -> Result<DiscoveryResult> {
        let start = std::time::Instant::now();

        let services = if let Some(ref registry) = self.registry {
            self.discover_from_registry(registry, &query).await
        } else {
            self.discover_broadcast(&query).await?
        };

        let total_matches = services.len();
        let services: Vec<DiscoveredService> = services
            .into_iter()
            .take(query.limit)
            .collect();

        Ok(DiscoveryResult {
            services,
            query,
            duration_ms: start.elapsed().as_millis() as u64,
            total_matches,
        })
    }

    /// Discover from registry
    async fn discover_from_registry(
        &self,
        registry: &ServiceRegistry,
        query: &ServiceQuery,
    ) -> Vec<DiscoveredService> {
        let mut services = if let Some(ref service_type) = query.service_type {
            registry.find_by_type(service_type).await
        } else {
            registry.list().await
        };

        // Apply filters
        services = services
            .into_iter()
            .filter(|s| {
                // Name pattern filter
                if let Some(ref pattern) = query.name_pattern {
                    glob_match(pattern, &s.name)
                } else {
                    true
                }
            })
            .filter(|s| {
                // Capability filter
                if query.capabilities.is_empty() {
                    true
                } else {
                    query.capabilities.iter().all(|c| {
                        s.endpoints.iter().any(|e| e.capabilities.contains(c))
                    })
                }
            })
            .filter(|s| {
                // Tag filter
                if query.tags.is_empty() {
                    true
                } else {
                    query.tags.iter().any(|t| s.tags.contains(t))
                }
            })
            .collect();

        // Check health
        let mut discovered: Vec<DiscoveredService> = Vec::new();
        for service in services {
            let healthy = if query.healthy_only {
                service.endpoints.iter().any(|e| {
                    // In production, check health
                    true
                })
            } else {
                true
            };

            discovered.push(DiscoveredService {
                service,
                latency_ms: None,
                healthy,
                score: 1.0,
            });
        }

        discovered
    }

    /// Discover via broadcast
    async fn discover_broadcast(&mut self, query: &ServiceQuery) -> Result<Vec<DiscoveredService>> {
        // In production, send broadcast and collect responses
        Ok(Vec::new())
    }

    /// Find nearest service of a type
    pub async fn find_nearest(&mut self, service_type: &str) -> Result<Option<DiscoveredService>> {
        let result = self
            .discover(ServiceQuery {
                service_type: Some(service_type.to_string()),
                limit: 1,
                ..Default::default()
            })
            .await?;

        Ok(result.services.into_iter().next())
    }

    /// Find all agents
    pub async fn find_agents(&mut self) -> Result<Vec<DiscoveredService>> {
        let result = self.discover(ServiceQuery::agents()).await?;
        Ok(result.services)
    }

    /// Cache a discovery result
    pub fn cache_result(&mut self, query: &ServiceQuery, result: &DiscoveryResult) {
        let key = self.query_key(query);
        self.cache.insert(
            key,
            CachedDiscovery {
                result: result.clone(),
                cached_at: chrono::Utc::now(),
                ttl_seconds: 60,
            },
        );
    }

    /// Get cached result if valid
    pub fn get_cached(&self, query: &ServiceQuery) -> Option<DiscoveryResult> {
        let key = self.query_key(query);
        self.cache.get(&key).and_then(|c| {
            if c.is_valid() {
                Some(c.result.clone())
            } else {
                None
            }
        })
    }

    /// Generate cache key for query
    fn query_key(&self, query: &ServiceQuery) -> String {
        format!(
            "{:?}:{:?}:{:?}",
            query.service_type, query.name_pattern, query.capabilities
        )
    }

    /// Announce this agent for discovery
    pub async fn announce(&self, service: &Service) -> Result<()> {
        if let Some(ref registry) = self.registry {
            // Register with registry
        }
        Ok(())
    }
}

/// Cached discovery result
#[derive(Debug, Clone)]
struct CachedDiscovery {
    result: DiscoveryResult,
    cached_at: chrono::DateTime<chrono::Utc>,
    ttl_seconds: u64,
}

impl CachedDiscovery {
    fn is_valid(&self) -> bool {
        let elapsed = chrono::Utc::now() - self.cached_at;
        (elapsed.num_seconds() as u64) < self.ttl_seconds
    }
}

/// Simple glob matching
fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return name.starts_with(parts[0]) && name.ends_with(parts[1]);
        } else if parts.len() == 1 {
            return name.starts_with(parts[0]);
        }
    }
    pattern == name
}

/// Built-in capability definitions
pub mod capabilities {
    /// Standard capabilities
    pub const CODING: &str = "coding";
    pub const RESEARCH: &str = "research";
    pub const COMMUNICATION: &str = "communication";
    pub const FILE_MANAGEMENT: &str = "file_management";
    pub const WEB_SEARCH: &str = "web_search";
    pub const DATABASE: &str = "database";
    pub const API_ACCESS: &str = "api_access";
    pub const SCHEDULING: &str = "scheduling";
    pub const NOTIFICATIONS: &str = "notifications";

    /// Get all standard capabilities
    pub fn all() -> Vec<&'static str> {
        vec![
            CODING,
            RESEARCH,
            COMMUNICATION,
            FILE_MANAGEMENT,
            WEB_SEARCH,
            DATABASE,
            API_ACCESS,
            SCHEDULING,
            NOTIFICATIONS,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery() {
        let config = DiscoveryConfig::default();
        let mut discovery = ServiceDiscovery::new(config);

        let query = ServiceQuery::agents();
        let result = discovery.discover(query).await.unwrap();

        assert!(result.services.is_empty()); // Empty registry
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("test*", "testing"));
        assert!(glob_match("*test", "mytest"));
        assert!(glob_match("test*test", "testabtest"));
        assert!(!glob_match("test*", "nottest"));
    }
}
