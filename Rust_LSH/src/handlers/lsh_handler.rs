use actix_web::{web, HttpResponse};
use crate::{errors::ApiError, services::lsh_service::LSHService, services::redis_service::RedisService};
use sha2::{Sha256, Digest};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    pub n_results: usize,
}

pub async fn add(
    lsh: web::Data<LSHService>,
    vectors: web::Json<Vec<Vec<f32>>>,
) -> Result<HttpResponse, ApiError> {
    let vecs = vectors.into_inner();
    lsh.add(&vecs)
        .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;
    Ok(HttpResponse::Ok().json("Vectors added"))
}


pub async fn query(
    lsh: web::Data<LSHService>,
    redis_service: web::Data<RedisService>,
    query: web::Json<Vec<f32>>,
    params: web::Query<QueryParams>,
) -> Result<HttpResponse, ApiError> {


    // Here we will recieve post_ids , fetch the vector from redis and then query the LSH 

    let query = query.into_inner();
    // Check if the query vector is empty
    if query.is_empty() {
        return Err(ApiError::BadRequest("Query vector is empty".to_string()));
    }

    let vec_str = redis_service
        .get(&format!("embedding:post:{}", query[0]))
        .await
        .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;



    let embedding: Vec<f32> = serde_json::from_str(&vec_str)
    .map_err(|e| ApiError::InternalServerError(format!("Failed to parse embedding: {}", e)))?;

// Convert to slice (&[f32])
    let vec: &[f32] = &embedding;


    let results = lsh
        .query(&vec, params.n_results)
        .map_err(|e| ApiError::InternalServerError(format!("{}",e)))?;


    //Hash resulting vectors 
    let mut hashed_results = Vec::new();
    for vector in &results {
        let hash = hash_vector(vector);
        hashed_results.push(hash);
    }


    let mut post_ids = Vec::new();
    for hash in hashed_results {
        let post_id = redis_service
            .get(&format!("post_from_embedding:{}", hash))
            .await
            .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;
        post_ids.push(post_id);
    }


    let mut posts = Vec::new();
    let query_post = redis_service
        .get(&format!("post:{}", query[0]))
        .await
        .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;
    posts.push(query_post);
    for post_id in post_ids.iter() {
        let post = redis_service
            .get(&format!("post:{}", post_id))
            .await
            .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;
        posts.push(post);
    }


    Ok(HttpResponse::Ok().json(posts))
}

fn hash_vector(vector: &[f32]) -> String {
    let mut hasher = Sha256::new();

    for &val in vector {
        hasher.update(&val.to_le_bytes()); // little-endian to match Python
    }

    let hash = hasher.finalize();
    format!("{:x}", hash)
}
