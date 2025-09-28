use crate::face_detection::FaceDetector;
use crate::face_storage::FaceDatabase;
use crate::camera::CameraCapture;
use anyhow::{Result, anyhow};
use chrono;

#[derive(Debug)]
pub struct AuthenticationResult {
    pub is_match: bool,
    pub confidence: f64,
    pub matched_user_id: Option<String>,
    pub similarity_threshold: f64,
    pub processing_time_ms: u128,
    pub face_detection_confidence: f64,
}

pub struct AdvancedAuthenticator {
    detector: FaceDetector,
    database: FaceDatabase,
    adaptive_threshold: f64,
    false_positive_rate: f64,
    false_negative_rate: f64,
}

impl AdvancedAuthenticator {
    pub fn new() -> Result<Self> {
        let detector = FaceDetector::new()?;
        let database = FaceDatabase::load()?;

        Ok(AdvancedAuthenticator {
            detector,
            database,
            adaptive_threshold: 0.85, // Start with high security threshold
            false_positive_rate: 0.01, // Target 1% false positive rate
            false_negative_rate: 0.05, // Target 5% false negative rate
        })
    }

    /// Adaptive threshold that adjusts based on user enrollment quality and historical performance
    fn calculate_adaptive_threshold(&self, user_id: &str) -> f64 {
        if let Some(profile) = self.database.get_user_profile(user_id) {
            let sample_count = profile.face_samples.len();
            let avg_confidence = profile.face_samples.iter()
                .map(|sample| sample.confidence_during_registration)
                .sum::<f64>() / sample_count as f64;

            // Lower threshold for users with more samples and higher enrollment confidence
            let base_threshold = self.database.accuracy_threshold;
            let sample_bonus = (sample_count as f64 / 10.0).min(0.1); // Up to 10% bonus for more samples
            let confidence_bonus = (avg_confidence - 0.7).max(0.0) * 0.2; // Bonus for high enrollment confidence

            (base_threshold - sample_bonus - confidence_bonus).max(0.7) // Never go below 70%
        } else {
            self.database.accuracy_threshold // Default threshold for unknown users
        }
    }

    pub fn authenticate_face_from_camera(&mut self) -> Result<AuthenticationResult> {
        let start_time = std::time::Instant::now();

        println!("🔍 Initializing advanced face authentication...");
        let mut camera = CameraCapture::new()?;

        // Capture image with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let temp_image_path = format!("captured_images/authentication_{}.jpg", timestamp);
        camera.capture_and_save(&temp_image_path)?;

        println!("📸 Image captured, analyzing with professional face detector...");

        // Detect faces using advanced detector
        let faces = self.detector.detect_faces(&temp_image_path)?;

        if faces.is_empty() {
            println!("🔍 No faces detected in captured image: {}", temp_image_path);
            return Err(anyhow!("No faces detected. Please ensure your face is clearly visible and well-lit."));
        }

        if faces.len() > 1 {
            println!("⚠️  Multiple faces detected ({}), using the most confident detection", faces.len());
        }

        // Use the face with highest detection confidence
        let best_face = faces.iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        println!("✅ Face detected with {:.1}% confidence", best_face.confidence * 100.0);
        println!("🧠 Extracted {} advanced features", best_face.features.len());

        // Check if database has any enrolled users
        if self.database.get_all_users().is_empty() {
            println!("📁 Authentication image saved: {}", temp_image_path);
            return Err(anyhow!("No users enrolled in the system. Please register a face first."));
        }

        println!("🔍 Comparing against {} enrolled users using parallel processing...",
                 self.database.get_all_users().len());

        // Find best match using optimized database search
        let best_match = self.database.find_best_match(&best_face.features);

        let processing_time = start_time.elapsed().as_millis();
        println!("⚡ Processing completed in {}ms", processing_time);

        // Keep the captured image for reference/debugging
        println!("📁 Authentication image saved: {}", temp_image_path);

        if let Some((user_id, confidence)) = best_match {
            let adaptive_threshold = self.calculate_adaptive_threshold(&user_id);

            println!("🎯 Best match: User '{}' with {:.1}% confidence",
                     user_id, confidence * 100.0);
            println!("🎚️  Adaptive threshold for this user: {:.1}%",
                     adaptive_threshold * 100.0);

            let is_match = confidence >= adaptive_threshold;

            if is_match {
                // Update authentication statistics
                self.database.update_authentication_stats(&user_id)?;
                println!("✅ Authentication successful! Welcome back, {}", user_id);
            } else {
                println!("❌ Authentication failed. Confidence too low for secure access.");
            }

            Ok(AuthenticationResult {
                is_match,
                confidence,
                matched_user_id: Some(user_id),
                similarity_threshold: adaptive_threshold,
                processing_time_ms: processing_time,
                face_detection_confidence: best_face.confidence,
            })
        } else {
            println!("❌ No matching face found in database");
            Ok(AuthenticationResult {
                is_match: false,
                confidence: 0.0,
                matched_user_id: None,
                similarity_threshold: self.database.accuracy_threshold,
                processing_time_ms: processing_time,
                face_detection_confidence: best_face.confidence,
            })
        }
    }

    pub fn get_database_stats(&self) -> crate::face_storage::DatabaseStats {
        self.database.get_database_stats()
    }

    pub fn optimize_database(&mut self) -> Result<usize> {
        self.database.optimize_database()
    }
}

// Legacy function for compatibility
pub fn authenticate_face(image_path: &str) -> Result<AuthenticationResult> {
    let start_time = std::time::Instant::now();

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
    let database = FaceDatabase::load()?;

    if database.get_all_faces().is_empty() {
        return Err(anyhow!("No registered faces found. Please register a face first."));
    }

    println!("🔍 Comparing with {} registered face(s) using advanced similarity matching...",
             database.get_all_faces().len());

    // Find best match
    let best_match = database.find_best_match(&best_face.features);
    let processing_time = start_time.elapsed().as_millis();

    if let Some((user_id, confidence)) = best_match {
        let threshold = database.accuracy_threshold;
        let is_match = confidence >= threshold;

        println!("🎯 Best match: User '{}' with {:.1}% confidence",
                 user_id, confidence * 100.0);
        println!("🎚️  Threshold: {:.1}%", threshold * 100.0);

        Ok(AuthenticationResult {
            is_match,
            confidence,
            matched_user_id: Some(user_id),
            similarity_threshold: threshold,
            processing_time_ms: processing_time,
            face_detection_confidence: best_face.confidence,
        })
    } else {
        Ok(AuthenticationResult {
            is_match: false,
            confidence: 0.0,
            matched_user_id: None,
            similarity_threshold: database.accuracy_threshold,
            processing_time_ms: processing_time,
            face_detection_confidence: best_face.confidence,
        })
    }
}

// Updated function for camera-based authentication
pub fn authenticate_face_from_camera() -> Result<AuthenticationResult> {
    let mut authenticator = AdvancedAuthenticator::new()?;
    authenticator.authenticate_face_from_camera()
}