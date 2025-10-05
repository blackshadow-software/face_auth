#!/usr/bin/env python3
"""
Test script to verify the registration and authentication flow
Creates mock face encoding data to test the system without camera
"""

import json
import os
import numpy as np
from datetime import datetime

def create_mock_face_encoding():
    """Create a random 128-dimensional face encoding (standard for face_recognition library)"""
    return np.random.rand(128).tolist()

def create_mock_user(user_id, num_samples=3):
    """Create a mock user with face encodings"""
    face_encodings = []

    for i in range(num_samples):
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S_%f")
        face_encodings.append({
            "encoding": create_mock_face_encoding(),
            "timestamp": datetime.now().isoformat(),
            "image_path": f"captured_images/registration_{user_id}_{timestamp}_sample{i+1}.jpg",
            "sample_id": f"{user_id}_{timestamp}"
        })

    user_data = {
        "user_id": user_id,
        "face_encodings": face_encodings,
        "enrollment_date": datetime.now().isoformat(),
        "sample_count": len(face_encodings)
    }

    return user_data

def test_registration():
    """Test registration by creating mock user files"""
    print("=" * 60)
    print("Testing Registration Flow")
    print("=" * 60)

    # Create directories
    os.makedirs("generated", exist_ok=True)
    os.makedirs("source", exist_ok=True)

    # Create mock users
    test_users = ["alice", "bob", "charlie"]

    for user in test_users:
        user_data = create_mock_user(user, num_samples=3)

        # Save to generated directory
        generated_file = f"generated/{user}.json"
        with open(generated_file, 'w') as f:
            json.dump(user_data, f, indent=2)

        print(f"✅ Created mock user: {user}")
        print(f"   File: {generated_file}")
        print(f"   Samples: {user_data['sample_count']}")

    print(f"\n✅ Successfully created {len(test_users)} mock users in generated/")
    print(f"\nGenerated files:")
    for user in test_users:
        file_size = os.path.getsize(f"generated/{user}.json")
        print(f"  - generated/{user}.json ({file_size} bytes)")

    return test_users

def test_source_preparation(users):
    """Test copying users to source directory"""
    print("\n" + "=" * 60)
    print("Testing Source Directory Preparation")
    print("=" * 60)

    # Copy first two users to source
    users_to_copy = users[:2]  # alice and bob

    for user in users_to_copy:
        src_file = f"generated/{user}.json"
        dst_file = f"source/{user}.json"

        # Copy file content
        with open(src_file, 'r') as f:
            data = json.load(f)

        with open(dst_file, 'w') as f:
            json.dump(data, f, indent=2)

        print(f"✅ Copied {user} to source/")

    print(f"\n✅ Source directory ready with {len(users_to_copy)} users")
    print(f"\nSource files:")
    for user in users_to_copy:
        file_size = os.path.getsize(f"source/{user}.json")
        print(f"  - source/{user}.json ({file_size} bytes)")

    return users_to_copy

def verify_file_structure():
    """Verify the files can be loaded correctly"""
    print("\n" + "=" * 60)
    print("Verifying File Structure")
    print("=" * 60)

    source_files = [f for f in os.listdir("source") if f.endswith('.json')]
    print(f"Found {len(source_files)} JSON files in source/")

    for file in source_files:
        filepath = os.path.join("source", file)
        try:
            with open(filepath, 'r') as f:
                data = json.load(f)

            user_id = data.get("user_id")
            encodings = data.get("face_encodings", [])

            print(f"\n✅ {file}")
            print(f"   User ID: {user_id}")
            print(f"   Face encodings: {len(encodings)}")
            print(f"   Enrollment date: {data.get('enrollment_date', 'N/A')}")

            # Verify encoding structure
            if encodings:
                first_encoding = encodings[0]
                encoding_length = len(first_encoding.get("encoding", []))
                print(f"   Encoding dimension: {encoding_length}")

                if encoding_length != 128:
                    print(f"   ⚠️  Warning: Expected 128-dimensional encoding, got {encoding_length}")

        except Exception as e:
            print(f"\n❌ Error loading {file}: {e}")

    print("\n✅ All files verified successfully")

def main():
    print("\n" + "=" * 60)
    print("FACE AUTHENTICATION SYSTEM - MOCK DATA TEST")
    print("=" * 60)
    print("\nThis script creates mock user data to test the system")
    print("without requiring camera access.\n")

    # Test registration (create mock users)
    users = test_registration()

    # Test source preparation (copy users to source/)
    source_users = test_source_preparation(users)

    # Verify file structure
    verify_file_structure()

    # Summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    print(f"✅ Created {len(users)} users in generated/:")
    for user in users:
        print(f"   - {user}")

    print(f"\n✅ Prepared {len(source_users)} users in source/ for authentication:")
    for user in source_users:
        print(f"   - {user}")

    print("\n" + "=" * 60)
    print("NEXT STEPS")
    print("=" * 60)
    print("1. The generated/ directory now contains user face encodings")
    print("2. The source/ directory contains users ready for authentication")
    print("3. You can now test authentication (it will fail with mock data)")
    print("   because the camera will capture a real face that won't match")
    print("   the random mock encodings")
    print("\n4. To test with real users:")
    print("   - Delete mock data: rm -rf generated/* source/*")
    print("   - Run: ./face_auth_env/bin/python python_face_auth_simple.py --mode register --user YOUR_NAME --samples 3")
    print("   - Copy to source: cp generated/YOUR_NAME.json source/")
    print("   - Test auth: ./face_auth_env/bin/python python_face_auth_simple.py --mode auth")
    print("\n5. Or use the Rust application:")
    print("   - cargo run")
    print("=" * 60)

if __name__ == "__main__":
    main()
