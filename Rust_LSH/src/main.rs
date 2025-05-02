use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod errors;
mod handlers;
mod models;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    
    let redis_service = services::redis_service::RedisService::new(&redis_url)
        .expect("Failed to connect to Redis");

    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let port = env::var("RUST_API_PORT").unwrap_or_else(|_| "8080".into()).parse::<u16>().unwrap_or(8080);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_service.clone()))
            .service(routes::kv_routes::kv_routes())
    })
    .bind(format!("{}:{}", base_url, port))?
    .run()
    .await
}
