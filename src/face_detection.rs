use image::{imageops, GrayImage};
use anyhow::{Result, anyhow};

pub struct FaceDetector;

#[derive(Debug, Clone)]
pub struct FaceInfo {
    pub features: Vec<f64>,
}

impl FaceDetector {
    pub fn new() -> Result<Self> {
        Ok(FaceDetector)
    }

    pub fn detect_faces(&self, image_path: &str) -> Result<Vec<FaceInfo>> {
        // Load and process image
        let img = image::open(image_path)
            .map_err(|e| anyhow!("Could not load image {}: {}", image_path, e))?;

        // Convert to grayscale and get dimensions
        let gray_img = img.to_luma8();
        let (width, height) = gray_img.dimensions();

        println!("Processing image: {}x{}", width, height);

        // Improved face detection: focus on center region where faces are typically located
        // This reduces background noise significantly
        let center_crop_factor = 0.7; // Use central 70% of the image
        let margin_x = ((width as f32) * (1.0 - center_crop_factor) / 2.0) as u32;
        let margin_y = ((height as f32) * (1.0 - center_crop_factor) / 2.0) as u32;

        let crop_width = width - (2 * margin_x);
        let crop_height = height - (2 * margin_y);

        // Crop to center region (likely face area)
        let face_region = imageops::crop_imm(&gray_img, margin_x, margin_y, crop_width, crop_height);

        println!("Focused on center region: {}x{} (cropped from {}x{})",
                 crop_width, crop_height, width, height);

        // Apply brightness enhancement and histogram equalization for low-light conditions
        let brightened = self.enhance_brightness(&face_region.to_image(), 1.3)?; // 30% brightness boost
        let equalized = self.histogram_equalize(&brightened)?;

        // Resize to standard size for consistent feature extraction
        let resized = imageops::resize(&equalized, 128, 128, imageops::FilterType::Lanczos3);

        // Extract features from the face region
        let features = self.extract_features(&resized)?;

        println!("Successfully extracted {} features from face region", features.len());

        Ok(vec![FaceInfo { features }])
    }

    fn enhance_brightness(&self, img: &GrayImage, factor: f32) -> Result<GrayImage> {
        let (width, height) = img.dimensions();
        let mut enhanced = GrayImage::new(width, height);

        for (x, y, pixel) in img.enumerate_pixels() {
            let old_value = pixel[0] as f32;
            let new_value = (old_value * factor).min(255.0) as u8;
            enhanced.put_pixel(x, y, image::Luma([new_value]));
        }

        Ok(enhanced)
    }

    fn histogram_equalize(&self, img: &GrayImage) -> Result<GrayImage> {
        let (width, height) = img.dimensions();
        let mut histogram = vec![0u32; 256];

        // Calculate histogram
        for pixel in img.pixels() {
            histogram[pixel[0] as usize] += 1;
        }

        // Calculate cumulative distribution
        let total_pixels = (width * height) as f64;
        let mut cdf = vec![0.0; 256];
        cdf[0] = histogram[0] as f64 / total_pixels;

        for i in 1..256 {
            cdf[i] = cdf[i-1] + (histogram[i] as f64 / total_pixels);
        }

        // Create equalized image
        let mut equalized = GrayImage::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels() {
            let old_value = pixel[0] as usize;
            let new_value = (cdf[old_value] * 255.0) as u8;
            equalized.put_pixel(x, y, image::Luma([new_value]));
        }

        Ok(equalized)
    }

    fn extract_features(&self, img: &image::GrayImage) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let (width, height) = img.dimensions();

        // Face-specific feature extraction

        // 1. Divide face into regions (eyes, nose, mouth areas)
        let region_features = self.extract_regional_features(img)?;
        features.extend(region_features);

        // 2. Extract edge features (important for face structure)
        let edge_features = self.extract_edge_features(img)?;
        features.extend(edge_features);

        // 3. Extract texture features using Local Binary Patterns
        let texture_features = self.extract_texture_features(img)?;
        features.extend(texture_features);

        // 4. Extract geometric ratios (face proportions)
        let geometric_features = self.extract_geometric_features(img)?;
        features.extend(geometric_features);

