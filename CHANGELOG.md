# Changelog

## [Unreleased]

### Added
- Experimental MCP (Model Context Protocol) integration
- New feature flag `mcp_experimental` to enable/disable MCP functionality at compile time
- Documentation for MCP experimental features in README.md
- MCP server and client functionality (when compiled with the feature flag)

### Changed
- MCP-related command line options are now marked as experimental
- MCP-related code is now conditionally compiled only when the feature flag is enabled
- Updated documentation to reflect the experimental status of MCP features

### Developer Notes
- MCP tests are now conditionally compiled with the feature flag
- Added feature flag documentation in AmazonQ.md
