use anyhow::Result;
use std::io::{self, Write};

mod face_detection;
mod face_storage;
mod registration;
mod authentication;
mod camera;
mod python_integration;

use registration::register_face_from_camera;
use authentication::authenticate_face_from_camera;
use python_integration::PythonFaceAuth;

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
        println!("1. Register - Capture and register your face (Rust - Fast)");
        println!("2. Check - Authenticate your face (Rust - Fast)");
        println!("3. Python Register - High accuracy registration (99%+)");
        println!("4. Python Auth - High accuracy authentication (99%+)");
        println!("5. Exit");
        println!();
        print!("Enter your choice (1-5): ");
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
                            println!("\n✅ Authentication successful!");
                            println!("🎯 Confidence: {:.1}%", result.confidence * 100.0);
                            println!("🎚️  Threshold: {:.1}%", result.similarity_threshold * 100.0);
                            println!("👤 User: {}", result.matched_user_id.as_ref().unwrap_or(&"Unknown".to_string()));
                            println!("⚡ Processing time: {}ms", result.processing_time_ms);
                            println!("🔍 Face detection confidence: {:.1}%", result.face_detection_confidence * 100.0);
                            println!("🎉 Welcome! Access granted.");
                        } else {
                            println!("\n❌ Authentication failed!");
                            println!("🎯 Confidence: {:.1}%", result.confidence * 100.0);
                            println!("🎚️  Required threshold: {:.1}%", result.similarity_threshold * 100.0);
                            if let Some(user_id) = &result.matched_user_id {
                                println!("👤 Closest match: {}", user_id);
                            }
                            println!("⚡ Processing time: {}ms", result.processing_time_ms);
                            println!("🔍 Face detection confidence: {:.1}%", result.face_detection_confidence * 100.0);
                            println!("🔒 Access denied. Please try again or register your face.");
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
                println!("\n--- 🐍 Python High-Accuracy Registration ---");
                println!("This uses Python's face_recognition library for 99%+ accuracy");
                println!();

                match PythonFaceAuth::new() {
                    Ok(python_auth) => {
                        match python_auth.check_python_environment() {
                            Ok(_) => {
                                match python_auth.register_user("user", 3) {
                                    Ok(true) => {
                                        println!("\n🎉 Python registration completed successfully!");
                                        println!("✅ High-accuracy face model trained!");
                                    },
                                    Ok(false) => {
                                        println!("\n❌ Python registration failed");
                                    },
                                    Err(e) => {
                                        println!("\n❌ Registration error: {}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                println!("\n❌ Python environment error: {}", e);
                                println!("💡 Please run: ./setup_python_env.sh");
                            }
                        }
                    },
                    Err(e) => {
                        println!("\n❌ Python initialization error: {}", e);
                        println!("💡 Please run: ./setup_python_env.sh");
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "4" => {
                println!("\n--- 🐍 Python High-Accuracy Authentication ---");
                println!("This uses Python's face_recognition library for 99%+ accuracy");
                println!();

                match PythonFaceAuth::new() {
                    Ok(python_auth) => {
                        match python_auth.check_python_environment() {
                            Ok(_) => {
                                match python_auth.authenticate_user(0.6) {
                                    Ok(result) => {
                                        if result.success && result.is_match.unwrap_or(false) {
                                            println!("\n✅ Python Authentication Successful!");
                                            println!("🎯 Confidence: {:.1}%", result.confidence.unwrap_or(0.0) * 100.0);
                                            println!("📏 Distance: {:.3}", result.distance.unwrap_or(0.0));
                                            println!("👤 User: {}", result.matched_user.as_ref().unwrap_or(&"Unknown".to_string()));
                                            println!("⚡ Processing time: {}ms", result.processing_time_ms.unwrap_or(0));
                                            println!("🎉 Access granted with high accuracy!");
                                        } else {
                                            println!("\n❌ Python Authentication Failed!");
                                            println!("🎯 Confidence: {:.1}%", result.confidence.unwrap_or(0.0) * 100.0);
                                            println!("📏 Distance: {:.3}", result.distance.unwrap_or(0.0));
                                            println!("🎚️  Threshold: {:.3}", result.threshold.unwrap_or(0.0));
                                            println!("⚡ Processing time: {}ms", result.processing_time_ms.unwrap_or(0));
                                            println!("🔒 Access denied.");
                                        }
                                    },
                                    Err(e) => {
                                        println!("\n❌ Authentication error: {}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                println!("\n❌ Python environment error: {}", e);
                                println!("💡 Please run: ./setup_python_env.sh");
                            }
                        }
                    },
                    Err(e) => {
                        println!("\n❌ Python initialization error: {}", e);
                        println!("💡 Please run: ./setup_python_env.sh");
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "5" => {
                println!("\nThank you for using Face Authentication System!");
                println!("Goodbye! 👋");
                break;
            },
            _ => {
                println!("\n❌ Invalid choice. Please select 1-5.");
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