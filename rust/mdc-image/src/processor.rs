//! Image processing operations for movie covers
//!
//! This module provides image processing functionality including:
//! - Load/save images
//! - Resize and crop operations
//! - Watermark overlay
//! - Smart cropping with face detection
//! - Three crop modes: copy, smart, small

use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use std::path::Path;

/// Crop mode matching Python's imagecut values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropMode {
    /// Mode 0: Copy image without cropping
    Copy = 0,
    /// Mode 1: Smart crop using face detection
    Smart = 1,
    /// Mode 3: Small crop (specific mode, details TBD)
    Small = 3,
    /// Mode 4: Smart crop for censored content
    SmartCensored = 4,
}

impl From<i32> for CropMode {
    fn from(value: i32) -> Self {
        match value {
            0 => CropMode::Copy,
            1 => CropMode::Smart,
            3 => CropMode::Small,
            4 => CropMode::SmartCensored,
            _ => CropMode::Copy,
        }
    }
}

/// Face location detected in image
#[derive(Debug, Clone, Copy)]
pub struct FaceLocation {
    /// X coordinate of face center
    pub center_x: u32,
    /// Y coordinate of top of face
    pub top_y: u32,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Image processor configuration
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Aspect ratio for poster (default: 2/3 = 0.6667)
    pub aspect_ratio: f32,

    /// Always use smart crop even if mode is Copy
    pub force_smart_crop: bool,

    /// Skip download if poster already exists
    pub skip_existing: bool,

    /// Skip face recognition (fall back to right-aligned crop)
    pub skip_face_detection: bool,

    /// Debug mode
    pub debug: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: 2.0 / 3.0,
            force_smart_crop: false,
            skip_existing: false,
            skip_face_detection: false,
            debug: false,
        }
    }
}

/// Image processor
pub struct ImageProcessor {
    pub config: ProcessorConfig,
}

impl ImageProcessor {
    /// Create a new image processor
    pub fn new(config: ProcessorConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self {
            config: ProcessorConfig::default(),
        }
    }

    /// Load an image from file
    pub fn load_image<P: AsRef<Path>>(&self, path: P) -> Result<DynamicImage> {
        let img = image::open(path.as_ref())?;
        Ok(img)
    }

    /// Save an image to file
    pub fn save_image<P: AsRef<Path>>(&self, img: &DynamicImage, path: P) -> Result<()> {
        img.save(path.as_ref())?;
        Ok(())
    }

    /// Cut/crop image from fanart to poster
    ///
    /// # Arguments
    /// * `fanart_path` - Path to input fanart image
    /// * `poster_path` - Path to output poster image
    /// * `crop_mode` - Crop mode to use
    /// * `face_location` - Optional face location for smart cropping
    pub fn cut_image<P: AsRef<Path>>(
        &self,
        fanart_path: P,
        poster_path: P,
        crop_mode: CropMode,
        face_location: Option<FaceLocation>,
    ) -> Result<()> {
        // Check if we should skip (poster already exists)
        if self.config.skip_existing && poster_path.as_ref().exists() {
            if let Ok(metadata) = std::fs::metadata(poster_path.as_ref()) {
                if metadata.len() > 0 {
                    return Ok(());
                }
            }
        }

        // Determine effective crop mode
        let effective_mode = if self.config.force_smart_crop {
            CropMode::Smart
        } else {
            crop_mode
        };

        match effective_mode {
            CropMode::Copy => {
                // Simply copy the file
                std::fs::copy(fanart_path.as_ref(), poster_path.as_ref())?;
                if self.config.debug {
                    tracing::info!("Image copied: {:?}", poster_path.as_ref());
                }
                Ok(())
            }
            CropMode::Smart | CropMode::SmartCensored => {
                self.smart_crop(fanart_path, poster_path, face_location)
            }
            CropMode::Small => {
                // Small mode: TBD, for now same as smart
                self.smart_crop(fanart_path, poster_path, face_location)
            }
        }
    }

    /// Smart crop with face detection
    fn smart_crop<P: AsRef<Path>>(
        &self,
        fanart_path: P,
        poster_path: P,
        face_location: Option<FaceLocation>,
    ) -> Result<()> {
        let img = self.load_image(&fanart_path)?;
        let (width, height) = img.dimensions();

        let cropped = if (width as f32 / height as f32) > self.config.aspect_ratio {
            // Width too large - crop width (portrait orientation)
            self.crop_width(&img, face_location)
        } else if (width as f32 / height as f32) < self.config.aspect_ratio {
            // Height too large - crop height (landscape orientation)
            self.crop_height(&img, face_location)
        } else {
            // Already correct aspect ratio
            img
        };

        self.save_image(&cropped, poster_path.as_ref())?;

        if self.config.debug {
            tracing::info!("Image cropped: {:?}", poster_path.as_ref());
        }

        Ok(())
    }

