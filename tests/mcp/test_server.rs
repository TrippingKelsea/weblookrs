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

/// Test the capture_screenshot action with minimal parameters
#[tokio::test]
async fn test_capture_screenshot_minimal() -> Result<()> {
    // Create a server on a specific port
    let port = 9877;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a client to connect to the server
    let client = weblook::mcp::MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Invoke the capture_screenshot action
    let params = serde_json::json!({
        "url": "http://example.com",
        "wait": 1
    });
    
    let response = client.invoke_action("capture_screenshot", params).await?;
    
    // Verify the response
    assert!(response.get("image_data").is_some());
    assert_eq!(response.get("format").and_then(|v| v.as_str()), Some("png"));
    
    // Decode the image data to verify it's valid
    let image_data = response["image_data"].as_str().unwrap();
    let decoded = base64::decode(image_data)?;
    assert!(!decoded.is_empty());
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}

/// Test the record_interaction action with minimal parameters
#[tokio::test]
async fn test_record_interaction_minimal() -> Result<()> {
    // Create a server on a specific port
    let port = 9878;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    
    // Start the server
    server.start(addr).await?;
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Create a client to connect to the server
    let client = weblook::mcp::MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Invoke the record_interaction action with a short duration
    let params = serde_json::json!({
        "url": "http://example.com",
        "wait": 1,
        "duration": 2
    });
    
    let response = client.invoke_action("record_interaction", params).await?;
    
    // Verify the response
    assert!(response.get("image_data").is_some());
    assert_eq!(response.get("format").and_then(|v| v.as_str()), Some("gif"));
    
    // Decode the image data to verify it's valid
    let image_data = response["image_data"].as_str().unwrap();
    let decoded = base64::decode(image_data)?;
    assert!(!decoded.is_empty());
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}
