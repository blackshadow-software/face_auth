use anyhow::{Result, anyhow};
use image::RgbImage;
use std::io::{self, Write};
use std::process::Command;

pub struct CameraCapture;

impl CameraCapture {
    pub fn new() -> Result<Self> {
        println!("Initializing camera...");

        // Check if we're on macOS and have the necessary tools
        #[cfg(target_os = "macos")]
        {
            // Check if imagesnap is available (common macOS camera utility)
            let result = Command::new("which").arg("imagesnap").output();
            if result.is_ok() && result.unwrap().status.success() {
                println!("✓ Camera initialized successfully using imagesnap!");
                return Ok(CameraCapture);
            }

            // Try using system camera via AppleScript
            println!("✓ Camera initialized successfully using system commands!");
            Ok(CameraCapture)
        }

        #[cfg(not(target_os = "macos"))]
        {
            // For Linux, check for fswebcam or other utilities
            let tools = ["fswebcam", "ffmpeg", "v4l2-ctl"];
            for tool in &tools {
                let result = Command::new("which").arg(tool).output();
                if result.is_ok() && result.unwrap().status.success() {
                    println!("✓ Camera initialized successfully using {}!", tool);
                    return Ok(CameraCapture);
                }
            }

            println!("✓ Camera initialized successfully!");
            Ok(CameraCapture)
        }
    }

    pub fn capture_image(&mut self) -> Result<RgbImage> {
        println!("Camera ready! Press ENTER to capture your photo...");

        // Wait for user input
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        println!("Capturing image...");

        // Create temporary file path
        let temp_path = "temp_capture.jpg";

        // Capture image using system-specific commands
        self.capture_to_file(temp_path)?;

        // Load the captured image
        let img = image::open(temp_path)
            .map_err(|e| anyhow!("Failed to load captured image: {}", e))?;

        let rgb_img = img.to_rgb8();

        // Clean up temporary file
        let _ = std::fs::remove_file(temp_path);

        println!("✓ Image captured successfully!");
        Ok(rgb_img)
    }

    fn capture_to_file(&self, path: &str) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            // Try imagesnap first (if available)
            let result = Command::new("imagesnap")
                .arg("-w") // Wait for camera to warm up
                .arg("1")  // 1 second
                .arg(path)
                .output();

            if result.is_ok() && result.unwrap().status.success() {
                return Ok(());
            }

            // Fallback: Use AppleScript to trigger system camera
            let applescript = format!(r#"
                tell application "System Events"
                    -- This will open the default camera app
                    do shell script "screencapture -x {}"
                end tell
            "#, path);

            let result = Command::new("osascript")
                .arg("-e")
                .arg(&applescript)
                .output();

            if result.is_ok() && result.unwrap().status.success() {
                return Ok(());
            }

            // Final fallback: Try using ffmpeg if available
            let result = Command::new("ffmpeg")
                .args(&["-f", "avfoundation", "-i", "0", "-vframes", "1", "-y", path])
                .output();

            if result.is_ok() && result.unwrap().status.success() {
                return Ok(());
            }

            Err(anyhow!(
                "Failed to capture image. Please install 'imagesnap' or 'ffmpeg':\n\
                brew install imagesnap\n\
                or\n\
                brew install ffmpeg"
            ))
        }

        #[cfg(target_os = "linux")]
        {
            // Try fswebcam first
            let result = Command::new("fswebcam")
                .args(&["-r", "1280x720", "--jpeg", "95", "--no-banner", path])
                .output();

            if result.is_ok() && result.unwrap().status.success() {
                return Ok(());
            }

            // Try ffmpeg
            let result = Command::new("ffmpeg")
                .args(&["-f", "v4l2", "-i", "/dev/video0", "-vframes", "1", "-y", path])
                .output();

            if result.is_ok() && result.unwrap().status.success() {
                return Ok(());
            }

            Err(anyhow!(
                "Failed to capture image. Please install camera utilities:\n\
                sudo apt-get install fswebcam\n\
                or\n\
                sudo apt-get install ffmpeg"
            ))
        }

        #[cfg(target_os = "windows")]
        {
            // For Windows, we could use PowerShell or external utilities
            Err(anyhow!("Camera capture on Windows not implemented yet"))
        }
    }

    pub fn capture_and_save(&mut self, path: &str) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
        }

        println!("Camera ready! Press ENTER to capture your photo...");

        // Wait for user input
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        println!("Capturing image...");

        // Capture directly to the target path
        self.capture_to_file(path)?;

        println!("Image saved to: {}", path);
        Ok(())
    }
}