pub mod kv_routes;
pub mod healthcheck;
pub mod lsh_routes;
use actix_web::web;


pub fn all() -> actix_web::Scope {
    web::scope("/api")
        .service(kv_routes::kv_routes())
        .service(lsh_routes::lsh_routes())
        .service(healthcheck::healthcheck_routes())
}
