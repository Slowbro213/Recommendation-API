use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Key not found")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::RedisError(msg) => HttpResponse::InternalServerError().json(msg),
            ApiError::NotFound(msg) => HttpResponse::NotFound().json(msg),
            ApiError::BadRequest(msg) => HttpResponse::BadRequest().json(msg),
            ApiError::InternalServerError(msg) => HttpResponse::InternalServerError().json(msg),
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        ApiError::RedisError(err.to_string())
    }
}
