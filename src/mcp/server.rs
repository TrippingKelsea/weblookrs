use anyhow::Result;
use super::mcp_sdk::server::{Server, ServerConfig};
use std::net::SocketAddr;
use tokio::sync::oneshot;

use super::actions;

/// MCP server for WebLook
pub struct MCPServer {
    server: Option<Server>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        MCPServer {
            server: None,
            shutdown_tx: None,
        }
    }

    /// Start the MCP server on the specified address
    pub async fn start(&mut self, addr: SocketAddr) -> Result<()> {
        // Create server config
        let config = ServerConfig::new()
            .with_addr(addr)
            .with_auth_disabled(); // For simplicity; in production, use proper auth
        
        // Create server
        let mut server = Server::new(config);
        
        // Register context actions
        actions::register_actions(&mut server)?;
        
        // Create shutdown channel
        let (tx, rx) = oneshot::channel();
        self.shutdown_tx = Some(tx);
        
        // Store server instance
        self.server = Some(server.clone());
        
        // Start server in background
        let server_handle = server.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = server_handle.serve() => {
                    println!("MCP server stopped");
                }
                _ = rx => {
                    println!("MCP server received shutdown signal");
                    let _ = server_handle.shutdown().await;
                }
            }
        });
        
        Ok(())
    }

    /// Stop the MCP server
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        if let Some(server) = &self.server {
            server.shutdown().await?;
        }
        
        self.server = None;
        
        Ok(())
    }
}

impl Drop for MCPServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
