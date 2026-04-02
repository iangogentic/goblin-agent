use std::collections::BTreeMap;

use goblin_app::McpServerInfra;
use goblin_domain::McpServerConfig;

use crate::mcp_client::GoblinMcpClient;

#[derive(Clone)]
pub struct GoblinMcpServer;

#[async_trait::async_trait]
impl McpServerInfra for GoblinMcpServer {
    type Client = GoblinMcpClient;

    async fn connect(
        &self,
        config: McpServerConfig,
        env_vars: &BTreeMap<String, String>,
    ) -> anyhow::Result<Self::Client> {
        Ok(GoblinMcpClient::new(config, env_vars))
    }
}
