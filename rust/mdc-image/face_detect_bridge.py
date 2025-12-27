#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Face detection bridge using face_recognition library

This script is called from Rust to detect faces in images.
It uses the face_recognition library (dlib-based).

Usage:
    python face_detect_bridge.py <image_path> [options]

Options:
    --model hog|cnn      Face detection model (default: hog)
    --upsample N         Number of times to upsample (default: 1)

Output:
    Prints JSON with face locations to stdout
    Format: {"faces": [{"center_x": 100, "top_y": 50, "confidence": 0.95}]}
    Exit code 0 on success, non-zero on failure
"""

import sys
import argparse
import json

try:
    import face_recognition
except ImportError:
    print(json.dumps({"error": "face_recognition not installed"}), file=sys.stdout)
    sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description='Face detection bridge')
    parser.add_argument('image_path', help='Path to image file')
    parser.add_argument('--model', default='hog', choices=['hog', 'cnn'],
                        help='Face detection model')
    parser.add_argument('--upsample', type=int, default=1,
                        help='Number of times to upsample image')

    args = parser.parse_args()

    try:
        # Load image
        image = face_recognition.load_image_file(args.image_path)

        # Detect faces
        face_locations = face_recognition.face_locations(
            image,
            number_of_times_to_upsample=args.upsample,
            model=args.model
        )

        if not face_locations:
            # No faces found
            result = {"faces": []}
            print(json.dumps(result))
            return 0

        # Find rightmost face (matching Python logic)
        max_right_x = 0
        max_top_y = 0

        for face_location in face_locations:
            top, right, bottom, left = face_location

            # Calculate center
            center_x = int((right + left) / 2)

            # Keep track of rightmost face
            if center_x > max_right_x:
                max_right_x = center_x
                max_top_y = top

        # Output result
        result = {
            "faces": [{
                "center_x": max_right_x,
                "top_y": max_top_y,
                "confidence": 1.0  # face_recognition doesn't provide confidence
            }],
            "count": len(face_locations),
            "model": args.model
        }

        print(json.dumps(result))
        return 0

    except FileNotFoundError:
        print(json.dumps({"error": f"Image not found: {args.image_path}"}), file=sys.stdout)
        return 2
    except Exception as e:
        print(json.dumps({"error": str(e)}), file=sys.stdout)
        return 1


if __name__ == '__main__':
    sys.exit(main())
