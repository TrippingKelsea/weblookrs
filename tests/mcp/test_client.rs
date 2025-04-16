use anyhow::Result;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

use weblook::mcp::{MCPClient, MCPServer};

/// Test that the client can connect to a server and get available actions
#[tokio::test]
async fn test_client_get_actions() -> Result<()> {
    // Create a server on a specific port
    let port = 9879;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a client to connect to the server
    let client = MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Get available actions
    let actions = client.get_available_actions().await?;
    
    // Check that the expected actions are available
    assert!(actions.contains(&"capture_screenshot".to_string()));
    assert!(actions.contains(&"record_interaction".to_string()));
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}

/// Test that the client can invoke an action and handle the response
#[tokio::test]
async fn test_client_invoke_action() -> Result<()> {
    // Create a server on a specific port
    let port = 9880;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a client to connect to the server
    let client = MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Invoke the capture_screenshot action
    let params = json!({
        "url": "http://example.com",
        "wait": 1
    });
    
    let response = client.invoke_action("capture_screenshot", params).await?;
    
    // Verify the response
    assert!(response.get("image_data").is_some());
    assert_eq!(response.get("format").and_then(|v| v.as_str()), Some("png"));
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}

/// Test that the client handles errors correctly
#[tokio::test]
async fn test_client_error_handling() -> Result<()> {
    // Create a server on a specific port
    let port = 9881;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a client to connect to the server
    let client = MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Invoke an invalid action
    let params = json!({
        "url": "http://example.com"
    });
    
    // The invocation should fail
    let result = client.invoke_action("invalid_action", params).await;
    assert!(result.is_err());
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}
