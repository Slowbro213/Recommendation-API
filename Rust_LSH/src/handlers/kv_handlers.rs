use actix_web::{web, HttpResponse};
use crate::{errors::ApiError, models::kv::KeyValue, services::redis_service::RedisService};

pub async fn get_key(
    path: web::Path<String>,
    redis: web::Data<RedisService>,
) -> Result<HttpResponse, ApiError> {
    let key = path.into_inner();
    redis.get(&key)
        .await
        .map(|value| HttpResponse::Ok().json(value))
}

pub async fn set_key(
    kv: web::Json<KeyValue>,
    redis: web::Data<RedisService>,
) -> Result<HttpResponse, ApiError> {
    if kv.key.is_empty() {
        return Err(ApiError::BadRequest("Key cannot be empty".into()));
    }
    redis.set(&kv.key, &kv.value).await?;
    Ok(HttpResponse::Ok().json("Value set"))
}
