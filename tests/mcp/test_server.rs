use anyhow::Result;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

use weblook::mcp::MCPServer;

/// Test that the MCP server starts and stops correctly
#[tokio::test]
async fn test_server_start_stop() -> Result<()> {
    // Create a server on a random port
    let addr: SocketAddr = "127.0.0.1:0".parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}

/// Test that the server exposes the expected context actions
#[tokio::test]
async fn test_server_actions() -> Result<()> {
    // Create a server on a specific port
    let port = 9876;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a client to connect to the server
    let client = weblook::mcp::MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Get available actions
    let actions = client.get_available_actions().await?;
    
    // Check that the expected actions are available
    assert!(actions.contains(&"capture_screenshot".to_string()));
    assert!(actions.contains(&"record_interaction".to_string()));
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}
