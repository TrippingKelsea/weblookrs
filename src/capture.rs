use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use thirtyfour::{ChromeCapabilities, WebDriver, ChromiumLikeCapabilities};
use tokio::time::sleep;
use url::Url;
use rand::Rng;
use std::net::TcpStream;

/// Options for capturing web content
pub struct CaptureOptions {
    pub url: String,
    pub output_path: PathBuf,
    pub wait: u64,
    pub size: String,
    pub js: Option<String>,
    pub debug: bool,
    pub is_recording: bool,
    pub recording_length: Option<u64>,
}

/// Viewport size representation
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

impl std::str::FromStr for ViewportSize {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid viewport size format. Expected WIDTHxHEIGHT"));
        }

        let width = parts[0].parse::<u32>()
            .context("Failed to parse viewport width")?;
        let height = parts[1].parse::<u32>()
            .context("Failed to parse viewport height")?;

        Ok(ViewportSize { width, height })
    }
}

// User agent strings for rotation
const USER_AGENTS: [&str; 2] = [
    // Chrome on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    // Chrome on Mac
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
];

// ChromeDriver management
pub struct ChromeDriverManager {
    process: Option<Child>,
    port: u16,
    debug: bool,
}

impl ChromeDriverManager {
    pub fn new(port: u16, debug: bool) -> Self {
        ChromeDriverManager {
            process: None,
            port,
            debug,
        }
    }

    pub fn is_running(&self) -> bool {
        TcpStream::connect(format!("127.0.0.1:{}", self.port)).is_ok()
    }

    pub fn start(&mut self) -> Result<()> {
        if self.is_running() {
            if self.debug {
                println!("ChromeDriver is already running on port {}", self.port);
            }
            return Ok(());
        }

        if self.debug {
            println!("Starting ChromeDriver on port {}...", self.port);
        }
        
        let process = if self.debug {
            Command::new("chromedriver")
                .arg(format!("--port={}", self.port))
                .spawn()
                .context("Failed to start ChromeDriver. Make sure it's installed.")?
        } else {
            Command::new("chromedriver")
                .arg(format!("--port={}", self.port))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("Failed to start ChromeDriver. Make sure it's installed.")?
        };

        self.process = Some(process);

        // Wait for ChromeDriver to start
        let start_time = std::time::Instant::now();
        while !self.is_running() {
            if start_time.elapsed() > Duration::from_secs(5) {
                return Err(anyhow::anyhow!("Timed out waiting for ChromeDriver to start"));
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if self.debug {
            println!("ChromeDriver started successfully");
        }
        Ok(())
    }
}

impl Drop for ChromeDriverManager {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            if self.debug {
                println!("Stopping ChromeDriver...");
            }
            let _ = process.kill();
            let _ = process.wait();
            if self.debug {
                println!("ChromeDriver stopped");
            }
        }
    }
}

/// Main capture function that handles both screenshots and recordings
pub async fn perform_capture(options: CaptureOptions) -> Result<()> {
    // Determine if we're outputting to stdout
    let is_piped = options.output_path.to_str() == Some("-");
    
    // Start ChromeDriver if not already running
    let chromedriver_port = 9515;
    let mut chromedriver = ChromeDriverManager::new(chromedriver_port, options.debug);
    chromedriver.start()?;

    // Parse URL
    let url = Url::parse(&options.url).context("Failed to parse URL")?;

    // Parse viewport size
    let viewport = options.size.parse::<ViewportSize>()?;

    // Determine recording length if recording
    let recording_length = if options.is_recording {
        options.recording_length.unwrap_or(10)
    } else {
        0
    };

    if !is_piped && !options.debug {
        eprintln!("{}", "Starting WebLook...".bright_cyan());
        if options.is_recording {
            eprintln!("{} {}", "•".yellow(), format!("Recording {} for {} seconds", url, recording_length).yellow());
        } else {
            eprintln!("{} {}", "•".yellow(), format!("Taking screenshot of {}", url).yellow());
        }
        std::io::stderr().flush().ok();
    }
    
    // Set up WebDriver
    let driver = setup_webdriver(viewport, chromedriver_port).await?;
    
    // Navigate to URL and wait
    navigate_and_wait(&driver, url, Duration::from_secs(options.wait), is_piped, options.debug).await?;
    
    // Execute JavaScript if provided
    if let Some(js_code) = &options.js {
        execute_javascript(&driver, js_code).await?;
    }
    
    // Capture screenshot or recording
    if options.is_recording {
        create_recording(&driver, recording_length, &options.output_path, is_piped, options.debug).await?;
    } else {
        take_screenshot(&driver, &options.output_path, is_piped, options.debug).await?;
    }
    
    // Clean up
    driver.quit().await?;
    
    // ChromeDriver will be automatically stopped by the Drop implementation
    
    Ok(())
}

