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
        """Register user with multiple face samples"""
        print(f"Starting registration for user: {user_id}")
        print(f"Will capture {num_samples} samples")

        os.makedirs("captured_images", exist_ok=True)
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
        print(f"Registration complete! {len(face_encodings)} samples stored for {user_id}")
        return True

    def authenticate_user(self, tolerance: float = 0.6) -> bool:
        """Authenticate user"""
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

        # Compare against registered users
        if not self.database.get("users"):
            print("No users registered")
            return False

        print(f"Comparing against {len(self.database['users'])} registered users...")

        best_match = None
        best_distance = float('inf')

        for user_id, user_data in self.database["users"].items():
            user_encodings = [np.array(sample["encoding"]) for sample in user_data["face_encodings"]]
            distances = face_recognition.face_distance(user_encodings, auth_encoding)
            min_distance = np.min(distances)

            print(f"User {user_id}: distance = {min_distance:.3f}")

            if min_distance < best_distance:
                best_distance = min_distance
                best_match = user_id

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

def main():
    parser = argparse.ArgumentParser(description="Simple Face Authentication")
    parser.add_argument("--mode", choices=["register", "auth"], required=True)
    parser.add_argument("--user", type=str, default="user")
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--tolerance", type=float, default=0.6)

    args = parser.parse_args()

    face_auth = SimpleFaceAuth()

    if args.mode == "register":
        success = face_auth.register_user(args.user, args.samples)
        sys.exit(0 if success else 1)
    elif args.mode == "auth":
        success = face_auth.authenticate_user(args.tolerance)
        sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()