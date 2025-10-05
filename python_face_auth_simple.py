#!/usr/bin/env python3
"""
Simplified Python Face Authentication for integration with Rust
Auto-captures without manual interaction
"""

import face_recognition
import cv2
import numpy as np
import json
import os
import time
import sys
from datetime import datetime
from typing import List, Dict, Tuple, Optional
import argparse

class SimpleFaceAuth:
    def __init__(self, db_path: str = "python_face_database.json"):
        self.db_path = db_path
        self.load_database()

    def load_database(self):
        """Load face database or create new one"""
        try:
            if os.path.exists(self.db_path):
                with open(self.db_path, 'r') as f:
                    self.database = json.load(f)
            else:
                self.database = {
                    "users": {},
                    "version": "1.0",
                    "accuracy_threshold": 0.6,
                    "created": datetime.now().isoformat()
                }
        except Exception as e:
            print(f"Error loading database: {e}")
            self.database = {"users": {}, "version": "1.0", "accuracy_threshold": 0.6}

    def save_database(self):
        """Save database to file"""
        try:
            with open(self.db_path, 'w') as f:
                json.dump(self.database, f, indent=2)
        except Exception as e:
            print(f"Error saving database: {e}")

    def auto_capture_image(self, save_path: str, delay_seconds: int = 2) -> bool:
        """Auto-capture image from camera after delay"""
        print(f"Initializing camera for auto-capture...")

        cap = cv2.VideoCapture(0)
        if not cap.isOpened():
            print("Error: Could not open camera")
            return False

        print(f"Camera ready! Auto-capturing in {delay_seconds} seconds...")
        print("Look directly at the camera and stay still...")

        # Wait for camera to stabilize
        for i in range(30):
            ret, frame = cap.read()
            if not ret:
                print("Error: Failed to read from camera")
                cap.release()
                return False

        # Countdown
        for i in range(delay_seconds, 0, -1):
            print(f"Capturing in {i}...")
            for j in range(30):  # ~1 second at 30 FPS
                ret, frame = cap.read()
                if ret:
                    # Show frame with countdown (optional)
                    display_frame = frame.copy()
                    cv2.putText(display_frame, f"Capturing in {i}...",
                               (50, 50), cv2.FONT_HERSHEY_SIMPLEX, 1, (0, 255, 0), 2)
                    cv2.imshow('Auto Capture', display_frame)
                    cv2.waitKey(1)

        # Capture the image
        ret, frame = cap.read()
        if ret:
            cv2.imwrite(save_path, frame)
            print(f"Image captured: {save_path}")

            # Show captured image briefly
            cv2.putText(frame, "CAPTURED!", (50, 50), cv2.FONT_HERSHEY_SIMPLEX, 1, (0, 255, 0), 2)
            cv2.imshow('Auto Capture', frame)
            cv2.waitKey(1000)  # Show for 1 second

            cap.release()
            cv2.destroyAllWindows()
            return True
        else:
            print("Error: Failed to capture image")
            cap.release()
            cv2.destroyAllWindows()
            return False

    def detect_and_encode_face(self, image_path: str) -> Optional[np.ndarray]:
        """Detect and encode a single face"""
        try:
            # Load image
            image = face_recognition.load_image_file(image_path)

            # Find face locations
            face_locations = face_recognition.face_locations(image, model="hog")

            if not face_locations:
                print("No face detected in image")
                return None

            if len(face_locations) > 1:
                print(f"Multiple faces detected ({len(face_locations)}), using the first one")

            # Generate face encoding
            face_encodings = face_recognition.face_encodings(image, face_locations)

            if face_encodings:
                print(f"Face encoding generated successfully")
                return face_encodings[0]
            else:
                print("Failed to generate face encoding")
                return None

        except Exception as e:
            print(f"Error processing image: {e}")
            return None

    def register_user(self, user_id: str, num_samples: int = 3) -> bool:
        """Register user with multiple face samples and save to generated/ directory"""
        print(f"Starting registration for user: {user_id}")
        print(f"Will capture {num_samples} samples")

        os.makedirs("captured_images", exist_ok=True)
        os.makedirs("generated", exist_ok=True)
        face_encodings = []

        for i in range(num_samples):
            print(f"\n--- Sample {i+1}/{num_samples} ---")

            # Capture image
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S_%f")
            image_path = f"captured_images/registration_{user_id}_{timestamp}_sample{i+1}.jpg"

            if not self.auto_capture_image(image_path, delay_seconds=2):
                print(f"Failed to capture sample {i+1}")
                continue

            # Process image
            encoding = self.detect_and_encode_face(image_path)
            if encoding is not None:
                face_encodings.append({
                    "encoding": encoding.tolist(),
                    "timestamp": datetime.now().isoformat(),
                    "image_path": image_path,
                    "sample_id": f"{user_id}_{timestamp}"
                })
                print(f"Sample {i+1} processed successfully")
            else:
                print(f"Failed to process sample {i+1}")

        if not face_encodings:
            print("No valid face samples captured")
            return False

        # Store in database
        if "users" not in self.database:
            self.database["users"] = {}

        self.database["users"][user_id] = {
            "user_id": user_id,
            "face_encodings": face_encodings,
            "enrollment_date": datetime.now().isoformat(),
            "sample_count": len(face_encodings)
        }

        self.save_database()

        # Save user's face encodings to generated/ directory
        generated_file = f"generated/{user_id}.json"
        user_data = {
            "user_id": user_id,
            "face_encodings": face_encodings,
            "enrollment_date": datetime.now().isoformat(),
            "sample_count": len(face_encodings)
        }

        try:
            with open(generated_file, 'w') as f:
                json.dump(user_data, f, indent=2)
            print(f"✅ User data saved to: {generated_file}")
        except Exception as e:
            print(f"⚠️ Warning: Failed to save to generated/ directory: {e}")

        print(f"Registration complete! {len(face_encodings)} samples stored for {user_id}")
        return True

    def authenticate_user(self, tolerance: float = 0.6) -> bool:
        """Authenticate user by matching against files in source/ directory"""
        print("Starting authentication...")

        # Capture authentication image
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        auth_image_path = f"captured_images/authentication_{timestamp}.jpg"

        os.makedirs("captured_images", exist_ok=True)

        if not self.auto_capture_image(auth_image_path, delay_seconds=2):
            print("Failed to capture authentication image")
            return False

        # Process authentication image
        auth_encoding = self.detect_and_encode_face(auth_image_path)
        if auth_encoding is None:
            print("No face detected in authentication image")
            return False

        # Load face encodings from source/ directory
        source_dir = "source"
        if not os.path.exists(source_dir):
            print(f"Error: '{source_dir}' directory does not exist")
            print(f"Please create '{source_dir}' directory and add user face encoding files")
            return False

        # Get all JSON files from source/ directory
        json_files = [f for f in os.listdir(source_dir) if f.endswith('.json')]

        if not json_files:
            print(f"No user files found in '{source_dir}' directory")
            print(f"Please add user face encoding JSON files to '{source_dir}' directory")
            return False

        print(f"Found {len(json_files)} user file(s) in '{source_dir}' directory")
        print(f"Comparing against users from source/ directory...")

        best_match = None
        best_distance = float('inf')
        users_loaded = 0

        for json_file in json_files:
            file_path = os.path.join(source_dir, json_file)
            try:
                with open(file_path, 'r') as f:
                    user_data = json.load(f)

                user_id = user_data.get("user_id")
                if not user_id:
                    print(f"Warning: No user_id in {json_file}, skipping")
                    continue

                face_encodings_data = user_data.get("face_encodings", [])
                if not face_encodings_data:
                    print(f"Warning: No face encodings in {json_file}, skipping")
                    continue

                users_loaded += 1
                user_encodings = [np.array(sample["encoding"]) for sample in face_encodings_data]
                distances = face_recognition.face_distance(user_encodings, auth_encoding)
                min_distance = np.min(distances)

                print(f"User {user_id}: distance = {min_distance:.3f}")

                if min_distance < best_distance:
                    best_distance = min_distance
                    best_match = user_id

            except Exception as e:
                print(f"Error loading {json_file}: {e}")
                continue

        if users_loaded == 0:
            print("No valid user files could be loaded from source/ directory")
            return False

        # Check if match is within tolerance
        if best_match and best_distance <= tolerance:
            confidence = max(0, 1 - best_distance)
            print(f"Authentication successful!")
            print(f"User: {best_match}")
            print(f"Distance: {best_distance:.3f}")
            print(f"Confidence: {confidence:.1%}")
            return True
        else:
            print(f"Authentication failed!")
            if best_match:
                print(f"Closest match: {best_match} (distance: {best_distance:.3f})")
                print(f"Threshold: {tolerance:.3f}")
            return False

    def export_user(self, user_id: str, export_path: str = None) -> bool:
        """Export a user's face data to a file"""
        if user_id not in self.database["users"]:
            print(f"User '{user_id}' not found in database")
            return False

        # Auto-generate filename if not provided
        if export_path is None:
            # Create exports directory if it doesn't exist
            export_dir = "exported_credentials"
            os.makedirs(export_dir, exist_ok=True)

            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            export_path = f"{export_dir}/{user_id}_credentials_{timestamp}.json"

        user_data = {
            "user_id": user_id,
            "user_data": self.database["users"][user_id],
            "exported_at": datetime.now().isoformat(),
            "version": self.database.get("version", "1.0")
        }

        try:
            with open(export_path, 'w') as f:
                json.dump(user_data, f, indent=2)
            print(f"User '{user_id}' exported successfully to {export_path}")
            return True
        except Exception as e:
            print(f"Error exporting user: {e}")
            return False

    def import_user(self, import_path: str) -> bool:
        """Import a user's face data from a file"""
        try:
            with open(import_path, 'r') as f:
                user_data = json.load(f)

            user_id = user_data["user_id"]

            # Check if user already exists
            if user_id in self.database["users"]:
                response = input(f"User '{user_id}' already exists. Overwrite? (y/N): ")
                if response.lower() != 'y':
                    print("Import cancelled")
                    return False

            # Import the user data
            self.database["users"][user_id] = user_data["user_data"]
            self.save_database()

            print(f"User '{user_id}' imported successfully from {import_path}")
            print(f"Original export date: {user_data.get('exported_at', 'Unknown')}")
            return True

        except Exception as e:
            print(f"Error importing user: {e}")
            return False

    def list_users(self) -> None:
        """List all users in the database"""
        if not self.database["users"]:
            print("No users found in database")
            return

        print(f"Users in database ({len(self.database['users'])} total):")
        for user_id, user_data in self.database["users"].items():
            num_encodings = len(user_data.get("face_encodings", []))
            created = user_data.get("created_at", "Unknown")
            print(f"  - {user_id}: {num_encodings} face samples (created: {created})")

def main():
    parser = argparse.ArgumentParser(description="Simple Face Authentication")
    parser.add_argument("--mode", choices=["register", "auth", "export", "import", "list"], required=True)
    parser.add_argument("--user", type=str, default="user")
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--tolerance", type=float, default=0.6)
    parser.add_argument("--file", type=str, help="File path for export/import operations")

    args = parser.parse_args()

    face_auth = SimpleFaceAuth()

    if args.mode == "register":
        success = face_auth.register_user(args.user, args.samples)
        sys.exit(0 if success else 1)
    elif args.mode == "auth":
        success = face_auth.authenticate_user(args.tolerance)
        sys.exit(0 if success else 1)
    elif args.mode == "export":
        success = face_auth.export_user(args.user, args.file)
        sys.exit(0 if success else 1)
    elif args.mode == "import":
        if not args.file:
            print("Error: --file required for import mode")
            sys.exit(1)
        success = face_auth.import_user(args.file)
        sys.exit(0 if success else 1)
    elif args.mode == "list":
        face_auth.list_users()
        sys.exit(0)

if __name__ == "__main__":
    main()