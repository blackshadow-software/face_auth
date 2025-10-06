use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::Path;

pub struct StandalonePythonFaceAuth {
    executable_path: String,
    script_path: String,
}

impl StandalonePythonFaceAuth {
    pub fn new() -> Result<Self> {
        // Find Python script first
        let script_path = Self::find_script_path()?;

        // Try to find or setup Python environment
        let executable_path = Self::find_or_setup_python()?;

        // Verify dependencies are installed
        Self::ensure_dependencies(&executable_path)?;

        Ok(Self {
            executable_path,
            script_path,
        })
    }

    fn find_script_path() -> Result<String> {
        let script_paths = vec![
            "python_face_auth_simple.py",
            "../python_face_auth_simple.py",
            "../../python_face_auth_simple.py",
        ];

        for path in &script_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(anyhow!(
            "Python script not found. Tried:\n{}",
            script_paths.join("\n")
        ))
    }

    fn find_or_setup_python() -> Result<String> {
        println!("🔍 Searching for Python environment...");

        // First, try to find existing virtual environment
        let venv_paths = vec![
            "./face_auth_env/bin/python",
            "../face_auth_env/bin/python",
            "../../face_auth_env/bin/python",
        ];

        for path in &venv_paths {
            if Path::new(path).exists() {
                println!("✅ Found virtual environment at: {}", path);
                return Ok(path.to_string());
            }
        }

        println!("⚠️  Virtual environment not found");
        println!("🔧 Attempting to create virtual environment automatically...");

        // Try to create virtual environment
        if let Ok(python_path) = Self::create_virtual_environment() {
            return Ok(python_path);
        }

        println!("⚠️  Could not create virtual environment");
        println!("🔍 Falling back to system Python...");

        // Fallback to system Python
        Self::find_system_python()
    }

    fn create_virtual_environment() -> Result<String> {
        // Check if python3 is available
        let python_check = Command::new("python3")
            .arg("--version")
            .output();

        if python_check.is_err() {
            return Err(anyhow!("python3 not found in system"));
        }

        println!("📦 Creating virtual environment at ./face_auth_env...");

        // Create virtual environment
        let output = Command::new("python3")
            .args(&["-m", "venv", "face_auth_env"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create venv: {}", stderr));
        }

        let venv_python = "./face_auth_env/bin/python";
        if Path::new(venv_python).exists() {
            println!("✅ Virtual environment created successfully");
            Ok(venv_python.to_string())
        } else {
            Err(anyhow!("Virtual environment created but python not found"))
        }
    }

    fn find_system_python() -> Result<String> {
        // Try different Python commands
        let python_commands = vec!["python3", "python"];

        for cmd in python_commands {
            let check = Command::new(cmd)
                .arg("--version")
                .output();

            if let Ok(output) = check {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("✅ Found system Python: {} ({})", cmd, version.trim());
                    return Ok(cmd.to_string());
                }
            }
        }

        Err(anyhow!(
            "No Python installation found. Please install Python 3.8+ from:\n\
             - macOS: brew install python3\n\
             - Linux: sudo apt install python3 python3-pip\n\
             - Windows: https://www.python.org/downloads/"
        ))
    }

    fn ensure_dependencies(python_path: &str) -> Result<()> {
        println!("🔍 Checking Python dependencies...");

        // Check if face_recognition is installed
        let check = Command::new(python_path)
            .args(&["-c", "import face_recognition; import cv2; print('OK')"])
            .output();

        if let Ok(output) = check {
            if output.status.success() && String::from_utf8_lossy(&output.stdout).contains("OK") {
                println!("✅ All dependencies are installed");
                return Ok(());
            }
        }

        println!("⚠️  Required dependencies not found");
        println!("📦 Installing dependencies automatically...");

        Self::install_dependencies(python_path)
    }

    fn install_dependencies(python_path: &str) -> Result<()> {
        // First ensure pip is up to date
        println!("📦 Upgrading pip...");
        let _ = Command::new(python_path)
            .args(&["-m", "pip", "install", "--upgrade", "pip"])
            .output();

        // Install each required package
        let packages = vec![
            "numpy>=1.21.0",
            "Pillow>=9.0.0",
            "cmake>=3.18.0",
            "dlib>=19.24.0",
            "opencv-python>=4.8.0",
            "face_recognition>=1.3.0",
        ];

        for (i, package) in packages.iter().enumerate() {
            println!("📦 Installing {}/{}: {}", i + 1, packages.len(), package);

            let output = Command::new(python_path)
                .args(&["-m", "pip", "install", package])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("⚠️  Warning: Failed to install {}: {}", package, stderr);

                // For critical packages, fail
                if package.contains("face_recognition") {
                    return Err(anyhow!(
                        "Failed to install face_recognition. This is required.\n\
                         On macOS, you may need to install: brew install cmake\n\
                         Error: {}", stderr
                    ));
                }
            } else {
                println!("✅ Installed {}", package);
            }
        }

        println!("✅ All dependencies installed successfully!");
        Ok(())
    }

    pub fn register_user(&self, username: &str, samples: u32, generated_dir: &str) -> Result<bool> {
        println!("🦀 Using standalone Python executable (NO Python install required)");
        println!("📦 Executable: {}", self.executable_path);

        let output = Command::new(&self.executable_path)
            .arg(&self.script_path)
            .arg("--mode")
            .arg("register")
            .arg("--user")
            .arg(username)
            .arg("--samples")
            .arg(&samples.to_string())
            .arg("--generated-dir")
            .arg(generated_dir)
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

    pub fn authenticate_user(&self, tolerance: f64, source_dir: &str) -> Result<StandaloneAuthResult> {
        println!("🦀 Using standalone Python executable (NO Python install required)");
        println!("📦 Executable: {}", self.executable_path);

        let output = Command::new(&self.executable_path)
            .arg(&self.script_path)
            .arg("--mode")
            .arg("auth")
            .arg("--tolerance")
            .arg(&tolerance.to_string())
            .arg("--source-dir")
            .arg(source_dir)
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
        cmd.arg(&self.script_path)
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
            .arg(&self.script_path)
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
            .arg(&self.script_path)
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
            .arg(&self.script_path)
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