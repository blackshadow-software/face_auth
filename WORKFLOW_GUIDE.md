# Face Authentication System - Workflow Guide

## System Architecture

The face authentication system has been redesigned with a clear separation between registration and authentication:

```
┌─────────────────────────────────────────────────────────────┐
│                    REGISTRATION FLOW                         │
└─────────────────────────────────────────────────────────────┘
   User registers → Face captured → Encodings generated
                                           ↓
                              Saved to TWO locations:
                              1. python_face_database.json (legacy)
                              2. generated/{username}.json ✨ NEW
```

```
┌─────────────────────────────────────────────────────────────┐
│                   AUTHENTICATION FLOW                        │
└─────────────────────────────────────────────────────────────┘
   User authenticates → Face captured → Load encodings from:
                                        source/*.json ✨ NEW
                                             ↓
                                        Compare & match
```

## Key Directories

| Directory | Purpose | Created By | Used For |
|-----------|---------|------------|----------|
| `generated/` | Stores registered user face encodings | Registration | Backup/transfer |
| `source/` | Stores authorized users for authentication | Manual copy | Authentication |
| `captured_images/` | Temporary camera captures | Both | Debugging |

## Workflow

### 1. Register New Users

**Option A: Using Rust Application**
```bash
cargo run
# Select: 1 (Register)
# Enter username: john
# Look at camera (3 samples will be captured)
```

**Option B: Using Python Script Directly**
```bash
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user john --samples 3
```

**What Happens:**
- ✅ Captures 3 face samples from camera
- ✅ Generates 128-dimensional face encodings
- ✅ Saves to `python_face_database.json`
- ✅ **NEW:** Saves to `generated/john.json`

**Result:**
```
generated/
└── john.json          # Face encoding for user 'john'
```

### 2. Authorize Users for Authentication

To enable a user for authentication, copy their file from `generated/` to `source/`:

```bash
# Authorize john for authentication
cp generated/john.json source/

# Authorize multiple users
cp generated/alice.json generated/bob.json source/

# Authorize all registered users
cp generated/*.json source/
```

**Result:**
```
source/
├── john.json          # John can now authenticate
├── alice.json         # Alice can now authenticate
└── bob.json           # Bob can now authenticate
```

### 3. Authenticate Users

**Option A: Using Rust Application**
```bash
cargo run
# Select: 2 (Authenticate)
# Look at camera
```

**Option B: Using Python Script Directly**
```bash
./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.6
```

**What Happens:**
- ✅ Captures face from camera
- ✅ **NEW:** Loads all `.json` files from `source/` directory
- ✅ Compares captured face against loaded encodings
- ✅ Reports match if distance ≤ tolerance (default: 0.6)

**Sample Output (Success):**
```
Starting authentication...
Found 3 user file(s) in 'source' directory
Comparing against users from source/ directory...
User john: distance = 0.234
User alice: distance = 0.891
User bob: distance = 0.945

Authentication successful!
User: john
Distance: 0.234
Confidence: 76.6%
```

**Sample Output (Failure):**
```
Starting authentication...
Found 3 user file(s) in 'source' directory
Comparing against users from source/ directory...
User john: distance = 0.789
User alice: distance = 0.856
User bob: distance = 0.923

Authentication failed!
Closest match: john (distance: 0.789)
Threshold: 0.600
```

## Use Cases

### Use Case 1: Register Multiple Users

```bash
# Register three users
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user alice --samples 3
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user bob --samples 3
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user charlie --samples 3

# Check generated files
ls -lh generated/
# Output:
# alice.json
# bob.json
# charlie.json
```

### Use Case 2: Selective Authentication

Only authorize specific users:

```bash
# Only alice and bob can authenticate
cp generated/alice.json generated/bob.json source/

# charlie.json stays in generated/ - not authorized
```

### Use Case 3: Transfer Users Between Devices

**On Device A (source device):**
```bash
# Register user
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user john --samples 3

# Generated file is at: generated/john.json
# Copy this file to Device B
```

**On Device B (target device):**
```bash
# Place received file in source/
cp /path/to/john.json source/

# john can now authenticate on Device B
./face_auth_env/bin/python python_face_auth_simple.py --mode auth
```

