//! Face detection via Python face_recognition library
//!
//! This module provides face detection by calling a Python script
//! that uses the face_recognition library (dlib-based).

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::FaceLocation;

/// Face detection model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceModel {
    /// HOG (Histogram of Oriented Gradients) - faster, less accurate
    Hog,
    /// CNN (Convolutional Neural Network) - slower, more accurate
    Cnn,
}

impl FaceModel {
    fn as_str(&self) -> &str {
        match self {
            FaceModel::Hog => "hog",
            FaceModel::Cnn => "cnn",
        }
    }
}

/// Response from face detection script
#[derive(Debug, Deserialize, Serialize)]
struct FaceDetectionResponse {
    #[serde(default)]
    faces: Vec<FaceData>,
    #[serde(default)]
    count: usize,
    #[serde(default)]
    model: String,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FaceData {
    center_x: u32,
    top_y: u32,
    confidence: f32,
}

/// Face detector using Python subprocess
pub struct FaceDetector {
    /// Path to Python executable
    python_path: String,

    /// Path to face_detect_bridge.py
    bridge_script: String,

    /// Face detection model
    model: FaceModel,

    /// Upsample count
    upsample: u32,

    /// Debug mode
    debug: bool,
}

impl FaceDetector {
    /// Create a new face detector
    pub fn new() -> Self {
        Self {
            python_path: "python3".to_string(),
            bridge_script: Self::find_bridge_script(),
            model: FaceModel::Hog,
            upsample: 1,
            debug: false,
        }
    }

    /// Find the face_detect_bridge.py script
    fn find_bridge_script() -> String {
        let candidates = vec![
            "./face_detect_bridge.py",
            "./mdc-image/face_detect_bridge.py",
            "../mdc-image/face_detect_bridge.py",
            "/usr/local/share/mdc/face_detect_bridge.py",
        ];

        for path in candidates {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        "./face_detect_bridge.py".to_string()
    }

    /// Set custom Python path
    pub fn python_path(mut self, path: &str) -> Self {
        self.python_path = path.to_string();
        self
    }

    /// Set custom bridge script path
    pub fn bridge_script(mut self, path: &str) -> Self {
        self.bridge_script = path.to_string();
        self
    }

    /// Set face detection model
    pub fn model(mut self, model: FaceModel) -> Self {
        self.model = model;
        self
    }

    /// Set upsample count
    pub fn upsample(mut self, count: u32) -> Self {
        self.upsample = count;
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Detect faces in an image
    pub fn detect_faces<P: AsRef<Path>>(&self, image_path: P) -> Result<Option<FaceLocation>> {
        let path_str = image_path.as_ref().to_string_lossy();

        // Build command arguments
        let args = vec![
            self.bridge_script.clone(),
            path_str.to_string(),
            "--model".to_string(),
            self.model.as_str().to_string(),
            "--upsample".to_string(),
            self.upsample.to_string(),
        ];

        if self.debug {
            tracing::debug!("Face detection command: {} {}", self.python_path, args.join(" "));
        }

        // Execute Python script
        let output = Command::new(&self.python_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if self.debug {
            tracing::debug!("Face detection output: {}", stdout);
        }

        // Parse JSON response
        let response: FaceDetectionResponse = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse face detection response: {}", e))?;

        if let Some(error) = response.error {
            return Err(anyhow!("Face detection error: {}", error));
        }

        if response.faces.is_empty() {
            return Ok(None);
        }

        // Return the first (rightmost) face
        let face = &response.faces[0];
        Ok(Some(FaceLocation {
            center_x: face.center_x,
            top_y: face.top_y,
            confidence: face.confidence,
        }))
    }

    /// Check if face detection is available
    pub fn is_available(&self) -> bool {
        // Check if Python is available
        let python_check = Command::new(&self.python_path)
            .arg("--version")
            .output();

        if python_check.is_err() {
            return false;
        }

        // Check if bridge script exists
        if !std::path::Path::new(&self.bridge_script).exists() {
            return false;
        }

        // Check if face_recognition module is installed
        let module_check = Command::new(&self.python_path)
            .args(&["-c", "import face_recognition"])
            .output();

        module_check.is_ok() && module_check.unwrap().status.success()
    }
}

impl Default for FaceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_detector_creation() {
        let detector = FaceDetector::new();
        assert_eq!(detector.model, FaceModel::Hog);
        assert_eq!(detector.upsample, 1);
    }

    #[test]
    fn test_builder() {
        let detector = FaceDetector::new()
            .python_path("python3")
            .model(FaceModel::Cnn)
            .upsample(2)
            .debug(true);

        assert_eq!(detector.python_path, "python3");
        assert_eq!(detector.model, FaceModel::Cnn);
        assert_eq!(detector.upsample, 2);
        assert!(detector.debug);
    }

    #[test]
    fn test_availability_check() {
        let detector = FaceDetector::new();
        let available = detector.is_available();

        // This test just verifies the availability check works
        println!("Face detection available: {}", available);
    }
}
