use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use redis::Commands;
use serde_json;

mod errors;
mod handlers;
mod models;
mod routes;
mod services;


fn redis_message_listener(redis_url: String, running: Arc<AtomicBool>, lsh_service: services::lsh_service::LSHService) {
    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");

    // Two independent connections
    let mut pubsub_conn = client.get_connection().expect("Redis connect failed (pubsub)");
    let mut data_conn = client.get_connection().expect("Redis connect failed (data)");

    let mut pubsub = pubsub_conn.as_pubsub();
    let _ = pubsub.subscribe("new_embedding");

    while running.load(Ordering::SeqCst) {
        if let Ok(msg) = pubsub.get_message() {
            let post_id: String = msg.get_payload().expect("Failed to get payload");
            println!("[Rust] Received new post_id: {}", post_id);

            let json_str: String = data_conn.get("embedding:post:".to_owned() + &post_id).expect("Failed to get vector");
            let vector: Vec<f32> = serde_json::from_str(&json_str).expect("Failed to parse vector");

            // Insert the vector into the LSH service 
            lsh_service.add(&[vector]).expect("Failed to insert vector into LSH service");

        }
    }

    println!("[Rust] Message listener shutting down.");
}



fn listen_for_shutdown(redis_url: String, running: Arc<AtomicBool>) {
    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let mut conn = client.get_connection().expect("Redis connect failed");
    let mut pubsub = conn.as_pubsub();
    let _ = pubsub.subscribe("shutdown");

    while running.load(Ordering::SeqCst) {
        if let Ok(msg) = pubsub.get_message() {
            if msg.get_channel_name() == "shutdown" {
                println!("[Rust] Received shutdown signal from Redis.");
                running.store(false, Ordering::SeqCst);
                graceful_shutdown();
                break;
            }
        }
    }
}


fn graceful_shutdown() {
    println!("[Rust] Graceful shutdown initiated.");
    // Perform any necessary cleanup here
    std::process::exit(0);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());

    let redis_service = services::redis_service::RedisService::new(&redis_url)
        .expect("Failed to connect to Redis");

    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("RUST_API_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse::<u16>()
        .unwrap_or(8080);

    let n_projections = 25;
    let n_hash_tables = 30000;
    let dim = 768;

    let lsh_service = services::lsh_service::LSHService::new(n_projections, n_hash_tables, dim);

    let running_flag = Arc::new(AtomicBool::new(true));

    {
        let redis_url = redis_url.clone();
        let flag = running_flag.clone();
        let lsh_clone = lsh_service.clone();
        thread::spawn(move || redis_message_listener(redis_url, flag, lsh_clone));
    }

    {
        let redis_url = redis_url.clone();
        let flag = running_flag.clone();
        thread::spawn(move || listen_for_shutdown(redis_url, flag));
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_service.clone()))
            .app_data(web::Data::new(lsh_service.clone()))
            .service(routes::all())
    })
    .bind((base_url.as_str(), port))?
    .run()
    .await
}
