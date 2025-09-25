use anyhow::Result;
use std::io::{self, Write};

mod face_detection;
mod face_storage;
mod registration;
mod authentication;
mod camera;

use registration::register_face_from_camera;
use authentication::authenticate_face_from_camera;

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        // Show main menu
        println!("=================================");
        println!("  Face Authentication System");
        println!("=================================");
        println!();
        println!("📷 Camera Permission Notice:");
        println!("This app requires camera access to capture your face.");
        println!("macOS may ask for permission - please click 'Allow'.");
        println!();
        println!("Please select an option:");
        println!("1. Register - Capture and register your face");
        println!("2. Check - Authenticate your face");
        println!("3. Exit");
        println!();
        print!("Enter your choice (1, 2, or 3): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("Error reading input: {}. Please try again.", e);
            continue;
        }
        let choice = input.trim();

        match choice {
            "1" => {
                println!("\n--- Face Registration Mode ---");
                println!("This will capture your face using the camera and register it for authentication.");
                println!();

                match register_face_from_camera() {
                    Ok(_) => {
                        println!("\n✓ Face registration completed successfully!");
                        println!();
                        print!("Press ENTER to return to main menu...");
                        io::stdout().flush().unwrap();
                        let _ = io::stdin().read_line(&mut String::new());
                    },
                    Err(e) => {
                        println!("\n❌ Registration failed: {}", e);
                        println!();
                        println!("Would you like to:");
                        println!("1. Try again");
                        println!("2. Return to main menu");
                        print!("Enter choice (1 or 2): ");
                        io::stdout().flush().unwrap();

                        let mut retry_input = String::new();
                        if io::stdin().read_line(&mut retry_input).is_ok() && retry_input.trim() == "1" {
                            continue; // This will restart the main loop, showing the menu again
                        }
                    }
                }
            },
            "2" => {
                println!("\n--- Face Authentication Mode ---");
                println!("This will capture your face using the camera and check if it matches the registered face.");
                println!();

                match authenticate_face_from_camera() {
                    Ok(result) => {
                        if result.is_match {
                            println!("\n✓ Authentication successful! Confidence: {:.2}%", result.confidence * 100.0);
                            println!("Welcome! Face authentication passed.");
                        } else {
                            println!("\n✗ Authentication failed. Confidence: {:.2}%", result.confidence * 100.0);
                            println!("Access denied. Face not recognized.");
                        }
                        println!();
                        print!("Press ENTER to return to main menu...");
                        io::stdout().flush().unwrap();
                        let _ = io::stdin().read_line(&mut String::new());
                    },
                    Err(e) => {
                        println!("\n❌ Authentication failed: {}", e);
                        println!();
                        println!("Would you like to:");
                        println!("1. Try again");
                        println!("2. Return to main menu");
                        print!("Enter choice (1 or 2): ");
                        io::stdout().flush().unwrap();

                        let mut retry_input = String::new();
                        if io::stdin().read_line(&mut retry_input).is_ok() && retry_input.trim() == "1" {
                            continue; // This will restart the main loop, showing the menu again
                        }
                    }
                }
            },
            "3" => {
                println!("\nThank you for using Face Authentication System!");
                println!("Goodbye! 👋");
                break;
            },
            _ => {
                println!("\n❌ Invalid choice. Please select 1, 2, or 3.");
                println!();
                print!("Press ENTER to continue...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            }
        }

        println!(); // Add some spacing before showing menu again
    }

    Ok(())
}