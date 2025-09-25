use crate::face_detection::FaceDetector;
use crate::face_storage::FaceDatabase;
use crate::camera::CameraCapture;
use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct AuthenticationResult {
    pub is_match: bool,
    pub confidence: f64,
    pub matched_user_id: Option<String>,
}

pub fn authenticate_face(image_path: &str) -> Result<AuthenticationResult> {
    println!("Initializing face detector...");
    let detector = FaceDetector::new()?;

    println!("Detecting faces in image: {}", image_path);
    let faces = detector.detect_faces(image_path)?;

    if faces.is_empty() {
        return Err(anyhow!("No faces detected in the image. Please ensure the image contains a clear face."));
    }

    if faces.len() > 1 {
        println!("Warning: Multiple faces detected. Using the first detected face.");
    }

    let face = &faces[0];
    println!("Face detected! Extracting features...");

    // Load database
    let database = FaceDatabase::load()?;

    if database.get_all_faces().is_empty() {
        return Err(anyhow!("No registered faces found. Please register a face first using 'face_auth register'."));
    }

    println!("Comparing with {} registered face(s)...", database.get_all_faces().len());

    // Find best match
    let mut best_match: Option<(String, f64)> = None;

    for stored_face in database.get_all_faces() {
        let similarity = FaceDetector::compute_similarity(&face.features, &stored_face.features);

        match &best_match {
            None => best_match = Some((stored_face.user_id.clone(), similarity)),
            Some((_, best_similarity)) => {
                if similarity > *best_similarity {
                    best_match = Some((stored_face.user_id.clone(), similarity));
                }
            }
        }
    }

    if let Some((user_id, confidence)) = best_match {
        // Define threshold for successful authentication (tuned based on testing)
        // Since we're getting consistently high confidence scores (80%+), we can set a reasonable threshold
        let threshold = 0.70; // Lowered slightly to account for lighting/angle variations

        let is_match = confidence >= threshold;

        println!("Best match: User '{}' with confidence: {:.2}%", user_id, confidence * 100.0);

        Ok(AuthenticationResult {
            is_match,
            confidence,
            matched_user_id: Some(user_id),
        })
    } else {
        Ok(AuthenticationResult {
            is_match: false,
            confidence: 0.0,
            matched_user_id: None,
        })
    }
}

pub fn authenticate_face_from_camera() -> Result<AuthenticationResult> {
    println!("Initializing camera and face detector...");
    let mut camera = CameraCapture::new()?;
    let detector = FaceDetector::new()?;

    // Capture image from camera
    let temp_image_path = "temp_authentication.jpg";
    camera.capture_and_save(temp_image_path)?;

    // Detect faces in captured image
    println!("Detecting faces in captured image...");
    let faces = detector.detect_faces(temp_image_path)?;

    if faces.is_empty() {
        // Clean up temp file
        let _ = std::fs::remove_file(temp_image_path);
        return Err(anyhow!("No faces detected in the captured image. Please ensure your face is clearly visible and try again."));
    }

    if faces.len() > 1 {
        println!("Warning: Multiple faces detected. Using the first detected face.");
    }

    let face = &faces[0];
    println!("Face detected! Extracting features...");

    // Load database
    let database = FaceDatabase::load()?;

    if database.get_all_faces().is_empty() {
        // Clean up temp file
        let _ = std::fs::remove_file(temp_image_path);
        return Err(anyhow!("No registered faces found. Please register a face first by selecting option 1."));
    }

    println!("Comparing with {} registered face(s)...", database.get_all_faces().len());

    // Find best match
    let mut best_match: Option<(String, f64)> = None;

    for stored_face in database.get_all_faces() {
        let similarity = FaceDetector::compute_similarity(&face.features, &stored_face.features);

        match &best_match {
            None => best_match = Some((stored_face.user_id.clone(), similarity)),
            Some((_, best_similarity)) => {
                if similarity > *best_similarity {
                    best_match = Some((stored_face.user_id.clone(), similarity));
                }
            }
        }
    }

    // Clean up temp file
    let _ = std::fs::remove_file(temp_image_path);

    if let Some((user_id, confidence)) = best_match {
        // Define threshold for successful authentication (tuned based on testing)
        let threshold = 0.70;

        let is_match = confidence >= threshold;

        println!("Best match: User '{}' with confidence: {:.2}%", user_id, confidence * 100.0);

        Ok(AuthenticationResult {
            is_match,
            confidence,
            matched_user_id: Some(user_id),
        })
    } else {
        Ok(AuthenticationResult {
            is_match: false,
            confidence: 0.0,
            matched_user_id: None,
        })
    }
}