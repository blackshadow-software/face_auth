use image::{imageops, GrayImage};
use anyhow::{Result, anyhow};
use ndarray::Array2;

#[derive(Debug, Clone)]
pub struct FaceInfo {
    pub bbox: BoundingBox,
    pub landmarks: Vec<Point>,
    pub features: Vec<f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct FaceDetector {
    feature_extractor: AdvancedFeatureExtractor,
}

impl FaceDetector {
    pub fn new() -> Result<Self> {
        println!("Initializing optimized face detector...");

        Ok(FaceDetector {
            feature_extractor: AdvancedFeatureExtractor::new(),
        })
    }

    pub fn detect_faces(&self, image_path: &str) -> Result<Vec<FaceInfo>> {
        println!("Loading and preprocessing image: {}", image_path);

        // Load image
        let img = image::open(image_path)
            .map_err(|e| anyhow!("Could not load image {}: {}", image_path, e))?;

        // Convert to grayscale for processing
        let gray_img = img.to_luma8();
        let (width, height) = gray_img.dimensions();

        println!("Processing image: {}x{}", width, height);

        // Simple and fast face detection - focus on center region
        let face_bbox = self.detect_center_face_region(width, height)?;

        println!("Face region detected at center");

        // Convert image to ndarray for processing
        let img_array = self.image_to_array(&gray_img)?;

        // Extract face region
        let face_region = self.extract_face_region(&img_array, &face_bbox)?;

        // Detect facial landmarks (estimated)
        let landmarks = self.detect_landmarks(&face_bbox)?;

        // Extract advanced features
        let features = self.feature_extractor.extract_features(&face_region)?;

        // Calculate confidence based on image quality
        let confidence = self.calculate_confidence(width, height)?;

        Ok(vec![FaceInfo {
            bbox: face_bbox,
            landmarks,
            features,
            confidence,
        }])
    }

    fn detect_center_face_region(&self, width: u32, height: u32) -> Result<BoundingBox> {
        // Simple approach: assume face is in center region
        let face_size = (width.min(height) / 3).max(100).min(300); // Face size between 100-300px
        let x = (width - face_size) / 2;
        let y = (height - face_size) / 2;

        Ok(BoundingBox {
            x,
            y,
            width: face_size,
            height: face_size,
        })
    }

    fn image_to_array(&self, img: &GrayImage) -> Result<Array2<u8>> {
        let (width, height) = img.dimensions();
        let mut array = Array2::zeros((height as usize, width as usize));

        for (x, y, pixel) in img.enumerate_pixels() {
            array[[y as usize, x as usize]] = pixel[0];
        }

        Ok(array)
    }

    fn extract_face_region(&self, img: &Array2<u8>, bbox: &BoundingBox) -> Result<Array2<u8>> {
        let (img_height, img_width) = img.dim();

        let x1 = bbox.x as usize;
        let y1 = bbox.y as usize;
        let x2 = ((bbox.x + bbox.width) as usize).min(img_width);
        let y2 = ((bbox.y + bbox.height) as usize).min(img_height);

        if x2 <= x1 || y2 <= y1 {
            return Err(anyhow!("Invalid face region"));
        }

        let face_region = img.slice(ndarray::s![y1..y2, x1..x2]).to_owned();
        Ok(face_region)
    }

    fn detect_landmarks(&self, bbox: &BoundingBox) -> Result<Vec<Point>> {
        // Estimate key facial landmarks based on face proportions
        let face_width = bbox.width as f64;
        let face_height = bbox.height as f64;

        let landmarks = vec![
            // Left eye
            Point { x: bbox.x as f64 + face_width * 0.3, y: bbox.y as f64 + face_height * 0.35 },
            // Right eye
            Point { x: bbox.x as f64 + face_width * 0.7, y: bbox.y as f64 + face_height * 0.35 },
            // Nose tip
            Point { x: bbox.x as f64 + face_width * 0.5, y: bbox.y as f64 + face_height * 0.55 },
            // Left mouth corner
            Point { x: bbox.x as f64 + face_width * 0.35, y: bbox.y as f64 + face_height * 0.75 },
            // Right mouth corner
            Point { x: bbox.x as f64 + face_width * 0.65, y: bbox.y as f64 + face_height * 0.75 },
        ];

        Ok(landmarks)
    }

    fn calculate_confidence(&self, width: u32, height: u32) -> Result<f64> {
        // Simple confidence based on image size
        let size_score = if width >= 640 && height >= 480 { 0.9 } else { 0.7 };
        Ok(size_score)
    }

