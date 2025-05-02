use actix_web::web;
use crate::handlers::healthcheck;

pub fn healthcheck_routes() -> actix_web::Scope {
    web::scope("")
        .route("/health", web::get().to(healthcheck::healthcheck))
}
