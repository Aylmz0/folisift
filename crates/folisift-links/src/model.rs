use std::path::Path;

use model2vec_rs::model::StaticModel;

use crate::link::{Embedder, Result};

pub struct Model2VecEmbedder {
    model: StaticModel,
}

impl Model2VecEmbedder {
    pub fn load(path: &Path) -> Result<Self> {
        let model = StaticModel::from_pretrained(path, None, None, None)
            .map_err(|error| crate::link::Error::Model(error.to_string()))?;
        Ok(Self { model })
    }
}

impl Embedder for Model2VecEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(self.model.encode(texts))
    }
}
