use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use tokio::task;



mod errors;
mod handlers;
mod models;
mod routes;
mod services;


async fn redis_listener(redis_url: String) {
    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let mut conn = client.get_connection().expect("Redis connect failed");

    let mut pubsub = conn.as_pubsub();
    let _ = pubsub.subscribe("new_embedding");
    let _ = pubsub.subscribe("shutdown");

    loop {
        let msg = pubsub.get_message().expect("Failed to get message");
        if msg.get_channel_name() == "shutdown"
        {
            println!("[Rust] Received shutdown signal");
            std::process::exit(0);
}
        let post_id: String = msg.get_payload().expect("Failed to get payload");
        println!("[Rust] Received new post_id: {}", post_id);
        // Fetch embedding, update LSH, etc.
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    
    let redis_service = services::redis_service::RedisService::new(&redis_url.clone())
        .expect("Failed to connect to Redis");

    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let port = env::var("RUST_API_PORT").unwrap_or_else(|_| "8080".into()).parse::<u16>().unwrap_or(8080);

    let n_projections = 9;
    let n_hash_tables = 10;
    let dim = 3;

    let lsh_service = services::lsh_service::LSHService::new(n_projections, n_hash_tables, dim);

    task::spawn(redis_listener(redis_url.clone()));

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_service.clone()))
            .app_data(web::Data::new(lsh_service.clone()))
            .service(routes::all())
    })
    .bind(format!("{}:{}", base_url, port))?
    .run()
    .await;

    Ok(())
}
