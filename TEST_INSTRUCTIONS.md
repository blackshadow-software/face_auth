# Face Authentication System - Testing Instructions

## Overview
The system has been updated with the following behavior:
- **Registration**: Saves face encodings to `generated/{username}.json` for each user
- **Authentication**: Matches faces against files in the `source/` directory

## Directory Structure
```
face_auth/
├── generated/          # Auto-created during registration
│   ├── user1.json     # Face encoding for user1
│   └── user2.json     # Face encoding for user2
└── source/            # Manually managed - copy files here for authentication
    ├── user1.json     # Copy from generated/ to enable authentication
    └── user2.json     # Copy from generated/ to enable authentication
```

## Testing Workflow

### 1. Register Users
Run the application and select option 1 (Register):
```bash
cargo run
# Select: 1 (Register)
# Enter username: user1
# Look at camera when prompted (3 samples will be captured)
```

This will:
- Capture 3 face samples
- Save to `python_face_database.json`
- **NEW**: Save to `generated/user1.json`

Repeat for more users (user2, user3, etc.)

### 2. Prepare Authentication Files
Copy user files from `generated/` to `source/` directory:
```bash
# Create source directory
mkdir -p source

# Copy users you want to authenticate
cp generated/user1.json source/
cp generated/user2.json source/
```

### 3. Test Authentication
Run the application and select option 2 (Authenticate):
```bash
cargo run
# Select: 2 (Authenticate)
# Look at camera when prompted
```

The system will:
- Capture your face
- Load all `.json` files from `source/` directory
- Match against loaded user encodings
- Display authentication result

## Quick Test Commands

### Register a user (via Python directly)
```bash
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user testuser --samples 3
```

### Check generated files
```bash
ls -la generated/
cat generated/testuser.json | jq '.user_id'
```

### Prepare source directory
```bash
mkdir -p source
cp generated/testuser.json source/
ls -la source/
```

### Test authentication (via Python directly)
```bash
./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.6
```

### Test via Rust application
```bash
cargo run
```

## Expected Behavior

### Successful Registration
```
Starting registration for user: alice
Will capture 3 samples
--- Sample 1/3 ---
[Camera captures face]
Sample 1 processed successfully
[Repeat for samples 2 and 3]
✅ User data saved to: generated/alice.json
Registration complete! 3 samples stored for alice
```

### Successful Authentication
```
Starting authentication...
[Camera captures face]
Found 2 user file(s) in 'source' directory
Comparing against users from source/ directory...
User alice: distance = 0.234
User bob: distance = 0.891
Authentication successful!
User: alice
Distance: 0.234
Confidence: 76.6%
```

### Failed Authentication (No source files)
```
Starting authentication...
[Camera captures face]
Error: 'source' directory does not exist
Please create 'source' directory and add user face encoding files
```

## Troubleshooting

### Error: "No user files found in 'source' directory"
**Solution**: Copy at least one user file from `generated/` to `source/`:
```bash
cp generated/*.json source/
```

### Error: "source directory does not exist"
**Solution**: Create the source directory:
```bash
mkdir -p source
```

### Low confidence / Authentication fails
**Solution**:
- Ensure good lighting
- Look directly at camera
- Try adjusting tolerance (default 0.6, lower = stricter)
- Re-register with better quality samples
