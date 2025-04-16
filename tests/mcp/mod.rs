// MCP tests are only available when the mcp_experimental feature is enabled
#[cfg(feature = "mcp_experimental")]
mod test_server;

#[cfg(feature = "mcp_experimental")]
mod test_client;

#[cfg(feature = "mcp_experimental")]
mod test_integration;
