// src/services/lsh_service.rs

use crate::errors::ApiError;
use lsh_rs::prelude::{LshMem, SignRandomProjections};

pub struct LSHService {
    lsh: LshMem<SignRandomProjections<f32>, f32>,
}

impl LSHService {
    pub fn new(n_projections: usize, n_hash_tables: usize, dim: usize) -> Self {
        let mut lsh = LshMem::new(n_projections, n_hash_tables, dim);
        let lsh = lsh.seed(31).srp().unwrap();
        Self { lsh }
    }

    pub fn add(&mut self, vectors: &[Vec<f32>]) -> std::result::Result<(), ApiError> {
        self.lsh
            .store_vecs(vectors)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to add vectors: {}", e)))?;
        Ok(())
    }

    pub fn query(&self, query: &[f32], n_results: usize) -> std::result::Result<Vec<usize>, ApiError> {
        let ids_u32 = self.lsh
            .query_bucket_ids(query)
            .map_err(|e| ApiError::InternalServerError(format!("Query failed: {}", e)))?;
        Ok(ids_u32.into_iter()
                   .map(|i| i as usize)
                   .take(n_results)
                   .collect())
    }
}
