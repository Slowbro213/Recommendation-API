use crate::errors::ApiError;
use redis::{AsyncCommands, Client};

#[derive(Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub fn new(redis_url: &str) -> Result<Self, ApiError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), ApiError> {
        let mut conn = self.client.get_async_connection().await?;
        // Explicitly handle the return value
        let _: () = conn.set(key, value).await?;
        Ok(())
    }

   pub async fn get(&self, key: &str) -> Result<String, ApiError> {
    let mut conn = self.client.get_async_connection().await?;
    match conn.get(key).await {
        Ok(value) => Ok(value),
        Err(e) if e.kind() == redis::ErrorKind::TypeError => {
            Err(ApiError::NotFound(format!("Key '{}' not found", key)))
        }
        Err(e) => Err(e.into()),
    }
}}
