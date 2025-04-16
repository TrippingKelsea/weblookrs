use anyhow::Result;
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::time::sleep;

/// Test that the CLI can start in server mode and respond to requests
#[tokio::test]
async fn test_cli_server_mode() -> Result<()> {
    // Start the CLI in server mode on a random port
    let port = 9879;
    let addr = format!("127.0.0.1:{}", port);
    
    let mut child = Command::new("cargo")
        .args(["run", "--", "--mcp-server", &addr])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // Give the server time to start
    sleep(Duration::from_secs(2)).await;
    
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
    
    // Use the CLI in client mode to connect to the server
    let output_file = NamedTempFile::new()?;
    let output_path = output_file.path().to_str().unwrap();
    
    let status = Command::new("cargo")
        .args([
            "run", "--",
            "--mcp-client", &format!("http://{}", addr),
            "--output", output_path,
            format!("file://{}", html_path.display()).as_str(),
        ])
        .status()?;
    
    assert!(status.success());
    
    // Check that the output file exists and is not empty
    let metadata = std::fs::metadata(output_path)?;
    assert!(metadata.len() > 0);
    
    // Kill the server process
    child.kill()?;
    
    Ok(())
}

/// Test that the CLI can connect to an existing MCP server
#[tokio::test]
async fn test_cli_client_mode() -> Result<()> {
    // Start a server using the MCPServer directly
    let port = 9880;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    
    let rt = tokio::runtime::Runtime::new()?;
    let server_handle = rt.spawn(async move {
        let mut server = weblook::mcp::MCPServer::new();
        let _ = server.start(addr).await;
        
        // Keep the server running for the duration of the test
        sleep(Duration::from_secs(10)).await;
        
        let _ = server.stop().await;
    });
    
    // Give the server time to start
    sleep(Duration::from_secs(2)).await;
    
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
    
    // Use the CLI in client mode to connect to the server
    let output_file = NamedTempFile::new()?;
    let output_path = output_file.path().to_str().unwrap();
    
    let status = Command::new("cargo")
        .args([
            "run", "--",
            "--mcp-client", &format!("http://127.0.0.1:{}", port),
            "--output", output_path,
            format!("file://{}", html_path.display()).as_str(),
        ])
        .status()?;
    
    assert!(status.success());
    
    // Check that the output file exists and is not empty
    let metadata = std::fs::metadata(output_path)?;
    assert!(metadata.len() > 0);
    
    // Cancel the server task
    server_handle.abort();
    
    Ok(())
}

/// Test the end-to-end flow with recording
#[tokio::test]
async fn test_end_to_end_recording() -> Result<()> {
    // Start the CLI in server mode on a random port
    let port = 9881;
    let addr = format!("127.0.0.1:{}", port);
    
    let mut child = Command::new("cargo")
        .args(["run", "--", "--mcp-server", &addr])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // Give the server time to start
    sleep(Duration::from_secs(2)).await;
    
    // Create an animated HTML file to test with
    let html_content = r#"<!DOCTYPE html>
    <html>
    <head>
        <title>Animated Test</title>
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
    
    // Use the CLI in client mode to connect to the server and create a recording
    let output_file = NamedTempFile::new()?;
    let output_path = output_file.path().to_str().unwrap();
    
    let status = Command::new("cargo")
        .args([
            "run", "--",
            "--mcp-client", &format!("http://{}", addr),
            "--record", "2",
            "--output", output_path,
            format!("file://{}", html_path.display()).as_str(),
        ])
        .status()?;
    
    assert!(status.success());
    
    // Check that the output file exists and is not empty
    let metadata = std::fs::metadata(output_path)?;
    assert!(metadata.len() > 0);
    
    // Verify that the file is a GIF by checking the magic number
    let mut file = std::fs::File::open(output_path)?;
    let mut buffer = [0; 6];
    file.read_exact(&mut buffer)?;
    assert_eq!(&buffer[0..6], b"GIF89a");
    
    // Kill the server process
    child.kill()?;
    
    Ok(())
}