### Use Case 4: Revoke Access

Remove a user's authentication ability:

```bash
# Revoke bob's access
rm source/bob.json

# bob's data still exists in generated/ but he can't authenticate
```

## Testing

### Quick Test with Mock Data

```bash
# Create mock users (no camera needed)
./face_auth_env/bin/python test_mock_data.py

# This creates:
# - generated/alice.json, bob.json, charlie.json
# - source/alice.json, bob.json

# Test loading logic
./face_auth_env/bin/python test_auth_loading.py
```

### Test with Real Camera

```bash
# 1. Clear mock data
rm -rf generated/* source/*

# 2. Register yourself
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user myname --samples 3

# 3. Authorize yourself
cp generated/myname.json source/

# 4. Test authentication
./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.6

# 5. Or use Rust app
cargo run
```

## File Format

Each user file (`generated/username.json` or `source/username.json`) contains:

```json
{
  "user_id": "john",
  "face_encodings": [
    {
      "encoding": [0.123, 0.456, ... 128 values],
      "timestamp": "2025-10-05T13:07:14.745572",
      "image_path": "captured_images/registration_john_...",
      "sample_id": "john_20251005_130714_123456"
    },
    { ... sample 2 ... },
    { ... sample 3 ... }
  ],
  "enrollment_date": "2025-10-05T13:07:14.745572",
  "sample_count": 3
}
```

## Troubleshooting

### Problem: "Error: 'source' directory does not exist"

**Solution:**
```bash
mkdir -p source
cp generated/*.json source/
```

### Problem: "No user files found in 'source' directory"

**Solution:**
```bash
# Check what's in generated
ls generated/

# Copy users to source
cp generated/username.json source/
```

### Problem: Authentication always fails

**Possible causes:**
1. **No users in source/**: Copy at least one user from `generated/`
2. **Poor lighting**: Ensure good lighting during both registration and authentication
3. **Face angle**: Look directly at camera
4. **Tolerance too strict**: Try higher tolerance (e.g., 0.8)

```bash
# Test with higher tolerance
./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.8
```

### Problem: Distance values seem wrong

**Face distance interpretation:**
- `< 0.4`: Very likely the same person (high confidence)
- `0.4 - 0.6`: Likely the same person (medium confidence)
- `0.6 - 0.8`: Uncertain (low confidence)
- `> 0.8`: Different person

## Advanced Usage

### Custom Tolerance

```bash
# Stricter matching (fewer false positives)
./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.4

# Looser matching (fewer false negatives)
./face_auth_env/bin/python python_face_auth_simple.py --mode auth --tolerance 0.8
```

### More Face Samples

```bash
# Register with 5 samples for better accuracy
./face_auth_env/bin/python python_face_auth_simple.py --mode register --user john --samples 5
```

### Batch Operations

```bash
# Authorize all users
cp generated/*.json source/

# Remove all authorized users
rm source/*.json

# Backup generated users
tar -czf face_auth_backup.tar.gz generated/
```

## Status Commands

Check system status:

```bash
# View registered users
ls -lh generated/

# View authorized users
ls -lh source/

# Count users
echo "Registered: $(ls -1 generated/*.json 2>/dev/null | wc -l)"
echo "Authorized: $(ls -1 source/*.json 2>/dev/null | wc -l)"

# Or use the helper script
./test_workflow.sh
```

## Migration from Old System

If you have existing users in `python_face_database.json`:

```bash
# The database still works as before
# But authentication now ONLY uses source/ directory

# To migrate:
# 1. Re-register all users (this creates generated/ files)
# 2. Copy needed users to source/
cp generated/*.json source/
```

## Security Notes

⚠️ **Important:**
- `generated/` contains all registered users' face data
- `source/` controls who can authenticate
- Keep `generated/` backed up
- Use `source/` for access control
- Face encoding files contain biometric data - protect them

## Summary

✅ **Registration**: Saves to `generated/{username}.json`
✅ **Authentication**: Loads from `source/*.json`
✅ **Access Control**: Copy files from `generated/` to `source/` to authorize
✅ **Revocation**: Remove files from `source/` to revoke access
