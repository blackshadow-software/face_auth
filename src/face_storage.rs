use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredFace {
    pub user_id: String,
    pub features: Vec<f64>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FaceDatabase {
    pub faces: Vec<StoredFace>,
}

impl FaceDatabase {
    pub fn new() -> Self {
        FaceDatabase {
            faces: Vec::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let db_path = "face_database.json";
        if Path::new(db_path).exists() {
            let content = fs::read_to_string(db_path)?;
            let db: FaceDatabase = serde_json::from_str(&content)?;
            Ok(db)
        } else {
            Ok(FaceDatabase::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let db_path = "face_database.json";
        let content = serde_json::to_string_pretty(self)?;
        fs::write(db_path, content)?;
        Ok(())
    }

    pub fn add_face(&mut self, user_id: String, features: Vec<f64>) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let stored_face = StoredFace {
            user_id,
            features,
            timestamp,
        };

        self.faces.push(stored_face);
        self.save()?;
        Ok(())
    }

    pub fn get_all_faces(&self) -> &Vec<StoredFace> {
        &self.faces
    }

    pub fn clear(&mut self) -> Result<()> {
        self.faces.clear();
        self.save()?;
        Ok(())
    }
}