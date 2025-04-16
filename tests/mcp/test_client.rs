use anyhow::Result;
use mockito::{mock, server_url};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

use weblook::mcp::MCPClient;

/// Test that the client can connect to a server and get available actions
#[tokio::test]
async fn test_client_get_actions() -> Result<()> {
    // Set up a mock server
    let _m = mock("GET", "/actions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[
            {"name": "capture_screenshot", "description": "Capture a screenshot of a web page"},
            {"name": "record_interaction", "description": "Record an animated GIF of a web page"}
        ]"#)
        .create();
    
    // Create a client to connect to the mock server
    let client = MCPClient::new(&server_url()).await?;
    
    // Get available actions
    let actions = client.get_available_actions().await?;
    
    // Check that the expected actions are available
    assert_eq!(actions.len(), 2);
    assert!(actions.contains(&"capture_screenshot".to_string()));
    assert!(actions.contains(&"record_interaction".to_string()));
    
    Ok(())
}

/// Test that the client can invoke an action and handle the response
#[tokio::test]
async fn test_client_invoke_action() -> Result<()> {
    // Set up a mock server
    let _m = mock("POST", "/actions/capture_screenshot")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "image_data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
            "format": "png"
        }"#)
        .create();
    
    // Create a client to connect to the mock server
    let client = MCPClient::new(&server_url()).await?;
    
    // Invoke the capture_screenshot action
    let params = json!({
        "url": "https://example.com",
        "wait": 1
    });
    
    let response = client.invoke_action("capture_screenshot", params).await?;
    
    // Verify the response
    assert!(response.get("image_data").is_some());
    assert_eq!(response.get("format").and_then(|v| v.as_str()), Some("png"));
    
    Ok(())
}

/// Test that the client handles errors correctly
#[tokio::test]
async fn test_client_error_handling() -> Result<()> {
    // Set up a mock server that returns an error
    let _m = mock("POST", "/actions/invalid_action")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "error": "Action not found",
            "code": "ACTION_NOT_FOUND"
        }"#)
        .create();
    
    // Create a client to connect to the mock server
    let client = MCPClient::new(&server_url()).await?;
    
    // Invoke an invalid action
    let params = json!({
        "url": "https://example.com"
    });
    
    // The invocation should fail
    let result = client.invoke_action("invalid_action", params).await;
    assert!(result.is_err());
    
    Ok(())
}

/// Test that the client handles timeouts correctly
#[tokio::test]
async fn test_client_timeout() -> Result<()> {
    // Set up a mock server that delays the response
    let _m = mock("POST", "/actions/slow_action")
        .with_status(200)
        .with_delay(Duration::from_secs(5)) // 5 second delay
        .with_header("content-type", "application/json")
        .with_body(r#"{"result": "ok"}"#)
        .create();
    
    // Create a client with a short timeout
    let client = MCPClient::new(&server_url()).await?;
    
    // Invoke the slow action
    let params = json!({
        "url": "https://example.com"
    });
    
    // The invocation should time out
    let result = client.invoke_action("slow_action", params).await;
    
    // We expect an error due to timeout
    assert!(result.is_err());
    
    Ok(())
}
