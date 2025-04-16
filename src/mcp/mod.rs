// MCP (Model Context Protocol) - EXPERIMENTAL FEATURE
//
// This module provides experimental support for the Model Context Protocol,
// which allows WebLook to interact with AI models and other MCP-compatible services.
// 
// This feature is currently experimental and may change significantly in future releases.
// To enable MCP support, compile with the `mcp_experimental` feature flag:
//
// cargo build --features mcp_experimental
//
// Note: The MCP implementation currently uses a mock SDK for development purposes.

// Use our mock SDK implementation for now
pub mod mock_sdk;
pub use mock_sdk as mcp_sdk;

pub mod server;
pub mod client;
pub mod actions;

// Re-export main components
pub use server::MCPServer;
pub use client::MCPClient;
