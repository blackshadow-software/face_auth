use anyhow::Result;
use face_auth::FaceAuth;
use std::io::{self, Write};

/// Simple interactive example of the face authentication library
#[tokio::main]
async fn main() -> Result<()> {
    println!("===========================================");
    println!("  Face Auth Library - Example Application");
    println!("===========================================\n");

    // Get custom paths from environment variables or use defaults
    let generated_dir = &"generated";
    let source_dir = &"source";
    println!("📁 Configuration:");
    println!("   Generated directory: {}", generated_dir);
    println!("   Source directory: {}", source_dir);
    println!();

    // Initialize the face authentication system
    let face_auth = FaceAuth::new()?;

    // Check if the system is working
    println!("Checking system...");
    face_auth.check_system().await?;
    println!("✓ System ready!\n");

    loop {
        println!("Select an option:");
        println!("1. Register a new user");
        println!("2. Authenticate user");
        println!("3. Export user data");
        println!("4. Import user data");
        println!("5. List all users");
        println!("6. Exit");
        print!("\nYour choice: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                print!("Enter username to register: ");
                io::stdout().flush()?;
                let mut username = String::new();
                io::stdin().read_line(&mut username)?;
                let username = username.trim();

                println!("\nRegistering user '{}'...", username);
                match face_auth.register_user(username, 3, &generated_dir).await {
                    Ok(true) => {
                        println!("✓ Registration successful!");
                        println!("File saved to: {}/{}.json", generated_dir, username);
                        println!("\nTo enable authentication, run:");
                        println!("  cp {}/{}.json {}/", generated_dir, username, source_dir);
                    }
                    Ok(false) => println!("✗ Registration failed"),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            "2" => {
                println!("\nAuthenticating...");
                match face_auth.authenticate_user(0.6, &source_dir).await {
                    Ok(result) => {
                        if result.is_authenticated {
                            println!("\n✓ Authentication successful!");
                            if let Some(user) = result.user_id {
                                println!("  User: {}", user);
                            }
                            if let Some(confidence) = result.confidence {
                                println!("  Confidence: {:.1}%", confidence * 100.0);
                            }
                            if let Some(distance) = result.distance {
                                println!("  Distance: {:.3}", distance);
                            }
                        } else {
                            println!("\n✗ Authentication failed");
                            if let Some(distance) = result.distance {
                                println!("  Best distance: {:.3}", distance);
                            }
                            if let Some(threshold) = result.threshold {
                                println!("  Threshold: {:.3}", threshold);
                            }
                        }
                    }
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            "3" => {
                print!("Enter username to export: ");
                io::stdout().flush()?;
                let mut username = String::new();
                io::stdin().read_line(&mut username)?;
                let username = username.trim();

                match face_auth.export_user(username, "").await {
                    Ok(true) => println!("✓ User '{}' exported successfully", username),
                    Ok(false) => println!("✗ Export failed"),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            "4" => {
                print!("Enter filename to import: ");
                io::stdout().flush()?;
                let mut filename = String::new();
                io::stdin().read_line(&mut filename)?;
                let filename = filename.trim();

                match face_auth.import_user(filename).await {
                    Ok(true) => println!("✓ User imported successfully"),
                    Ok(false) => println!("✗ Import failed"),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            "5" => {
                println!("\nListing users...");
                if let Err(e) = face_auth.list_users().await {
                    println!("✗ Error: {}", e);
                }
            }
            "6" => {
                println!("\nGoodbye!");
                break;
            }
            _ => println!("Invalid choice. Please select 1-6."),
        }

        println!("\n---\n");
    }

    Ok(())
}