async fn setup_webdriver(viewport: ViewportSize, port: u16) -> Result<WebDriver> {
    let mut caps = ChromeCapabilities::new();
    
    // Select a random user agent
    let mut rng = rand::thread_rng();
    let user_agent = USER_AGENTS[rng.gen_range(0..USER_AGENTS.len())];
    
    // Configure headless mode and user agent
    caps.add_arg("--headless=new")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg(&format!("--window-size={},{}", viewport.width, viewport.height))?;
    caps.add_arg(&format!("--user-agent={}", user_agent))?;
    
    // Connect to WebDriver
    let driver = WebDriver::new(&format!("http://localhost:{}", port), caps).await?;
    
    // Set viewport size
    driver.set_window_rect(0, 0, viewport.width, viewport.height).await?;
    
    Ok(driver)
}

async fn navigate_and_wait(driver: &WebDriver, url: Url, wait_time: Duration, is_piped: bool, debug: bool) -> Result<()> {
    // Navigate to the URL
    driver.goto(url.as_str()).await?;
    
    // Wait for the specified time with a nice countdown
    if !is_piped {
        // Force flush stdout to ensure messages appear
        eprintln!("Page loaded. Waiting for {} seconds...", wait_time.as_secs());
        std::io::stderr().flush().ok();
        
        display_countdown(wait_time, "Loading page", debug).await;
    } else {
        sleep(wait_time).await;
    }
    
    Ok(())
}

// Display a colorful countdown timer
async fn display_countdown(duration: Duration, message: &str, debug: bool) {
    if !debug {
        eprintln!("Starting countdown: {} for {} seconds", message, duration.as_secs());
        std::io::stderr().flush().ok();
        
        let pb = ProgressBar::new(duration.as_secs());
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}s")
            .unwrap()
            .progress_chars("#>-"));
        
        pb.set_message(message.bright_green().to_string());
        
        for i in 1..=duration.as_secs() {
            // Rainbow text for the countdown
            match i % 6 {
                0 => pb.set_message(message.red().to_string()),
                1 => pb.set_message(message.yellow().to_string()),
                2 => pb.set_message(message.green().to_string()),
                3 => pb.set_message(message.cyan().to_string()),
                4 => pb.set_message(message.blue().to_string()),
                _ => pb.set_message(message.magenta().to_string()),
            }
            
            pb.set_position(i);
            sleep(Duration::from_secs(1)).await;
        }
        
        pb.finish_with_message(format!("{} complete!", message).green().to_string());
    } else {
        eprintln!("Waiting for {} seconds...", duration.as_secs());
        sleep(duration).await;
    }
}

async fn execute_javascript(driver: &WebDriver, js_code: &str) -> Result<()> {
    // Execute the JavaScript code
    driver.execute(js_code, vec![]).await?;
    
    // Give a short time for any JS effects to complete
    sleep(Duration::from_millis(500)).await;
    
    Ok(())
}

async fn take_screenshot(driver: &WebDriver, output_path: &PathBuf, is_piped: bool, debug: bool) -> Result<()> {
    // Take screenshot
    if !is_piped && !debug {
        eprintln!("{}", "Taking screenshot...".bright_cyan());
        std::io::stderr().flush().ok();
    }
    
    let screenshot = driver.screenshot_as_png().await?;
    
    // Handle output
    if output_path.to_str() == Some("-") {
        // Write to stdout
        io::stdout().write_all(&screenshot)?;
    } else {
        // Write to file
        std::fs::write(output_path, screenshot)?;
        
        if !is_piped && !debug {
            eprintln!("{} {}", "✓".green(), format!("Screenshot saved to {}", output_path.display()).bright_green());
            std::io::stderr().flush().ok();
        } else if !is_piped && debug {
            eprintln!("Screenshot saved to {}", output_path.display());
        }
    }
    
    Ok(())
}

