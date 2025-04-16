
# WebLook - Web Page Screenshot/Recording Tool

## Requirements

1. **Core Functionality**:
   - Open a given URL in a headless browser
   - Wait a specified amount of time before capturing
   - Take either a static screenshot or create a short animated GIF recording
   - Run as efficiently as possible in headless mode

2. **Default Parameters**:
   - Default URL: 127.0.0.1:8080
   - Default wait time: 10 seconds
   - Default recording length: 10 seconds
   - Default output: weblook.png (for screenshot) or weblook.gif (for recording)

3. **Input/Output Options**:
   - Accept URL as a command-line parameter
   - Accept URL as piped input
   - Support saving output to a specified file
   - Support piping output to stdout for use in pipelines

4. **Configuration Options**:
   - Configurable window size for the browser viewport
   - Configurable wait time before capture
   - Configurable recording length for GIFs

5. **Experimental Features**:
   - MCP (Model Context Protocol) integration as an optional feature
   - Enabled via the `mcp_experimental` feature flag
   - Support for both server and client modes

## Implementation Approaches

### Approach 1: Headless Chrome with WebDriver

**Description**: Use WebDriver protocol with a headless Chrome/Chromium browser.

**Components**:
- Rust WebDriver client library (e.g., `thirtyfour`, `fantoccini`)
- Chrome/Chromium browser (installed on the system)
- Image processing library for screenshots (e.g., `image`)
- GIF creation library (e.g., `gif`)

**Pros**:
- Full browser rendering ensures accurate representation of modern web pages
- WebDriver is a standardized protocol with good support
- Can execute JavaScript and interact with the page if needed

**Cons**:
- Requires Chrome/Chromium to be installed
- Potentially higher resource usage
- More complex setup and dependencies

### Approach 2: Lightweight Headless Browser Library

**Description**: Use a pure Rust headless browser library without external dependencies.

**Components**:
- Headless browser library (e.g., `headless_chrome`, `rust-headless-chrome`)
- Image processing libraries for capturing and creating GIFs

**Pros**:
- Potentially lighter weight than full WebDriver approach
- Fewer external dependencies
- Possibly faster startup time

**Cons**:
- May still require Chrome binaries
- Potentially less standardized than WebDriver
- May have fewer features or less compatibility with complex web pages

### Approach 3: HTTP Client with HTML Renderer

**Description**: Use an HTTP client to fetch content and render it with a HTML/CSS renderer.

**Components**:
- HTTP client (e.g., `reqwest`)
- HTML/CSS renderer (e.g., `lol_html`, custom renderer)
- WebKit or similar rendering engine wrapper
- Image processing libraries

**Pros**:
- Potentially the lightest weight solution
- No browser dependency
- Fastest execution time

**Cons**:
- Limited JavaScript support
- May not render complex modern websites correctly
- Less accurate representation of how pages appear to users

Each approach offers different trade-offs between accuracy, resource usage, dependencies, and complexity. The best choice depends on the specific requirements for accuracy of rendering, resource constraints, and deployment environment.

## Feature Flags

1. **mcp_experimental**:
   - Enables the experimental MCP (Model Context Protocol) integration
   - When enabled, adds MCP server and client functionality
   - Disabled by default to keep the core tool lightweight
   - Can be enabled with `cargo build --features mcp_experimental`
