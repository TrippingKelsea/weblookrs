use anyhow::Result;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

use weblook::mcp::{MCPClient, MCPServer};

/// Test basic integration between server and client
#[tokio::test]
async fn test_basic_integration() -> Result<()> {
    // Start a server
    let port = 9882;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let mut server = MCPServer::new();
    server.start(addr).await?;
    
    // Give the server time to start
    sleep(Duration::from_secs(1)).await;
    
    // Create a client to connect to the server
    let client = MCPClient::new(&format!("http://127.0.0.1:{}", port)).await?;
    
    // Get available actions
    let actions = client.get_available_actions().await?;
    assert!(actions.contains(&"capture_screenshot".to_string()));
    assert!(actions.contains(&"record_interaction".to_string()));
    
    // Invoke the capture_screenshot action
    let params = serde_json::json!({
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
