use std::path::Path;

use model2vec_rs::model::StaticModel;
use sha2::{Digest, Sha256};

use crate::link::{Embedder, Error, Result};

pub struct Model2VecEmbedder {
    model: StaticModel,
}

impl Model2VecEmbedder {
    pub fn load(path: &Path) -> Result<Self> {
        verify_model_dir(path)?;
        let model = StaticModel::from_pretrained(path, None, None, None)
            .map_err(|error| Error::Model(error.to_string()))?;
        Ok(Self { model })
    }
}

impl Embedder for Model2VecEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(self.model.encode(texts))
    }
}

/// Verify the release manifest hashes before loading model weights.
pub fn verify_model_dir(path: &Path) -> Result<()> {
    let manifest_path = path.join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).map_err(|error| {
            Error::Model(format!(
                "model manifest {}: {error}",
                manifest_path.display()
            ))
        })?)
        .map_err(|error| Error::Model(format!("model manifest JSON: {error}")))?;
    let files = manifest
        .get("files")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| Error::Model("model manifest has no files map".to_string()))?;
    for (name, entry) in files {
        let expected = entry
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| Error::Model(format!("model file {name} has no sha256")))?;
        let bytes = std::fs::read(path.join(name))
            .map_err(|error| Error::Model(format!("model file {name}: {error}")))?;
        let actual = format!("{:x}", Sha256::digest(bytes));
        if actual != expected {
            return Err(Error::Model(format!(
                "model file {name} hash mismatch: expected {expected}, got {actual}"
            )));
        }
    }
    Ok(())
}