    /// Crop width (portrait mode) - center on face if detected
    fn crop_width(&self, img: &DynamicImage, face_location: Option<FaceLocation>) -> DynamicImage {
        let (width, height) = img.dimensions();
        let crop_width_half = height / 3; // New width is 2/3 of height
        let target_width = (crop_width_half as f32 * self.config.aspect_ratio) as u32;

        let (crop_left, crop_right) = if let Some(face) = face_location {
            // Center on face
            let center = face.center_x;
            let mut left = center.saturating_sub(crop_width_half);
            let mut right = center + crop_width_half;

            // Handle boundaries
            if left == 0 {
                right = target_width;
            } else if right > width {
                left = width.saturating_sub(target_width);
                right = width;
            }

            (left, right)
        } else {
            // Default: align to right
            let left = width.saturating_sub(target_width);
            (left, width)
        };

        img.crop_imm(crop_left, 0, crop_right - crop_left, height)
    }

    /// Crop height (landscape mode) - position face at top
    fn crop_height(&self, img: &DynamicImage, face_location: Option<FaceLocation>) -> DynamicImage {
        let (width, height) = img.dimensions();
        let crop_height = (width as f32 * 3.0 / 2.0) as u32;

        let (crop_top, crop_bottom) = if let Some(face) = face_location {
            // Position face near top
            let top = face.top_y;
            let bottom = top + crop_height;

            if bottom > height {
                // If overflow, start from top
                (0, crop_height)
            } else {
                (top, bottom)
            }
        } else {
            // Default: crop from top
            (0, crop_height.min(height))
        };

        img.crop_imm(0, crop_top, width, crop_bottom - crop_top)
    }

    /// Resize image to specific dimensions
    pub fn resize(&self, img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
    }

    /// Resize image maintaining aspect ratio (fit within bounds)
    pub fn resize_fit(&self, img: &DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
        img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3)
    }

    /// Add watermark overlay to image
    ///
    /// # Arguments
    /// * `base` - Base image to add watermark to
    /// * `watermark_path` - Path to watermark PNG (with transparency)
    /// * `_position` - Position as (x, y) tuple
    pub fn add_watermark<P: AsRef<Path>>(
        &self,
        base: &DynamicImage,
        watermark_path: P,
        _position: (u32, u32),
    ) -> Result<DynamicImage> {
        let watermark = self.load_image(watermark_path)?;
        let mut result = base.to_rgba8();

        // Overlay watermark with alpha blending
        image::imageops::overlay(
            &mut result,
            &watermark.to_rgba8(),
            _position.0 as i64,
            _position.1 as i64,
        );

        Ok(DynamicImage::ImageRgba8(result))
    }

    /// Add text watermark
    ///
    /// Note: This is a placeholder. Full text rendering would require imageproc + rusttype
    pub fn add_text_watermark(
        &self,
        base: &DynamicImage,
        _text: &str,
        _position: (u32, u32),
    ) -> Result<DynamicImage> {
        // TODO: Implement text rendering with rusttype
        // For now, just return the original image
        if self.config.debug {
            tracing::warn!("Text watermark not yet implemented: {}", _text);
        }
        Ok(base.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crop_mode_conversion() {
        assert_eq!(CropMode::from(0), CropMode::Copy);
        assert_eq!(CropMode::from(1), CropMode::Smart);
        assert_eq!(CropMode::from(3), CropMode::Small);
        assert_eq!(CropMode::from(4), CropMode::SmartCensored);
        assert_eq!(CropMode::from(99), CropMode::Copy); // Invalid defaults to Copy
    }

    #[test]
    fn test_processor_creation() {
        let processor = ImageProcessor::default();
        assert_eq!(processor.config.aspect_ratio, 2.0 / 3.0);
    }

    #[test]
    fn test_custom_config() {
        let config = ProcessorConfig {
            aspect_ratio: 0.75,
            force_smart_crop: true,
            skip_existing: true,
            skip_face_detection: true,
            debug: true,
        };

        let processor = ImageProcessor::new(config);
        assert_eq!(processor.config.aspect_ratio, 0.75);
        assert!(processor.config.force_smart_crop);
    }
}
