use crate::errors::ApiError;
use lsh_rs::prelude::{LshMem, SignRandomProjections};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LSHService {
    lsh: Arc<Mutex<LshMem<SignRandomProjections<f32>, f32>>>,
}

impl LSHService {
    pub fn new(n_projections: usize, n_hash_tables: usize, dim: usize) -> Self {
        let lsh = LshMem::new(n_projections, n_hash_tables, dim)
            .srp()
            .unwrap();
        Self {
            lsh: Arc::new(Mutex::new(lsh)),
        }
    }

    pub fn add(&self, vectors: &[Vec<f32>]) -> Result<(), ApiError> {
        let mut locked = self
            .lsh
            .lock()
            .map_err(|_| ApiError::InternalServerError("LSH mutex poisoned".to_string()))?;
        locked
            .store_vecs(vectors)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to add vectors: {}", e)))?;
        Ok(())
    }

    pub fn query(&self, query: &[f32], n_results: usize) -> Result<Vec<Vec<f32>>, ApiError> {
    let locked = self
        .lsh
        .lock()
        .map_err(|_| ApiError::InternalServerError("LSH mutex poisoned".to_string()))?;
    let ids_u32 = locked
        .query_bucket(query)
        .map_err(|e| ApiError::InternalServerError(format!("Query failed: {}", e)))?;

    // Convert Vec<&Vec<f32>> into Vec<Vec<f32>> by cloning each inner vector
    Ok(ids_u32
        .into_iter()
        .take(n_results)
        .map(|v| v.clone())  // Clone each inner Vec<f32>
        .collect())
}
}