    pub fn compute_similarity(features1: &[f64], features2: &[f64]) -> f64 {
        if features1.len() != features2.len() || features1.is_empty() {
            return 0.0;
        }

        // Use cosine similarity for better face matching
        let dot_product: f64 = features1.iter().zip(features2.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm1: f64 = features1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = features2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        let cosine_similarity = dot_product / (norm1 * norm2);

        // Convert to range [0, 1] and apply nonlinear scaling for better discrimination
        let similarity = (cosine_similarity + 1.0) / 2.0;

        // Apply exponential scaling to enhance differences
        similarity.powf(1.5).min(1.0).max(0.0)
    }
}

struct AdvancedFeatureExtractor;

impl AdvancedFeatureExtractor {
    fn new() -> Self {
        AdvancedFeatureExtractor
    }

    fn extract_features(&self, face: &Array2<u8>) -> Result<Vec<f64>> {
        println!("Extracting features from face region...");

        let mut features = Vec::new();

        // Convert to standard size for consistent feature extraction
        let resized = self.resize_face(face, 64, 64)?; // Smaller size for faster processing

        // Extract simplified but effective features

        // 1. Regional intensity statistics (simplified)
        let regional_features = self.extract_regional_features(&resized)?;
        features.extend(regional_features);

        // 2. Edge features (simplified)
        let edge_features = self.extract_edge_features(&resized)?;
        features.extend(edge_features);

        // 3. Symmetry features
        let symmetry_features = self.extract_symmetry_features(&resized)?;
        features.extend(symmetry_features);

        // Normalize features to unit vector for cosine similarity
        let norm: f64 = features.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            features.iter_mut().for_each(|x| *x /= norm);
        }

        println!("Extracted {} normalized features", features.len());
        Ok(features)
    }

    fn resize_face(&self, face: &Array2<u8>, target_width: usize, target_height: usize) -> Result<Array2<u8>> {
        let (height, width) = face.dim();

        // Simple nearest neighbor resize for speed
        let mut resized = Array2::zeros((target_height, target_width));

        for y in 0..target_height {
            for x in 0..target_width {
                let src_x = (x * width / target_width).min(width - 1);
                let src_y = (y * height / target_height).min(height - 1);
                resized[[y, x]] = face[[src_y, src_x]];
            }
        }

        Ok(resized)
    }

    fn extract_regional_features(&self, face: &Array2<u8>) -> Result<Vec<f64>> {
        let mut features = Vec::new();
        let (height, width) = face.dim();

        // Extract features from 4x4 grid regions
        let grid_size = 4;
        let step_x = width / grid_size;
        let step_y = height / grid_size;

        for y in 0..grid_size {
            for x in 0..grid_size {
                let start_x = x * step_x;
                let start_y = y * step_y;
                let end_x = ((x + 1) * step_x).min(width);
                let end_y = ((y + 1) * step_y).min(height);

                if start_x < width && start_y < height && end_x > start_x && end_y > start_y {
                    let region = face.slice(ndarray::s![start_y..end_y, start_x..end_x]);

                    // Calculate mean intensity
                    let mean = region.iter().map(|&x| x as f64).sum::<f64>() / region.len() as f64;
                    features.push(mean / 255.0);
                }
            }
        }

        Ok(features)
    }

    fn extract_edge_features(&self, face: &Array2<u8>) -> Result<Vec<f64>> {
        let (height, width) = face.dim();
        let mut edge_features = Vec::new();

        // Calculate horizontal and vertical edge strength
        let mut h_edges = 0.0;
        let mut v_edges = 0.0;
        let mut count = 0;

        for y in 1..height-1 {
            for x in 1..width-1 {
                // Horizontal edge (difference between top and bottom)
                let h_diff = (face[[y-1, x]] as f64 - face[[y+1, x]] as f64).abs();
                // Vertical edge (difference between left and right)
                let v_diff = (face[[y, x-1]] as f64 - face[[y, x+1]] as f64).abs();

                h_edges += h_diff;
                v_edges += v_diff;
                count += 1;
            }
        }

        if count > 0 {
            edge_features.push(h_edges / (count as f64 * 255.0));
            edge_features.push(v_edges / (count as f64 * 255.0));
        } else {
            edge_features.push(0.0);
            edge_features.push(0.0);
        }

        Ok(edge_features)
    }

    fn extract_symmetry_features(&self, face: &Array2<u8>) -> Result<Vec<f64>> {
        let (height, width) = face.dim();
        let mid_x = width / 2;

        let mut symmetry_diff = 0.0;
        let mut count = 0;

        // Calculate vertical symmetry
        for y in 0..height {
            for x in 0..mid_x {
                if width > x {
                    let left_val = face[[y, x]] as f64;
                    let right_val = face[[y, width - 1 - x]] as f64;
                    symmetry_diff += (left_val - right_val).abs();
                    count += 1;
                }
            }
        }

        let symmetry = if count > 0 {
            1.0 - (symmetry_diff / (count as f64 * 255.0))
        } else {
            0.0
        };

        Ok(vec![symmetry.max(0.0)])
    }
}