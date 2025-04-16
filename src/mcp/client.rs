use anyhow::Result;
use super::mcp_sdk::client::{Client, ClientConfig};
use serde_json::Value;
use std::time::Duration;

/// MCP client for WebLook
pub struct MCPClient {
    client: Client,
}

impl MCPClient {
    /// Create a new MCP client connected to the specified endpoint
    pub async fn new(endpoint: &str) -> Result<Self> {
        let config = ClientConfig::new()
            .with_endpoint(endpoint)
            .with_timeout(Duration::from_secs(60))
            .with_auth_disabled(); // For simplicity; in production, use proper auth
        
        let client = Client::new(config).await?;
        
        Ok(MCPClient { client })
    }

    /// Invoke a context action on a remote MCP server
    pub async fn invoke_action(&self, action_name: &str, params: Value) -> Result<Value> {
        let response = self.client.invoke_action(action_name, params).await?;
        Ok(response)
    }

    /// Get available actions from the remote MCP server
    pub async fn get_available_actions(&self) -> Result<Vec<String>> {
        let actions = self.client.get_available_actions().await?;
        Ok(actions.into_iter().map(|a| a.name).collect())
    }
}