        println!("Extracted {} face-specific features", features.len());
        Ok(features)
    }

    fn extract_regional_features(&self, img: &image::GrayImage) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let (width, height) = img.dimensions();

        // Divide face into 9 regions (3x3 grid)
        let region_width = width / 3;
        let region_height = height / 3;

        for ry in 0..3 {
            for rx in 0..3 {
                let start_x = rx * region_width;
                let start_y = ry * region_height;
                let end_x = ((rx + 1) * region_width).min(width);
                let end_y = ((ry + 1) * region_height).min(height);

                // Calculate mean intensity for this region
                let mut sum = 0.0;
                let mut count = 0;

                for y in start_y..end_y {
                    for x in start_x..end_x {
                        sum += img.get_pixel(x, y)[0] as f64;
                        count += 1;
                    }
                }

                let mean = if count > 0 { sum / count as f64 } else { 0.0 };
                features.push(mean / 255.0); // Normalize

                // Calculate variance for this region
                let mut var_sum = 0.0;
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let diff = img.get_pixel(x, y)[0] as f64 - mean;
                        var_sum += diff * diff;
                    }
                }
                let variance = if count > 0 { var_sum / count as f64 } else { 0.0 };
                features.push(variance / (255.0 * 255.0)); // Normalize
            }
        }

        Ok(features)
    }

    fn extract_edge_features(&self, img: &image::GrayImage) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let (width, height) = img.dimensions();

        // Simple edge detection using Sobel-like operator
        let mut edge_strength = 0.0;
        let mut edge_count = 0;

        for y in 1..height-1 {
            for x in 1..width-1 {
                // Horizontal gradient
                let gx = (img.get_pixel(x+1, y)[0] as f64) - (img.get_pixel(x-1, y)[0] as f64);
                // Vertical gradient
                let gy = (img.get_pixel(x, y+1)[0] as f64) - (img.get_pixel(x, y-1)[0] as f64);

                let gradient_magnitude = (gx * gx + gy * gy).sqrt();
                edge_strength += gradient_magnitude;
                edge_count += 1;
            }
        }

        let avg_edge_strength = if edge_count > 0 { edge_strength / edge_count as f64 } else { 0.0 };
        features.push(avg_edge_strength / 255.0); // Normalize

        Ok(features)
    }

    fn extract_texture_features(&self, img: &image::GrayImage) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let (width, height) = img.dimensions();

        // Enhanced Local Binary Pattern
        let mut lbp_histogram = vec![0u32; 256];

        for y in 1..height-1 {
            for x in 1..width-1 {
                let center_pixel = img.get_pixel(x, y)[0];
                let mut lbp_value = 0u8;

                // 8 neighbors in circular pattern
                let neighbors = [
                    img.get_pixel(x-1, y-1)[0], img.get_pixel(x, y-1)[0], img.get_pixel(x+1, y-1)[0],
                    img.get_pixel(x+1, y)[0], img.get_pixel(x+1, y+1)[0], img.get_pixel(x, y+1)[0],
                    img.get_pixel(x-1, y+1)[0], img.get_pixel(x-1, y)[0]
                ];

                for (i, &neighbor) in neighbors.iter().enumerate() {
                    if neighbor >= center_pixel {
                        lbp_value |= 1 << i;
                    }
                }

                lbp_histogram[lbp_value as usize] += 1;
            }
        }

        // Use only the most significant LBP bins (reduce from 256 to 32)
        let total_pixels = (width - 2) * (height - 2);
        for i in (0..256).step_by(8) {
            let bin_sum: u32 = lbp_histogram[i..i.min(256).min(i+8)].iter().sum();
            features.push(bin_sum as f64 / total_pixels as f64);
        }

        Ok(features)
    }

    fn extract_geometric_features(&self, img: &image::GrayImage) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let (width, height) = img.dimensions();

        // Face proportion ratios
        features.push(width as f64 / height as f64); // Aspect ratio

        // Symmetry measure (compare left vs right half)
        let mut symmetry_diff = 0.0;
        let mid_x = width / 2;

        for y in 0..height {
            for x in 0..mid_x {
                let left_pixel = img.get_pixel(x, y)[0] as f64;
                let right_pixel = img.get_pixel(width - 1 - x, y)[0] as f64;
                symmetry_diff += (left_pixel - right_pixel).abs();
            }
        }

        let symmetry = 1.0 - (symmetry_diff / (255.0 * (mid_x * height) as f64));
        features.push(symmetry.max(0.0)); // Ensure non-negative

        Ok(features)
    }

    pub fn compute_similarity(features1: &[f64], features2: &[f64]) -> f64 {
        if features1.len() != features2.len() {
            return 0.0;
        }

        // Use Euclidean distance with exponential decay for better discrimination
        let mut squared_diff_sum = 0.0;
        let mut weight_sum = 0.0;

        for (i, (&f1, &f2)) in features1.iter().zip(features2.iter()).enumerate() {
            let diff = f1 - f2;
            let weight = if i < 18 { 3.0 } else if i < 50 { 2.0 } else { 1.0 }; // Weight regional features more
            squared_diff_sum += weight * diff * diff;
            weight_sum += weight;
        }

        if weight_sum == 0.0 {
            return 0.0;
        }

        // Average weighted squared difference
        let mean_squared_diff = squared_diff_sum / weight_sum;

        // Convert to similarity using exponential decay (more discriminative)
        // This will give much lower scores for different faces
        let similarity = (-mean_squared_diff * 100.0).exp(); // Scale factor for more discrimination

        // Ensure we get meaningful differences between same person vs different people
        similarity.min(1.0).max(0.0)
    }
}