# WebLook

A command-line tool for capturing screenshots and recordings of web pages.

## Why This Tool Exists

I created WebLook to support my local development workflow, particularly when testing Amazon Q CLI and MCP (Model Context Protocol) applications. I needed a lightweight, read-only web tool with sane defaults that could quickly capture visual states of locally-running web applications without complex configuration. WebLook is designed to be simple, efficient, and integrate seamlessly into development pipelines with minimal overhead.

## Features

- Take screenshots of web pages
- Create animated GIF recordings of web pages
- Configurable wait time before capture
- Configurable window size
- Configurable recording length
- Support for input/output piping
- Headless operation
- Execute custom JavaScript before capture
- Capture browser console logs
- Automatic user-agent rotation (Windows/Mac Chrome)
- Automatic ChromeDriver management
- Colorful progress indicators with countdown timers
- **[EXPERIMENTAL] MCP (Model Context Protocol) integration** for AI model interaction

## Usage

```
weblook [OPTIONS] [URL]
```

### Options

- `--output, -o <FILE>`: Specify output file (default: weblook.png or weblook.gif)
- `--wait, -w <SECONDS>`: Wait time before capture (default: 10 seconds)
- `--record, -r [SECONDS]`: Create a recording instead of screenshot (default length: 10 seconds)
- `--size, -s <WIDTHxHEIGHT>`: Set viewport size (default: 1280x720)
- `--js, -j <CODE>`: Execute JavaScript code before capture
- `--console-log <FILE>`: Capture browser console logs and save to specified file
- `--debug, -d`: Enable debug output (shows ChromeDriver messages)
- `--mcp-server <HOST:PORT>`: [EXPERIMENTAL] Start as MCP server on specified address
- `--mcp-client <URL>`: [EXPERIMENTAL] Connect to MCP server at specified URL
- `--help, -h`: Show help information

### Examples

```bash
# Take a screenshot of the default URL (127.0.0.1:8080)
weblook

# Take a screenshot of a specific URL
weblook https://example.com

# Take a screenshot after waiting 5 seconds
weblook --wait 5 https://example.com

# Create a 5-second recording
weblook --record 5 https://example.com

# Set viewport size to 1920x1080
weblook --size 1920x1080 https://example.com

# Execute JavaScript before capture
weblook --js "document.body.style.backgroundColor = 'red';" https://example.com

# Capture console logs to a file
weblook --console-log console.log https://example.com

# Pipe URL input and output to another command
echo "https://example.com" | weblook --output - | other-command

# Save output to a specific file
weblook https://example.com --output screenshot.png

# Show debug output
weblook --debug https://example.com

# [EXPERIMENTAL] Start as an MCP server
weblook --mcp-server 127.0.0.1:8000

# [EXPERIMENTAL] Use as an MCP client
weblook --mcp-client http://localhost:8000 https://example.com
```

## Installation

WebLook is currently not available on crates.io. To install:

```bash
# Clone the repository
git clone https://github.com/username/weblook.git
cd weblook

# Build and install without experimental MCP support
cargo build --release

# Build with experimental MCP support
cargo build --release --features mcp_experimental

# The binary will be available at target/release/weblook
# You can copy it to a directory in your PATH for easier access
cp target/release/weblook ~/.local/bin/  # or another directory in your PATH
```

## Requirements

- ChromeDriver must be installed
  - Install ChromeDriver: `sudo apt install chromium-chromedriver` (Ubuntu/Debian)
  - The application will automatically start and stop ChromeDriver as needed

## Experimental Features

### MCP (Model Context Protocol) Integration

The MCP integration is currently experimental and requires compiling with the `mcp_experimental` feature flag. This feature allows WebLook to:

1. Act as an MCP server that other applications can connect to
2. Act as an MCP client that can connect to other MCP servers

To enable MCP support:

```bash
cargo build --features mcp_experimental
```

## License

GPL-3.0

Copyright (C) 2025 Kelsea Blackwell
