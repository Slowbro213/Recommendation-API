use actix_web::{web, HttpResponse};
use crate::{errors::ApiError, services::lsh_service::LSHService, services::redis_service::RedisService};
use sha2::{Sha256, Digest};
use serde::Deserialize;
use futures::future::try_join_all;
use std::collections::HashSet;

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
    query: web::Json<Vec<u32>>,
    params: web::Query<QueryParams>,
) -> Result<HttpResponse, ApiError> {


    let queries = query.into_inner();

    if queries.is_empty() {
        return Err(ApiError::BadRequest("Query vector is empty".to_string()));
    }



    // Prefetch all embeddings in parallel
    let embedding_futures = queries.iter().map(|post_id| {
        let key = format!("embedding:post:{}", post_id);
        let key_clone = key.clone();  // Clone the String
        let redis_service = redis_service.clone();  // Clone the RedisService
        async move {  // `async move` takes ownership of key_clone
            redis_service.get(&key_clone).await
                .map_err(|e| ApiError::InternalServerError(format!("Redis fetch error: {}", e)))
        }
    });

    let raw_embeddings: Vec<String> = try_join_all(embedding_futures)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Redis fetch error: {}", e)))?;

    // Query LSH for each vector
    let mut hashed_results = HashSet::new(); // HashSet to avoid duplicates

    for vec_str in raw_embeddings {
        let embedding: Vec<f32> = serde_json::from_str(&vec_str)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to parse embedding: {}", e)))?;

        let results = lsh
            .query(&embedding, params.n_results)
            .map_err(|e| ApiError::InternalServerError(format!("LSH query error: {}", e)))?;

        for vector in results {
            hashed_results.insert(hash_vector(&vector));
        }
    }

    // Fetch post_ids in parallel using hashes
    let post_id_futures = hashed_results.iter().map(|hash| {
        let key = format!("post_from_embedding:{}", hash);
        let redis_service = redis_service.clone();  // Clone the RedisService
        async move {  // `async move` takes ownership of key
            redis_service.get(&key)
                .await
                .map_err(|e| ApiError::InternalServerError(format!("Redis post_id fetch error: {}", e)))
        }
    });

    let raw_post_ids: Vec<String> = try_join_all(post_id_futures)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Redis post_id fetch error: {}", e)))?;

    // Deduplicate + skip self matches
    let queries_set: HashSet<u32> = queries.into_iter().collect();
    let mut post_ids = Vec::new();

    for id_str in raw_post_ids {
        let id: u32 = id_str
            .parse()
            .map_err(|e| ApiError::InternalServerError(format!("Failed to parse post_id: {}", e)))?;

        if !queries_set.contains(&id) && !post_ids.contains(&id) {
            post_ids.push(id);
        }
    }


    //let mut posts = Vec::new();
    //let query_post = redis_service
    //    .get(&format!("post:{}", query[0]))
    //    .await
    //    .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;
    //posts.push(query_post);
    //for post_id in post_ids.iter() {
    //    let post = redis_service
    //        .get(&format!("post:{}", post_id))
    //        .await
    //        .map_err(|e| ApiError::InternalServerError(format!("{}", e)))?;
    //    posts.push(post);
    //}


    Ok(HttpResponse::Ok().json(post_ids))
}

fn hash_vector(vector: &[f32]) -> String {
    let mut hasher = Sha256::new();

    for &val in vector {
        hasher.update(&val.to_le_bytes()); // little-endian to match Python
    }

    let hash = hasher.finalize();
    format!("{:x}", hash)
}
