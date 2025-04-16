pub mod capture;

// MCP module is only available when the mcp_experimental feature is enabled
#[cfg(feature = "mcp_experimental")]
pub mod mcp;

// Re-export main components for easier use in tests
pub use capture::CaptureOptions;

// Re-export MCP components only when the feature is enabled
#[cfg(feature = "mcp_experimental")]
pub use mcp::{MCPServer, MCPClient};
