use crate::face_detection::FaceDetector;
use crate::face_storage::FaceDatabase;
use crate::camera::CameraCapture;
use anyhow::{Result, anyhow};
use chrono;
use std::io::{self, Write};

pub struct AdvancedRegistration {
    detector: FaceDetector,
    database: FaceDatabase,
}

impl AdvancedRegistration {
    pub fn new() -> Result<Self> {
        let detector = FaceDetector::new()?;
        let database = FaceDatabase::load()?;

        Ok(AdvancedRegistration {
            detector,
            database,
        })
    }

    /// Register multiple face samples for a user to improve accuracy
    pub fn register_user_interactive(&mut self, user_id: String) -> Result<()> {
        println!("=== 🎯 Advanced Face Registration for User: '{}' ===", user_id);

        // Check current enrollment status
        let (current_samples, required_samples) = self.database.get_enrollment_progress(&user_id);

        if current_samples > 0 {
            println!("📊 Current enrollment: {}/{} samples", current_samples, required_samples);
            println!("🔄 Adding additional samples to improve accuracy...");
        } else {
            println!("🆕 New user registration");
            println!("📋 Will collect {} samples for optimal accuracy", required_samples);
        }

        let mut successful_samples = 0;
        let target_new_samples = if current_samples >= required_samples {
            2 // Add 2 more samples for existing users
        } else {
            required_samples - current_samples // Complete enrollment
        };

        println!("\n🎯 Target: {} new samples", target_new_samples);

        for sample_num in 1..=target_new_samples {
            println!("\n--- 📸 Sample {}/{} ---", sample_num, target_new_samples);
            println!("💡 Tips for best results:");
            println!("   • Look directly at the camera");
            println!("   • Ensure good lighting");
            println!("   • Keep a neutral expression");
            println!("   • Stay still during capture");

            print!("\nPress ENTER when ready to capture sample {}...", sample_num);
            io::stdout().flush().unwrap();
            let _ = io::stdin().read_line(&mut String::new());

            match self.capture_and_register_sample(&user_id) {
                Ok(confidence) => {
                    successful_samples += 1;
                    println!("✅ Sample {} captured successfully! Quality: {:.1}%",
                             sample_num, confidence * 100.0);

                    if confidence < 0.8 {
                        println!("⚠️  Note: Sample quality could be better. Consider retaking if issues persist.");
                    }
                },
                Err(e) => {
                    println!("❌ Failed to capture sample {}: {}", sample_num, e);
                    println!("🔄 Let's try again...");

                    print!("Press ENTER to retry sample {}...", sample_num);
                    io::stdout().flush().unwrap();
                    let _ = io::stdin().read_line(&mut String::new());

                    // Retry once
                    match self.capture_and_register_sample(&user_id) {
                        Ok(confidence) => {
                            successful_samples += 1;
                            println!("✅ Retry successful! Quality: {:.1}%", confidence * 100.0);
                        },
                        Err(retry_error) => {
                            println!("❌ Retry failed: {}", retry_error);
                            println!("⏭️  Skipping this sample...");
                        }
                    }
                }
            }

            // Show current progress
            let (updated_samples, _) = self.database.get_enrollment_progress(&user_id);
            println!("📊 Progress: {}/{} total samples collected", updated_samples, self.database.min_samples_per_user);
        }

        // Final status
        let (final_samples, required) = self.database.get_enrollment_progress(&user_id);
        let is_enrolled = self.database.is_user_enrolled(&user_id);

        println!("\n=== 📋 Registration Complete ===");
        println!("✅ Successfully captured {} new samples", successful_samples);
        println!("📊 Total samples for user '{}': {}", user_id, final_samples);

        if is_enrolled {
            println!("🎉 User '{}' is fully enrolled and ready for authentication!", user_id);
            println!("🔐 Authentication threshold: {:.1}%", self.database.accuracy_threshold * 100.0);
        } else {
            println!("⚠️  User '{}' needs {} more samples for full enrollment",
                     user_id, required - final_samples);
            println!("🔄 Run registration again to complete enrollment");
        }

        // Update database reference
        self.database = FaceDatabase::load()?;

        Ok(())
    }

