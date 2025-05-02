use actix_web::{web, HttpResponse};
use crate::{errors::ApiError, services::lsh_service::LSHService};



pub fn add(
    lsh: web::Data<LSHService>,
    vectors: web::Json<Vec<Vec<f32>>>,
) -> Result<HttpResponse, ApiError> {
    lsh.add(&vectors).map_err(|e| ApiError::InternalServerError(format!("Failed to add vectors: {}", e)))?;
    Ok(HttpResponse::Ok().json("Vectors added"))
}



pub fn query(
    lsh: web::Data<LSHService>,
    query: web::Json<Vec<f32>>,
    n_results: web::Query<usize>,
) -> Result<HttpResponse, ApiError> {
    let lsh = lsh;
    let results = lsh.query(&query, *n_results).map_err(|e| ApiError::InternalServerError(format!("Query failed: {}", e)))?;
    Ok(HttpResponse::Ok().json(results))
}
