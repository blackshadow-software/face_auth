# 🚀 Setup Instructions

## For New Team Members

### 1. Clone Repository
```bash
git clone <your-repo-url>
cd face_auth
```

### 2. Setup Rust Environment
```bash
# Build Rust components
cargo build --release
```

### 3. Setup Python Environment (for High Accuracy)
```bash
# Run setup script (creates face_auth_env/ directory)
./setup_python_env.sh

# Activate environment
source face_auth_env/bin/activate
```

### 4. Test Both Systems
```bash
# Test Rust (fast, 66% accuracy)
./target/release/face_auth

# Test Python (99% accuracy)
source face_auth_env/bin/activate
python3 python_face_auth_simple.py --mode register --user test
python3 python_face_auth_simple.py --mode auth
```

## 📁 Directory Structure

```
face_auth/
├── src/                    # Rust source code (committed)
├── target/                 # Rust build artifacts (ignored)
├── face_auth_env/          # Python virtual env (ignored - 401MB)
├── captured_images/        # Face photos (ignored - privacy)
├── *_database.json         # Face data (ignored - privacy)
├── requirements.txt        # Python deps (committed)
├── setup_python_env.sh     # Setup script (committed)
├── python_face_auth*.py    # Python source (committed)
└── README.md              # Documentation (committed)
```

## 🔒 Privacy Notes

- Face images and databases are automatically excluded from git
- Each developer needs to register their own faces locally
- No face data is shared between team members