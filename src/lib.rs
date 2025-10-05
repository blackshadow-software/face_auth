//! # Face Authentication Library
//!
//! A Rust library for face authentication using Python's face_recognition library.
//!
//! ## Features
//!
//! - User registration with face capture
//! - Face-based authentication
//! - User data export/import
//! - File-based access control
//!
//! ## Example
//!
//! ```no_run
//! use face_auth::{FaceAuth, FaceAuthResult};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let auth = FaceAuth::new()?;
//!
//!     // Register a user
//!     auth.register_user("john", 3).await?;
//!
//!     // Authenticate
//!     let result = auth.authenticate_user(0.6).await?;
//!
//!     if result.is_authenticated {
//!         println!("Welcome, {}!", result.user_id.unwrap_or_default());
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod standalone_python;

use anyhow::Result;
pub use standalone_python::{StandalonePythonFaceAuth, StandaloneAuthResult};

/// Main face authentication interface
pub struct FaceAuth {
    python_auth: StandalonePythonFaceAuth,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct FaceAuthResult {
    pub is_authenticated: bool,
    pub user_id: Option<String>,
    pub confidence: Option<f64>,
    pub distance: Option<f64>,
    pub threshold: Option<f64>,
    pub processing_time_ms: Option<u32>,
}

impl From<StandaloneAuthResult> for FaceAuthResult {
    fn from(result: StandaloneAuthResult) -> Self {
        Self {
            is_authenticated: result.is_match.unwrap_or(false),
            user_id: result.matched_user,
            confidence: result.confidence,
            distance: result.distance,
            threshold: result.threshold,
            processing_time_ms: result.processing_time_ms,
        }
    }
}

impl FaceAuth {
    /// Create a new FaceAuth instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            python_auth: StandalonePythonFaceAuth::new()?,
        })
    }

    /// Register a new user with face samples
    ///
    /// # Arguments
    ///
    /// * `username` - The username to register
    /// * `samples` - Number of face samples to capture (default: 3)
    /// * `generated_dir` - Directory path where user data will be saved
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if registration was successful, `Ok(false)` if it failed
    pub async fn register_user(&self, username: &str, samples: u32, generated_dir: &str) -> Result<bool> {
        self.python_auth.register_user(username, samples, generated_dir)
    }

    /// Authenticate a user by capturing their face
    ///
    /// # Arguments
    ///
    /// * `tolerance` - Face matching tolerance (0.0-1.0, lower = stricter)
    /// * `source_dir` - Directory path where user data is loaded from
    ///
    /// # Returns
    ///
    /// Returns authentication result with user information
    pub async fn authenticate_user(&self, tolerance: f64, source_dir: &str) -> Result<FaceAuthResult> {
        let result = self.python_auth.authenticate_user(tolerance, source_dir)?;
        Ok(result.into())
    }

    /// Export a user's face data to a file
    ///
    /// # Arguments
    ///
    /// * `username` - The username to export
    /// * `filename` - Optional filename (auto-generated if empty)
    pub async fn export_user(&self, username: &str, filename: &str) -> Result<bool> {
        self.python_auth.export_user(username, filename)
    }

    /// Import a user's face data from a file
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the file to import
    pub async fn import_user(&self, filename: &str) -> Result<bool> {
        self.python_auth.import_user(filename)
    }

    /// List all registered users
    pub async fn list_users(&self) -> Result<()> {
        self.python_auth.list_users()
    }

    /// Check if the Python executable is working
    pub async fn check_system(&self) -> Result<()> {
        self.python_auth.check_executable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_face_auth_creation() {
        let result = FaceAuth::new();
        assert!(result.is_ok());
    }
}
