use anyhow::{Result, anyhow};
use image::{ImageBuffer, Rgb};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::io::{self, Write};

pub struct CameraCapture {
    camera: Camera,
}

impl CameraCapture {
    pub fn new() -> Result<Self> {
        println!("Initializing camera...");

        // Get the first available camera
        let index = CameraIndex::Index(0);

        // Request format
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

        // Create camera with better error handling
        let camera = Camera::new(index, requested)
            .map_err(|e| {
                match e.to_string().as_str() {
                    s if s.contains("permission") || s.contains("access") => {
                        anyhow!(
                            "Camera permission denied!\n\n\
                            On macOS: Go to System Preferences → Security & Privacy → Privacy → Camera\n\
                            and make sure this application has permission to access the camera.\n\n\
                            Then restart the application.\n\n\
                            Original error: {}", e
                        )
                    },
                    s if s.contains("busy") || s.contains("in use") => {
                        anyhow!(
                            "Camera is currently being used by another application.\n\
                            Please close other apps that might be using the camera (Photo Booth, Zoom, etc.) and try again.\n\n\
                            Original error: {}", e
                        )
                    },
                    _ => anyhow!(
                        "Failed to initialize camera: {}\n\n\
                        Possible solutions:\n\
                        1. Check camera permissions in System Preferences → Security & Privacy → Camera\n\
                        2. Make sure no other apps are using the camera\n\
                        3. Try reconnecting your camera if using an external one\n\
                        4. Restart the application", e
                    )
                }
            })?;

        println!("✓ Camera initialized successfully!");
        Ok(CameraCapture { camera })
    }

    pub fn capture_image(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        println!("Opening camera stream...");

        // Open the camera with better error handling
        self.camera.open_stream()
            .map_err(|e| {
                match e.to_string().as_str() {
                    s if s.contains("permission") || s.contains("access") => {
                        anyhow!(
                            "Camera access denied!\n\n\
                            macOS may have prompted for camera permission. If you denied it:\n\
                            1. Go to System Preferences → Security & Privacy → Privacy → Camera\n\
                            2. Find this application and check the box to allow camera access\n\
                            3. Restart the application\n\n\
                            Original error: {}", e
                        )
                    },
                    _ => anyhow!("Failed to open camera stream: {}\n\nTry closing other camera applications and restart this program.", e)
                }
            })?;

        println!("Camera ready! Press ENTER to capture your photo...");

        // Wait for user input
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Capture frame
        println!("Capturing image...");
        let frame = self.camera.frame()
            .map_err(|e| anyhow!("Failed to capture frame: {}", e))?;

        // Close camera
        let _ = self.camera.stop_stream();

        // Convert frame to image
        let decoded = frame.decode_image::<RgbFormat>()
            .map_err(|e| anyhow!("Failed to decode image: {}", e))?;

        let (width, height) = (decoded.width(), decoded.height());
        let raw_data = decoded.into_raw();

        let img_buffer = ImageBuffer::from_raw(width, height, raw_data)
            .ok_or_else(|| anyhow!("Failed to create image buffer"))?;

        println!("✓ Image captured successfully!");

        Ok(img_buffer)
    }

    pub fn capture_and_save(&mut self, path: &str) -> Result<()> {
        let img_buffer = self.capture_image()?;

        // Convert RGB to DynamicImage and save
        let dynamic_img = image::DynamicImage::ImageRgb8(img_buffer);
        dynamic_img.save(path)
            .map_err(|e| anyhow!("Failed to save image: {}", e))?;

        println!("Image saved to: {}", path);
        Ok(())
    }
}