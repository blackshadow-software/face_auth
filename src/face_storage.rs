use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredFace {
    pub user_id: String,
    pub features: Vec<f64>,
    pub timestamp: String,
    pub confidence_during_registration: f64,
    pub sample_id: String, // Unique identifier for each face sample
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub face_samples: Vec<StoredFace>,
    pub enrollment_date: String,
    pub last_authentication: Option<String>,
    pub authentication_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FaceDatabase {
    pub users: HashMap<String, UserProfile>,
    pub version: String,
    pub accuracy_threshold: f64,
    pub min_samples_per_user: usize,
    pub max_samples_per_user: usize,
}

impl FaceDatabase {
    pub fn new() -> Self {
        FaceDatabase {
            users: HashMap::new(),
            version: "2.0".to_string(),
            accuracy_threshold: 0.85, // Higher threshold for better security
            min_samples_per_user: 3,   // Require multiple samples for robustness
            max_samples_per_user: 10,  // Limit storage and computation
        }
    }

    pub fn load() -> Result<Self> {
        let db_path = "face_database_v2.json";

        if Path::new(db_path).exists() {
            let content = fs::read_to_string(db_path)?;
            let db: FaceDatabase = serde_json::from_str(&content)?;
            Ok(db)
        } else {
            // Try to migrate from old database format
            let old_db_path = "face_database.json";
            if Path::new(old_db_path).exists() {
                println!("Migrating from old database format...");
                let mut new_db = FaceDatabase::new();

                let content = fs::read_to_string(old_db_path)?;
                let old_db: serde_json::Value = serde_json::from_str(&content)?;

                if let Some(faces) = old_db.get("faces").and_then(|f| f.as_array()) {
                    for face in faces {
                        if let (Some(user_id), Some(features), Some(_timestamp)) = (
                            face.get("user_id").and_then(|v| v.as_str()),
                            face.get("features").and_then(|v| v.as_array()),
                            face.get("timestamp").and_then(|v| v.as_str()),
                        ) {
                            let features_vec: Vec<f64> = features.iter()
                                .filter_map(|v| v.as_f64())
                                .collect();

                            new_db.add_face_sample(
                                user_id.to_string(),
                                features_vec,
                                0.9, // Default confidence for migrated data
                            )?;
                        }
                    }
                }

                new_db.save()?;
                println!("Migration completed successfully!");
                Ok(new_db)
            } else {
                Ok(FaceDatabase::new())
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let db_path = "face_database_v2.json";
        let content = serde_json::to_string_pretty(self)?;
        fs::write(db_path, content)?;
        Ok(())
    }

    pub fn add_face_sample(&mut self, user_id: String, features: Vec<f64>, confidence: f64) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let sample_id = format!("{}_{}", user_id, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

        let stored_face = StoredFace {
            user_id: user_id.clone(),
            features,
            timestamp: timestamp.clone(),
            confidence_during_registration: confidence,
            sample_id,
        };

        // Create profile first if needed
        if !self.users.contains_key(&user_id) {
            let profile = UserProfile {
                user_id: user_id.clone(),
                face_samples: Vec::new(),
                enrollment_date: timestamp.clone(),
                last_authentication: None,
                authentication_count: 0,
            };
            self.users.insert(user_id.clone(), profile);
        }

        // Add the new sample
        {
            let profile = self.users.get_mut(&user_id).unwrap();
            profile.face_samples.push(stored_face);

            // Limit the number of samples per user
            if profile.face_samples.len() > self.max_samples_per_user {
                // Remove oldest samples, but keep the best ones based on confidence
                profile.face_samples.sort_by(|a, b| {
                    b.confidence_during_registration.partial_cmp(&a.confidence_during_registration)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                profile.face_samples.truncate(self.max_samples_per_user);
            }
        }

        self.save()?;
        let sample_count = self.users.get(&user_id).map_or(0, |p| p.face_samples.len());
        println!("Added face sample for user '{}'. Total samples: {}",
                 user_id, sample_count);
        Ok(())
    }

    pub fn get_user_profile(&self, user_id: &str) -> Option<&UserProfile> {
        self.users.get(user_id)
    }

    pub fn get_all_users(&self) -> Vec<&UserProfile> {
        self.users.values().collect()
    }

    pub fn get_all_faces(&self) -> Vec<&StoredFace> {
        self.users.values()
            .flat_map(|profile| &profile.face_samples)
            .collect()
    }

    pub fn update_authentication_stats(&mut self, user_id: &str) -> Result<()> {
        if let Some(profile) = self.users.get_mut(user_id) {
            profile.last_authentication = Some(chrono::Utc::now().to_rfc3339());
            profile.authentication_count += 1;
            self.save()?;
        }
        Ok(())
    }

    pub fn is_user_enrolled(&self, user_id: &str) -> bool {
        if let Some(profile) = self.users.get(user_id) {
            profile.face_samples.len() >= self.min_samples_per_user
        } else {
            false
        }
    }

    pub fn get_enrollment_progress(&self, user_id: &str) -> (usize, usize) {
        if let Some(profile) = self.users.get(user_id) {
            (profile.face_samples.len(), self.min_samples_per_user)
        } else {
            (0, self.min_samples_per_user)
        }
    }

    /// Find best matching user using parallel processing for performance
    pub fn find_best_match(&self, features: &[f64]) -> Option<(String, f64)> {
        if self.users.is_empty() {
            return None;
        }

        // Parallel computation for better performance
        let best_match = self.users.par_iter()
            .filter_map(|(user_id, profile)| {
                if profile.face_samples.is_empty() {
                    return None;
                }

                // Calculate average similarity across all samples for this user
                let similarities: Vec<f64> = profile.face_samples.par_iter()
                    .map(|stored_face| self.compute_similarity(features, &stored_face.features))
                    .collect();

                if similarities.is_empty() {
                    return None;
                }

                // Use different strategies for aggregating similarities
                let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
                let max_similarity = similarities.iter().fold(0.0f64, |a, &b| a.max(b));
                let min_similarity = similarities.iter().fold(1.0f64, |a, &b| a.min(b));

                // Weighted combination: favor consistency (high minimum) and peak similarity
                let weighted_similarity = 0.4 * max_similarity + 0.4 * avg_similarity + 0.2 * min_similarity;

                Some((user_id.clone(), weighted_similarity))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        best_match
    }

    /// Enhanced similarity computation with cosine similarity
    fn compute_similarity(&self, features1: &[f64], features2: &[f64]) -> f64 {
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

    pub fn clear(&mut self) -> Result<()> {
        self.users.clear();
        self.save()?;
        Ok(())
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<bool> {
        let removed = self.users.remove(user_id).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn get_database_stats(&self) -> DatabaseStats {
        let total_users = self.users.len();
        let total_samples = self.users.values()
            .map(|profile| profile.face_samples.len())
            .sum();

        let enrolled_users = self.users.values()
            .filter(|profile| profile.face_samples.len() >= self.min_samples_per_user)
            .count();

        let total_authentications = self.users.values()
            .map(|profile| profile.authentication_count)
            .sum();

        DatabaseStats {
            total_users,
            enrolled_users,
            total_samples,
            total_authentications,
            accuracy_threshold: self.accuracy_threshold,
            min_samples_per_user: self.min_samples_per_user,
        }
    }

    pub fn optimize_database(&mut self) -> Result<usize> {
        let mut removed_samples = 0;

        for profile in self.users.values_mut() {
            if profile.face_samples.len() > self.max_samples_per_user {
                // Sort by confidence and keep only the best samples
                profile.face_samples.sort_by(|a, b| {
                    b.confidence_during_registration.partial_cmp(&a.confidence_during_registration)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let original_len = profile.face_samples.len();
                profile.face_samples.truncate(self.max_samples_per_user);
                removed_samples += original_len - profile.face_samples.len();
            }
        }

        if removed_samples > 0 {
            self.save()?;
        }

        Ok(removed_samples)
    }
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub total_users: usize,
    pub enrolled_users: usize,
    pub total_samples: usize,
    pub total_authentications: u32,
    pub accuracy_threshold: f64,
    pub min_samples_per_user: usize,
}