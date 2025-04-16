use anyhow::Result;
use mcp_sdk::client::{Client, ClientConfig};
use serde_json::Value;
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
    let config = ClientConfig::new()
        .with_endpoint(&format!("http://127.0.0.1:{}", port))
        .with_timeout(Duration::from_secs(5))
        .with_auth_disabled();
    
    let client = Client::new(config).await?;
    
    // Get available actions
    let actions = client.get_available_actions().await?;
    
    // Check that the expected actions are available
    let action_names: Vec<String> = actions.into_iter().map(|a| a.name).collect();
    assert!(action_names.contains(&"capture_screenshot".to_string()));
    assert!(action_names.contains(&"record_interaction".to_string()));
    
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
    let config = ClientConfig::new()
        .with_endpoint(&format!("http://127.0.0.1:{}", port))
        .with_timeout(Duration::from_secs(30))
        .with_auth_disabled();
    
    let client = Client::new(config).await?;
    
    // Create a simple HTML file to test with
    let html_content = r#"<!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
    </head>
    <body>
        <h1>Hello, WebLook!</h1>
    </body>
    </html>"#;
    
    let temp_dir = tempfile::tempdir()?;
    let html_path = temp_dir.path().join("test.html");
    std::fs::write(&html_path, html_content)?;
    
    // Invoke the capture_screenshot action
    let params = serde_json::json!({
        "url": format!("file://{}", html_path.display()),
        "wait": 1
    });
    
    let response = client.invoke_action("capture_screenshot", params).await?;
    
    // Verify the response
    assert!(response.get("image_data").is_some());
    assert_eq!(response.get("format").and_then(Value::as_str), Some("png"));
    
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
    let config = ClientConfig::new()
        .with_endpoint(&format!("http://127.0.0.1:{}", port))
        .with_timeout(Duration::from_secs(30))
        .with_auth_disabled();
    
    let client = Client::new(config).await?;
    
    // Create a simple HTML file to test with
    let html_content = r#"<!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
        <style>
            @keyframes colorChange {
                0% { background-color: red; }
                50% { background-color: blue; }
                100% { background-color: green; }
            }
            body {
                animation: colorChange 2s infinite;
            }
        </style>
    </head>
    <body>
        <h1>Animated Test</h1>
    </body>
    </html>"#;
    
    let temp_dir = tempfile::tempdir()?;
    let html_path = temp_dir.path().join("animated.html");
    std::fs::write(&html_path, html_content)?;
    
    // Invoke the record_interaction action with a short duration
    let params = serde_json::json!({
        "url": format!("file://{}", html_path.display()),
        "wait": 1,
        "duration": 2
    });
    
    let response = client.invoke_action("record_interaction", params).await?;
    
    // Verify the response
    assert!(response.get("image_data").is_some());
    assert_eq!(response.get("format").and_then(Value::as_str), Some("gif"));
    
    // Decode the image data to verify it's valid
    let image_data = response["image_data"].as_str().unwrap();
    let decoded = base64::decode(image_data)?;
    assert!(!decoded.is_empty());
    
    // Stop the server
    server.stop().await?;
    
    Ok(())
}
