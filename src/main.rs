use anyhow::Result;
use std::io::{self, Write};
use face_auth::StandalonePythonFaceAuth;

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
        println!("1. Register - High accuracy (99%+)");
        println!("2. Authenticate - High accuracy (99%+)");
        println!("3. Export User - Export user credentials to file");
        println!("4. Import User - Import user credentials from file");
        println!("5. List Users - Show all registered users");
        println!("6. Exit");
        println!();
        print!("Enter your choice (1-6): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("Error reading input: {}. Please try again.", e);
            continue;
        }
        let choice = input.trim();

        match choice {
            "1" => {
                println!("\n--- 🐍 High-Accuracy Face Registration ---");
                println!("This uses Python's face_recognition library for 99%+ accuracy");
                println!("🎯 Industry-standard face detection and recognition!");
                println!();

                match StandalonePythonFaceAuth::new() {
                    Ok(standalone_auth) => {
                        match standalone_auth.check_executable() {
                            Ok(_) => {
                                print!("Enter username for registration: ");
                                io::stdout().flush().unwrap();
                                let mut username = String::new();
                                if io::stdin().read_line(&mut username).is_ok() {
                                    let username = username.trim();

                                    match standalone_auth.register_user(username, 3) {
                                        Ok(true) => {
                                            println!("\n🎉 Standalone Python registration successful!");
                                            println!("✅ High-accuracy face model trained with standalone executable!");
                                            println!("📦 No Python installation was required!");
                                        },
                                        Ok(false) => {
                                            println!("\n❌ Standalone Python registration failed");
                                            println!("💡 Make sure you're positioned in front of the camera");
                                        },
                                        Err(e) => {
                                            println!("\n❌ Registration error: {}", e);
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                println!("\n❌ Standalone executable error: {}", e);
                                println!("💡 Make sure you've built the standalone executable first");
                                println!("💡 Run: pyinstaller --onefile --console --add-data=\"face_auth_env/lib/python3.9/site-packages/face_recognition_models/models/*:face_recognition_models/models/\" python_face_auth_simple.py");
                            }
                        }
                    },
                    Err(e) => {
                        println!("\n❌ Failed to initialize standalone Python: {}", e);
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "2" => {
                println!("\n--- 🐍 High-Accuracy Face Authentication ---");
                println!("This uses Python's face_recognition library for 99%+ accuracy");
                println!("🎯 Industry-standard face detection and recognition!");
                println!();

                match StandalonePythonFaceAuth::new() {
                    Ok(standalone_auth) => {
                        match standalone_auth.check_executable() {
                            Ok(_) => {
                                match standalone_auth.authenticate_user(0.4) {
                                    Ok(result) => {
                                        if result.is_match.unwrap_or(false) {
                                            println!("\n✅ Standalone Python Authentication Successful!");
                                            println!("🎯 Confidence: {:.1}%", result.confidence.unwrap_or(0.0) * 100.0);
                                            println!("📏 Distance: {:.3}", result.distance.unwrap_or(0.0));
                                            println!("👤 User: {}", result.matched_user.as_ref().unwrap_or(&"Unknown".to_string()));
                                            println!("⚡ Processing time: {}ms", result.processing_time_ms.unwrap_or(0));
                                            println!("📦 No Python installation was required!");
                                            println!("🎉 Access granted with standalone executable!");
                                        } else {
                                            println!("\n❌ Standalone Python Authentication Failed!");
                                            println!("🎯 Confidence: {:.1}%", result.confidence.unwrap_or(0.0) * 100.0);
                                            println!("📏 Distance: {:.3}", result.distance.unwrap_or(0.0));
                                            println!("🎚️  Threshold: {:.3}", result.threshold.unwrap_or(0.0));
                                            println!("⚡ Processing time: {}ms", result.processing_time_ms.unwrap_or(0));
                                            println!("🔒 Access denied. Please try again or register first.");
                                        }
                                    },
                                    Err(e) => {
                                        println!("\n❌ Authentication error: {}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                println!("\n❌ Standalone executable error: {}", e);
                                println!("💡 Make sure you've built the standalone executable first");
                            }
                        }
                    },
                    Err(e) => {
                        println!("\n❌ Failed to initialize standalone Python: {}", e);
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "3" => {
                println!("\n--- 📤 Export User Credentials ---");
                println!("Export a user's face data to share with another device");
                println!();

                print!("Enter username to export: ");
                io::stdout().flush().unwrap();
                let mut username = String::new();
                if io::stdin().read_line(&mut username).is_ok() {
                    let username = username.trim();

                    match StandalonePythonFaceAuth::new() {
                        Ok(standalone_auth) => {
                            match standalone_auth.export_user(username, "") {
                                Ok(true) => {
                                    println!("\n✅ User '{}' exported successfully!", username);
                                    println!("📁 File saved in 'exported_credentials/' directory");
                                    println!("🔄 You can copy this file to another device");
                                    println!("🔄 Use 'Import User' on the target device to add this user");
                                },
                                Ok(false) => {
                                    println!("\n❌ Export failed. User '{}' may not exist.", username);
                                },
                                Err(e) => {
                                    println!("\n❌ Export error: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            println!("\n❌ System error: {}", e);
                        }
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "4" => {
                println!("\n--- 📥 Import User Credentials ---");
                println!("Import a user's face data from another device");
                println!();

                print!("Enter filename to import (from exported_credentials/ or full path): ");
                io::stdout().flush().unwrap();
                let mut filename = String::new();
                if io::stdin().read_line(&mut filename).is_ok() {
                    let filename = filename.trim();

                    match StandalonePythonFaceAuth::new() {
                        Ok(standalone_auth) => {
                            match standalone_auth.import_user(filename) {
                                Ok(true) => {
                                    println!("\n✅ User imported successfully from '{}'", filename);
                                    println!("👤 User is now available for authentication");
                                },
                                Ok(false) => {
                                    println!("\n❌ Import failed. Check if file exists and is valid.");
                                },
                                Err(e) => {
                                    println!("\n❌ Import error: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            println!("\n❌ System error: {}", e);
                        }
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "5" => {
                println!("\n--- 👥 Registered Users ---");
                println!();

                match StandalonePythonFaceAuth::new() {
                    Ok(standalone_auth) => {
                        match standalone_auth.list_users() {
                            Ok(_) => {
                                // Success message already printed by Python script
                            },
                            Err(e) => {
                                println!("❌ Error listing users: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        println!("❌ System error: {}", e);
                    }
                }

                println!();
                print!("Press ENTER to return to main menu...");
                io::stdout().flush().unwrap();
                let _ = io::stdin().read_line(&mut String::new());
            },
            "6" => {
                println!("\nThank you for using Face Authentication System!");
                println!("Goodbye! 👋");
                break;
            },
            _ => {
                println!("\n❌ Invalid choice. Please select 1-6.");
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