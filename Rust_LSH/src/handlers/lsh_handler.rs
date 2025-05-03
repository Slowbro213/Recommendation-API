use actix_web::{web, HttpResponse};
use crate::{errors::ApiError, services::lsh_service::LSHService};
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
    query: web::Json<Vec<f32>>,
    params: web::Query<QueryParams>,
) -> Result<HttpResponse, ApiError> {
    let results = lsh
        .query(&query, params.n_results)
        .map_err(|e| ApiError::InternalServerError(format!("{}",e)))?;
    Ok(HttpResponse::Ok().json(results))
}