    fn capture_and_register_sample(&mut self, user_id: &str) -> Result<f64> {
        let mut camera = CameraCapture::new()?;

        // Capture image with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f");
        let sample_count = self.database.get_enrollment_progress(user_id).0 + 1;
        let temp_image_path = format!("captured_images/registration_{}_{}_sample{}.jpg",
                                      user_id, timestamp, sample_count);

        camera.capture_and_save(&temp_image_path)?;

        // Detect faces
        let faces = self.detector.detect_faces(&temp_image_path)?;

        if faces.is_empty() {
            println!("📁 Image saved for debugging: {}", temp_image_path);
            return Err(anyhow!("No faces detected. Please ensure your face is clearly visible and well-lit."));
        }

        if faces.len() > 1 {
            println!("⚠️  Multiple faces detected. Using the most confident detection.");
        }

        // Use the best face detection
        let best_face = faces.iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        // Quality check
        if best_face.confidence < 0.7 {
            println!("📁 Low-quality image saved: {}", temp_image_path);
            return Err(anyhow!("Face detection confidence too low ({:.1}%). Please improve lighting and try again.",
                              best_face.confidence * 100.0));
        }

        // Add to database
        self.database.add_face_sample(
            user_id.to_string(),
            best_face.features.clone(),
            best_face.confidence
        )?;

        println!("📁 Sample saved: {}", temp_image_path);
        println!("🧠 Extracted {} advanced features", best_face.features.len());

        // Reload database to get updated state
        self.database = FaceDatabase::load()?;

        Ok(best_face.confidence)
    }

    pub fn get_registration_stats(&self) -> RegistrationStats {
        let stats = self.database.get_database_stats();
        RegistrationStats {
            total_users: stats.total_users,
            enrolled_users: stats.enrolled_users,
            pending_users: stats.total_users - stats.enrolled_users,
            total_samples: stats.total_samples,
            avg_samples_per_user: if stats.total_users > 0 {
                stats.total_samples as f64 / stats.total_users as f64
            } else {
                0.0
            },
            min_samples_required: stats.min_samples_per_user,
        }
    }
}

#[derive(Debug)]
pub struct RegistrationStats {
    pub total_users: usize,
    pub enrolled_users: usize,
    pub pending_users: usize,
    pub total_samples: usize,
    pub avg_samples_per_user: f64,
    pub min_samples_required: usize,
}

// Legacy functions for compatibility
pub fn register_face(image_path: &str) -> Result<()> {
    println!("🔍 Initializing professional face detector...");
    let detector = FaceDetector::new()?;

    println!("📸 Analyzing image: {}", image_path);
    let faces = detector.detect_faces(image_path)?;

    if faces.is_empty() {
        return Err(anyhow!("No faces detected in the image. Please ensure the image contains a clear face."));
    }

    if faces.len() > 1 {
        println!("⚠️  Multiple faces detected. Using the most confident detection.");
    }

    let best_face = faces.iter()
        .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    println!("✅ Face detected with {:.1}% confidence", best_face.confidence * 100.0);

    // Load database
    let mut database = FaceDatabase::load()?;

    // Use "user" as default user ID for legacy compatibility
    let user_id = "user".to_string();

    // Add sample to database
    database.add_face_sample(user_id.clone(), best_face.features.clone(), best_face.confidence)?;

    let (samples, required) = database.get_enrollment_progress(&user_id);
    println!("📊 Registration progress: {}/{} samples", samples, required);

    if database.is_user_enrolled(&user_id) {
        println!("✅ User '{}' is fully enrolled!", user_id);
    } else {
        println!("⚠️  Need {} more samples for complete enrollment", required - samples);
    }

    Ok(())
}

pub fn register_face_from_camera() -> Result<()> {
    let mut registration = AdvancedRegistration::new()?;

    // Use "user" as default user ID for legacy compatibility
    let user_id = "user".to_string();

    registration.register_user_interactive(user_id)
}