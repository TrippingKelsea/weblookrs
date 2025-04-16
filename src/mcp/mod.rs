// Use our mock SDK implementation for now
mod mock_sdk;
pub use mock_sdk as mcp_sdk;

pub mod server;
pub mod client;
pub mod actions;

// Re-export main components
pub use server::MCPServer;
pub use client::MCPClient;
pub use actions::{register_actions, ContextActionHandler};
