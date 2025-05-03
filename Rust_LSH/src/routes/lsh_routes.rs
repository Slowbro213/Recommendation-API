
use actix_web::web;
use crate::handlers::lsh_handler;

pub fn lsh_routes() -> actix_web::Scope {
    web::scope("/lsh")
        .route("/query", web::get().to(lsh_handler::query))
        .route("", web::post().to(lsh_handler::add))
}
