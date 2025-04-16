use anyhow::{Context, Result};
use base64::Engine;
use clap::Parser;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use url::Url;

mod capture;
#[cfg(feature = "mcp_experimental")]
mod mcp;

use capture::CaptureOptions;

#[derive(Parser, Debug)]
#[command(author, version, about = "Capture screenshots and recordings of web pages")]
struct Args {
    /// URL to capture (default: http://127.0.0.1:8080)
    #[arg(index = 1)]
    url: Option<String>,

    /// Output file path (default: weblook.png or weblook.gif)
    #[arg(short, long)]
    output: Option<String>,

    /// Wait time before capture in seconds (default: 10)
    #[arg(short, long, default_value = "10")]
    wait: u64,

    /// Create a recording instead of screenshot (value is length in seconds)
    #[arg(short, long)]
    record: Option<Option<u64>>,

    /// Set viewport size (format: WIDTHxHEIGHT, default: 1280x720)
    #[arg(short, long, default_value = "1280x720")]
    size: String,

    /// Execute custom JavaScript before capture
    #[arg(short = 'j', long)]
    js: Option<String>,
    
    /// Capture browser console logs and save to specified file
    #[arg(long = "console-log")]
    console_log: Option<String>,
    
    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
    
    /// [EXPERIMENTAL] Start as MCP server on specified address (format: host:port)
    #[cfg(feature = "mcp_experimental")]
    #[arg(long)]
    mcp_server: Option<String>,
    
    /// [EXPERIMENTAL] Connect to MCP server at specified URL
    #[cfg(feature = "mcp_experimental")]
    #[arg(long)]
    mcp_client: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Check if we're running in MCP server mode
    #[cfg(feature = "mcp_experimental")]
    if let Some(addr_str) = args.mcp_server {
        return run_mcp_server(addr_str).await;
    }
    
    // Check if we're running in MCP client mode
    #[cfg(feature = "mcp_experimental")]
    if let Some(ref endpoint) = args.mcp_client {
        return run_mcp_client(endpoint.clone(), &args).await;
    }
    
    // Normal capture mode
    run_capture(args).await
}

async fn run_capture(args: Args) -> Result<()> {
    // Handle piped input for URL
    let url_str = if args.url.is_none() && !atty::is(atty::Stream::Stdin) {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input.trim().to_string()
    } else {
        args.url.unwrap_or_else(|| "http://127.0.0.1:8080".to_string())
    };
    
    // Parse URL
    let _url = Url::parse(&url_str).context("Failed to parse URL")?;
    
    // Determine if we're recording and for how long
    let is_recording = args.record.is_some();
    let recording_length = args.record.flatten();
    
    // Determine output path
    let output_path = determine_output_path(args.output, is_recording)?;
    
    // Set up capture options
    let options = CaptureOptions {
        url: url_str,
        output_path,
        wait: args.wait,
        size: args.size,
        js: args.js,
        debug: args.debug,
        is_recording,
        recording_length,
        console_log: args.console_log,
    };
    
    // Perform capture
    capture::perform_capture(options).await
}

#[cfg(feature = "mcp_experimental")]
async fn run_mcp_server(addr_str: String) -> Result<()> {
    // Parse socket address
    let addr: SocketAddr = addr_str.parse()
        .context("Invalid MCP server address format. Expected format: host:port")?;
    
    println!("Starting MCP server on {}... (EXPERIMENTAL FEATURE)", addr);
    
    // Create and start MCP server
    let mut server = mcp::MCPServer::new();
    server.start(addr).await?;
    
    println!("MCP server started. Press Ctrl+C to stop.");
    
    // Wait for Ctrl+C
    signal::ctrl_c().await?;
    
    println!("Stopping MCP server...");
    server.stop().await?;
    println!("MCP server stopped.");
    
    Ok(())
}

#[cfg(feature = "mcp_experimental")]
async fn run_mcp_client(endpoint: String, args: &Args) -> Result<()> {
    println!("Connecting to MCP server at {}... (EXPERIMENTAL FEATURE)", endpoint);
    
    // Create MCP client
    let client = mcp::MCPClient::new(&endpoint).await?;
    
    // Get available actions
    let actions = client.get_available_actions().await?;
    println!("Available actions: {:?}", actions);
    
    // Determine if we're recording or taking a screenshot
    let is_recording = args.record.is_some();
    
    if is_recording {
        // Invoke record_interaction action
        let params = serde_json::json!({
            "url": args.url.clone().unwrap_or_else(|| "http://127.0.0.1:8080".to_string()),
            "duration": args.record.flatten().unwrap_or(10),
            "wait": args.wait,
            "size": args.size,
            "js": args.js,
        });
        
        println!("Invoking record_interaction action...");
        let response = client.invoke_action("record_interaction", params).await?;
        
        // Handle response
        if let Some(image_data) = response["image_data"].as_str() {
            // Decode base64 data
            let decoded = base64::engine::general_purpose::STANDARD.decode(image_data)?;
            
            // Determine output path
            let output_path = determine_output_path(args.output.clone(), true)?;
            
            // Write to file or stdout
            if output_path.to_str() == Some("-") {
                io::stdout().write_all(&decoded)?;
            } else {
                std::fs::write(&output_path, decoded)?;
                println!("Recording saved to {}", output_path.display());
            }
        } else {
            println!("Error: No image data in response");
        }
    } else {
        // Invoke capture_screenshot action
        let params = serde_json::json!({
            "url": args.url.clone().unwrap_or_else(|| "http://127.0.0.1:8080".to_string()),
            "wait": args.wait,
            "size": args.size,
            "js": args.js,
        });
        
        println!("Invoking capture_screenshot action...");
        let response = client.invoke_action("capture_screenshot", params).await?;
        
        // Handle response
        if let Some(image_data) = response["image_data"].as_str() {
            // Decode base64 data
            let decoded = base64::engine::general_purpose::STANDARD.decode(image_data)?;
            
            // Determine output path
            let output_path = determine_output_path(args.output.clone(), false)?;
            
            // Write to file or stdout
            if output_path.to_str() == Some("-") {
                io::stdout().write_all(&decoded)?;
            } else {
                std::fs::write(&output_path, decoded)?;
                println!("Screenshot saved to {}", output_path.display());
            }
        } else {
            println!("Error: No image data in response");
        }
    }
    
    Ok(())
}

fn determine_output_path(output: Option<String>, is_recording: bool) -> Result<PathBuf> {
    match output {
        Some(path) => {
            if path == "-" {
                // Output to stdout
                Ok(PathBuf::from("-"))
            } else {
                Ok(PathBuf::from(path))
            }
        },
        None => {
            // Default output path
            if is_recording {
                Ok(PathBuf::from("weblook.gif"))
            } else {
                Ok(PathBuf::from("weblook.png"))
            }
        }
    }
}
