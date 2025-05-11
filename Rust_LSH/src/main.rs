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
use crate::errors::ApiError;

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
            let lsh_service = lsh_service.clone();
            std::thread::spawn(move || {
                let vector = vector.clone();
                lsh_service.add(&[vector]).expect("Failed to insert vector into LSH service");
                println!("[Rust] Vector added to LSH service for post_id: {}", post_id);
            });

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



fn startup(redis_url: &str, lsh_service: &services::lsh_service::LSHService) -> Result<(), ApiError> {
    let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let mut conn = client.get_connection().expect("Redis connect failed");


    println!("[Rust] Starting up LSH service");

    let keys: Vec<String> = conn
        .keys("embedding:post:*")
        .expect("Failed to fetch keys");

    if keys.is_empty() {
        println!("No embedding keys found.");
        println!("[Rust] LSH service startup complete");
        return Ok(());
    }

    let values: Vec<String> = conn
        .get(keys.clone())
        .expect("Failed to fetch values");

    let mut counter : usize = 0;
    for value in values.iter() {
        let embedding: Vec<f32> = serde_json::from_str(&value)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to parse embedding: {}", e)))?;

        lsh_service.add(&[embedding]).expect("Failed to insert vector into LSH service");
        println!("[Rust] Vector added to LSH service {}", counter);
        counter += 1;
    }

    println!("[Rust] LSH service startup complete");

    Ok(())
}


fn graceful_shutdown() {
    println!("[Rust] Graceful shutdown initiated.");
    // Perform any necessary cleanup here
    std::process::exit(0);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".into());
    let redis_port = env::var("REDIS_PORT")
        .unwrap_or_else(|_| "6379".into())
        .parse::<u16>()
        .unwrap_or(6379);

    let redis_url = format!("redis://{}:{}", redis_host, redis_port);

    println!("[Rust] Redis URL: {}", redis_url);

    let redis_service = services::redis_service::RedisService::new(&redis_url)
        .expect("Failed to connect to Redis");

    let base_url = env::var("RUST_API_HOST").unwrap_or_else(|_| "127.0.0.1".into());
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
        let lsh_service = lsh_service.clone();
        startup(&redis_url, &lsh_service).expect("Failed to start up LSH service");
    }


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
