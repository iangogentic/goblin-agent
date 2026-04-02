//! Goblin ACP - Agent Communication Protocol
//!
//! Multi-agent coordination protocol for Goblin agents.
//! Enables service discovery, message passing, and agent coordination.

pub mod registry;
pub mod adapter;
pub mod discovery;

pub use registry::{ServiceRegistry, Service, ServiceEndpoint, RegistrationConfig};
pub use adapter::{MessageAdapter, ACPMessage, ACPPayload, Protocol};
pub use discovery::{ServiceDiscovery, DiscoveryConfig, ServiceQuery};
