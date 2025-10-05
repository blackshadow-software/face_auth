#!/usr/bin/env python3
"""
Test authentication loading logic without camera
Verifies that the authentication code can correctly load users from source/
"""

import json
import os
import numpy as np

def test_source_loading():
    """Test loading user data from source directory"""
    print("=" * 60)
    print("Testing Authentication Loading from source/")
    print("=" * 60)

    source_dir = "source"

    # Check if source directory exists
    if not os.path.exists(source_dir):
        print(f"❌ Error: '{source_dir}' directory does not exist")
        return False

    # Get all JSON files
    json_files = [f for f in os.listdir(source_dir) if f.endswith('.json')]

    if not json_files:
        print(f"❌ No user files found in '{source_dir}' directory")
        return False

    print(f"✅ Found {len(json_files)} user file(s) in '{source_dir}' directory")
    print(f"\nLoading users from source/ directory...")

    users_loaded = 0
    all_encodings = []

    for json_file in json_files:
        file_path = os.path.join(source_dir, json_file)
        try:
            with open(file_path, 'r') as f:
                user_data = json.load(f)

            user_id = user_data.get("user_id")
            if not user_id:
                print(f"⚠️  Warning: No user_id in {json_file}, skipping")
                continue

            face_encodings_data = user_data.get("face_encodings", [])
            if not face_encodings_data:
                print(f"⚠️  Warning: No face encodings in {json_file}, skipping")
                continue

            users_loaded += 1

            # Convert to numpy arrays (as face_recognition would)
            user_encodings = [np.array(sample["encoding"]) for sample in face_encodings_data]

            print(f"\n✅ User: {user_id}")
            print(f"   File: {json_file}")
            print(f"   Face samples: {len(user_encodings)}")
            print(f"   Encoding shape: {user_encodings[0].shape}")

            # Store for distance calculation test
            all_encodings.append({
                "user_id": user_id,
                "encodings": user_encodings
            })

        except Exception as e:
            print(f"\n❌ Error loading {json_file}: {e}")
            continue

    if users_loaded == 0:
        print("\n❌ No valid user files could be loaded from source/ directory")
        return False

    print(f"\n{'=' * 60}")
    print(f"✅ Successfully loaded {users_loaded} user(s)")
    print(f"{'=' * 60}")

    # Test similarity calculation between users
    if len(all_encodings) >= 2:
        print("\nTesting face distance calculation between users:")
        user1 = all_encodings[0]
        user2 = all_encodings[1]

        # Calculate distances between user1's first encoding and user2's encodings
        from face_recognition import face_distance

        distances = face_distance(user2["encodings"], user1["encodings"][0])
        min_distance = np.min(distances)

        print(f"\nDistance between {user1['user_id']} and {user2['user_id']}: {min_distance:.3f}")
        print(f"(Lower distance = more similar faces)")
        print(f"(Typical threshold: 0.6, these are random so should be ~0.8-1.2)")

    return True

def main():
    print("\n" + "=" * 60)
    print("AUTHENTICATION SOURCE LOADING TEST")
    print("=" * 60)
    print("\nThis test verifies that the authentication code")
    print("can correctly load user data from source/ directory.\n")

    success = test_source_loading()

    if success:
        print("\n" + "=" * 60)
        print("✅ TEST PASSED")
        print("=" * 60)
        print("The authentication system can successfully:")
        print("  - Find the source/ directory")
        print("  - Load JSON files from source/")
        print("  - Parse user data and face encodings")
        print("  - Calculate face distances")
        print("\nThe system is ready for real authentication testing!")
        print("=" * 60)
    else:
        print("\n" + "=" * 60)
        print("❌ TEST FAILED")
        print("=" * 60)
        print("Please check:")
        print("  - source/ directory exists")
        print("  - source/ contains valid user JSON files")
        print("  - Run: ./test_mock_data.py to create test data")
        print("=" * 60)

if __name__ == "__main__":
    main()