async fn create_recording(driver: &WebDriver, duration_secs: u64, output_path: &PathBuf, is_piped: bool, debug: bool) -> Result<()> {
    // Create a temporary directory for frames
    let temp_dir = tempfile::tempdir()?;
    let frames_per_second = 10;
    let total_frames = duration_secs * frames_per_second;
    let frame_delay = Duration::from_millis(1000 / frames_per_second);
    
    // Capture frames
    let mut frames = Vec::new();
    
    if !is_piped {
        if !debug {
            eprintln!("Starting recording for {} seconds...", duration_secs);
            std::io::stderr().flush().ok();
            
            let pb = ProgressBar::new(duration_secs);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}s")
                .unwrap()
                .progress_chars("#>-"));
            
            pb.set_message("Recording".bright_green().to_string());
            
            for i in 0..total_frames {
                // Take screenshot
                let screenshot_data = driver.screenshot_as_png().await?;
                let frame_path = temp_dir.path().join(format!("frame_{:04}.png", i));
                std::fs::write(&frame_path, screenshot_data)?;
                frames.push(frame_path);
                
                // Update progress bar with rainbow colors every second
                if i % frames_per_second == 0 {
                    let current_second = i / frames_per_second;
                    pb.set_position(current_second + 1);
                    
                    match current_second % 6 {
                        0 => pb.set_message("Recording".red().to_string()),
                        1 => pb.set_message("Recording".yellow().to_string()),
                        2 => pb.set_message("Recording".green().to_string()),
                        3 => pb.set_message("Recording".cyan().to_string()),
                        4 => pb.set_message("Recording".blue().to_string()),
                        _ => pb.set_message("Recording".magenta().to_string()),
                    }
                }
                
                // Wait for next frame
                sleep(frame_delay).await;
            }
            
            pb.finish_with_message("Recording complete!".green().to_string());
            eprintln!("{}", "Creating GIF...".bright_cyan());
            std::io::stderr().flush().ok();
        } else {
            eprintln!("Recording for {} seconds...", duration_secs);
            for i in 0..total_frames {
                // Take screenshot
                let screenshot_data = driver.screenshot_as_png().await?;
                let frame_path = temp_dir.path().join(format!("frame_{:04}.png", i));
                std::fs::write(&frame_path, screenshot_data)?;
                frames.push(frame_path);
                
                // Wait for next frame
                sleep(frame_delay).await;
            }
            eprintln!("Recording complete. Creating GIF...");
        }
    } else {
        for i in 0..total_frames {
            // Take screenshot
            let screenshot_data = driver.screenshot_as_png().await?;
            let frame_path = temp_dir.path().join(format!("frame_{:04}.png", i));
            std::fs::write(&frame_path, screenshot_data)?;
            frames.push(frame_path);
            
            // Wait for next frame
            sleep(frame_delay).await;
        }
    }
    
    // Create GIF from frames
    create_gif_from_frames(&frames, output_path, is_piped, debug)?;
    
    if !is_piped && !debug {
        eprintln!("{} {}", "✓".green(), format!("GIF saved to {}", output_path.display()).bright_green());
        std::io::stderr().flush().ok();
    } else if !is_piped && debug {
        eprintln!("GIF saved to {}", output_path.display());
    }
    
    Ok(())
}

fn create_gif_from_frames(frame_paths: &[PathBuf], output_path: &PathBuf, is_piped: bool, debug: bool) -> Result<()> {
    // Load all frames
    let mut frames = Vec::new();
    
    if !is_piped && !debug {
        let pb = ProgressBar::new(frame_paths.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("#>-"));
        
        pb.set_message("Processing frames".bright_blue().to_string());
        
        for (i, path) in frame_paths.iter().enumerate() {
            let img = image::open(path)?;
            let frame = img.to_rgba8();
            frames.push(frame);
            
            // Update progress bar with rainbow colors
            match i % 6 {
                0 => pb.set_message("Processing frames".red().to_string()),
                1 => pb.set_message("Processing frames".yellow().to_string()),
                2 => pb.set_message("Processing frames".green().to_string()),
                3 => pb.set_message("Processing frames".cyan().to_string()),
                4 => pb.set_message("Processing frames".blue().to_string()),
                _ => pb.set_message("Processing frames".magenta().to_string()),
            }
            
            pb.inc(1);
        }
        
        pb.finish_with_message("Frames processed!".green().to_string());
    } else if !is_piped && debug {
        eprintln!("Processing {} frames...", frame_paths.len());
        for path in frame_paths {
            let img = image::open(path)?;
            let frame = img.to_rgba8();
            frames.push(frame);
        }
        eprintln!("Frames processed. Creating GIF...");
    } else {
        for path in frame_paths {
            let img = image::open(path)?;
            let frame = img.to_rgba8();
            frames.push(frame);
        }
    }
    
    // Create GIF
    if output_path.to_str() == Some("-") {
        // Write to stdout
        let mut buffer = Vec::new();
        write_gif_to_buffer(&frames, &mut buffer)?;
        io::stdout().write_all(&buffer)?;
    } else {
        // Write to file
        let mut file = std::fs::File::create(output_path)?;
        write_gif_to_buffer(&frames, &mut file)?;
    }
    
    Ok(())
}

fn write_gif_to_buffer<W: Write>(frames: &[image::RgbaImage], buffer: &mut W) -> Result<()> {
    let (width, height) = (frames[0].width(), frames[0].height());
    
    let mut encoder = gif::Encoder::new(buffer, width as u16, height as u16, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;
    
    for frame in frames {
        let mut frame_data = Vec::new();
        for pixel in frame.pixels() {
            frame_data.push(pixel[0]);
            frame_data.push(pixel[1]);
            frame_data.push(pixel[2]);
        }
        
        let mut frame = gif::Frame::from_rgb(width as u16, height as u16, &frame_data);
        frame.delay = 10; // 1/10th of a second
        encoder.write_frame(&frame)?;
    }
    
    Ok(())
}
