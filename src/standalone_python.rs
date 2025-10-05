use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::Path;

pub struct StandalonePythonFaceAuth {
    executable_path: String,
}

impl StandalonePythonFaceAuth {
    pub fn new() -> Result<Self> {
        // Use the Python script directly with virtual environment instead of broken PyInstaller
        let executable_path = "./face_auth_env/bin/python";

        if !Path::new(executable_path).exists() {
            return Err(anyhow!("Python virtual environment not found at: {}\nPlease run: ./setup_python_env.sh", executable_path));
        }

        Ok(Self {
            executable_path: executable_path.to_string(),
        })
    }

    pub fn register_user(&self, username: &str, samples: u32) -> Result<bool> {
        println!("🦀 Using standalone Python executable (NO Python install required)");
        println!("📦 Executable: {}", self.executable_path);

        let output = Command::new(&self.executable_path)
            .arg("python_face_auth_simple.py")
            .arg("--mode")
            .arg("register")
            .arg("--user")
            .arg(username)
            .arg("--samples")
            .arg(&samples.to_string())
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("📱 Standalone Python output:\n{}", stdout);

            // Check if registration was successful
            if stdout.contains("Registration complete") || stdout.contains("samples stored") {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Standalone Python registration failed: {}", stderr))
        }
    }

    pub fn authenticate_user(&self, tolerance: f64) -> Result<StandaloneAuthResult> {
        println!("🦀 Using standalone Python executable (NO Python install required)");
        println!("📦 Executable: {}", self.executable_path);

        let output = Command::new(&self.executable_path)
            .arg("python_face_auth_simple.py")
            .arg("--mode")
            .arg("auth")
            .arg("--tolerance")
            .arg(&tolerance.to_string())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("📱 Standalone Python output:\n{}", stdout);

        if !stderr.is_empty() {
            println!("⚠️ Standalone Python stderr:\n{}", stderr);
        }

        // Parse the output to determine authentication result
        let success = output.status.success();
        let is_match = stdout.contains("Authentication successful") || stdout.contains("✅");
        let confidence = extract_confidence_from_output(&stdout);
        let distance = extract_distance_from_output(&stdout);
        let matched_user = extract_matched_user_from_output(&stdout);
        let processing_time = extract_processing_time_from_output(&stdout);

        Ok(StandaloneAuthResult {
            success,
            is_match: Some(is_match),
            confidence,
            distance,
            threshold: Some(tolerance),
            matched_user,
            processing_time_ms: processing_time,
            raw_output: stdout.to_string(),
        })
    }

    pub fn export_user(&self, username: &str, filename: &str) -> Result<bool> {
        let mut cmd = Command::new(&self.executable_path);
        cmd.arg("python_face_auth_simple.py")
            .arg("--mode")
            .arg("export")
            .arg("--user")
            .arg(username);

        // Only add --file argument if filename is not empty
        if !filename.is_empty() {
            cmd.arg("--file").arg(filename);
        }

        let output = cmd.output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
            Ok(stdout.contains("exported successfully"))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}", stderr);
            Ok(false)
        }
    }

    pub fn import_user(&self, filename: &str) -> Result<bool> {
        let output = Command::new(&self.executable_path)
            .arg("python_face_auth_simple.py")
            .arg("--mode")
            .arg("import")
            .arg("--file")
            .arg(filename)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
            Ok(stdout.contains("imported successfully"))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}", stderr);
            Ok(false)
        }
    }

    pub fn list_users(&self) -> Result<()> {
        let output = Command::new(&self.executable_path)
            .arg("python_face_auth_simple.py")
            .arg("--mode")
            .arg("list")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            println!("{}", stderr);
        }

        Ok(())
    }

    pub fn check_executable(&self) -> Result<()> {
        if !Path::new(&self.executable_path).exists() {
            return Err(anyhow!("Standalone executable not found: {}", self.executable_path));
        }

        // Test the executable
        let output = Command::new(&self.executable_path)
            .arg("python_face_auth_simple.py")
            .arg("--help")
            .output()?;

        if output.status.success() {
            println!("✅ Standalone Python executable is working");
            println!("📏 File size: {} MB",
                std::fs::metadata(&self.executable_path)
                    .map(|m| m.len() / 1024 / 1024)
                    .unwrap_or(0)
            );
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Standalone executable test failed: {}", stderr))
        }
    }
}

#[derive(Debug)]
pub struct StandaloneAuthResult {
    pub success: bool,
    pub is_match: Option<bool>,
    pub confidence: Option<f64>,
    pub distance: Option<f64>,
    pub threshold: Option<f64>,
    pub matched_user: Option<String>,
    pub processing_time_ms: Option<u32>,
    pub raw_output: String,
}

// Helper functions to parse output
fn extract_confidence_from_output(output: &str) -> Option<f64> {
    // Look for patterns like "Confidence: 95.2%" or "confidence: 0.952"
    for line in output.lines() {
        if let Some(start) = line.find("onfidence: ") {
            let substr = &line[start + 11..];
            if let Some(end) = substr.find('%') {
                if let Ok(val) = substr[..end].parse::<f64>() {
                    return Some(val / 100.0);
                }
            } else if let Some(space_end) = substr.find(' ') {
                if let Ok(val) = substr[..space_end].parse::<f64>() {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn extract_distance_from_output(output: &str) -> Option<f64> {
    // Look for patterns like "Distance: 0.342" or "distance: 0.342"
    for line in output.lines() {
        if let Some(start) = line.find("istance: ") {
            let substr = &line[start + 9..];
            if let Some(space_end) = substr.find(' ') {
                if let Ok(val) = substr[..space_end].parse::<f64>() {
                    return Some(val);
                }
            } else if let Ok(val) = substr.trim().parse::<f64>() {
                return Some(val);
            }
        }
    }
    None
}

fn extract_matched_user_from_output(output: &str) -> Option<String> {
    // Look for patterns like "User: username" or "Matched user: username"
    for line in output.lines() {
        if let Some(start) = line.find("ser: ") {
            let substr = &line[start + 5..];
            if let Some(end) = substr.find('\n') {
                return Some(substr[..end].trim().to_string());
            } else {
                return Some(substr.trim().to_string());
            }
        }
    }
    None
}

fn extract_processing_time_from_output(output: &str) -> Option<u32> {
    // Look for patterns like "Processing time: 1234ms" or "took 1234 ms"
    for line in output.lines() {
        if let Some(start) = line.find("rocessing time: ") {
            let substr = &line[start + 16..];
            if let Some(ms_pos) = substr.find("ms") {
                if let Ok(val) = substr[..ms_pos].trim().parse::<u32>() {
                    return Some(val);
                }
            }
        } else if let Some(start) = line.find("took ") {
            let substr = &line[start + 5..];
            if let Some(ms_pos) = substr.find(" ms") {
                if let Ok(val) = substr[..ms_pos].trim().parse::<u32>() {
                    return Some(val);
                }
            }
        }
    }
    None
}