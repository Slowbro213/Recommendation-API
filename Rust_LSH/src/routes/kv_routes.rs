use actix_web::web;
use crate::handlers::kv_handlers;

pub fn kv_routes() -> actix_web::Scope {
    web::scope("/kv")
        .route("/{key}", web::get().to(kv_handlers::get_key))
        .route("", web::post().to(kv_handlers::set_key))
}
