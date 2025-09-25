use crate::face_detection::FaceDetector;
use crate::face_storage::FaceDatabase;
use crate::camera::CameraCapture;
use anyhow::{Result, anyhow};

pub fn register_face(image_path: &str) -> Result<()> {
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

    // Load or create database
    let mut database = FaceDatabase::load()?;

    // For simplicity, we'll use "user" as the default user ID
    // In a real application, you might want to allow custom user IDs
    let user_id = "user".to_string();

    // Check if user already exists
    let existing_faces: Vec<_> = database.get_all_faces().iter()
        .filter(|f| f.user_id == user_id)
        .collect();

    if !existing_faces.is_empty() {
        println!("Existing registration found for user '{}'. Replacing with new registration.", user_id);
        // Remove existing faces for this user
        database.faces.retain(|f| f.user_id != user_id);
    }

    // Add new face to database
    database.add_face(user_id.clone(), face.features.clone())?;

    println!("Successfully registered face for user '{}'", user_id);
    println!("Extracted {} feature points", face.features.len());

    Ok(())
}

pub fn register_face_from_camera() -> Result<()> {
    println!("Initializing camera and face detector...");
    let mut camera = CameraCapture::new()?;
    let detector = FaceDetector::new()?;

    // Capture image from camera
    let temp_image_path = "temp_registration.jpg";
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

    // Load or create database
    let mut database = FaceDatabase::load()?;

    // For simplicity, we'll use "user" as the default user ID
    let user_id = "user".to_string();

    // Check if user already exists
    let existing_faces: Vec<_> = database.get_all_faces().iter()
        .filter(|f| f.user_id == user_id)
        .collect();

    if !existing_faces.is_empty() {
        println!("Existing registration found for user '{}'. Replacing with new registration.", user_id);
        // Remove existing faces for this user
        database.faces.retain(|f| f.user_id != user_id);
    }

    // Add new face to database
    database.add_face(user_id.clone(), face.features.clone())?;

    // Clean up temp file
    let _ = std::fs::remove_file(temp_image_path);

    println!("Successfully registered face for user '{}'", user_id);
    println!("Extracted {} feature points", face.features.len());

    Ok(())
}