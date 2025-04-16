pub mod capture;
pub mod mcp;

// Re-export main components for easier use in tests
pub use capture::CaptureOptions;
pub use mcp::{MCPServer, MCPClient};
