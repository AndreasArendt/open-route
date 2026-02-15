mod graphhopper;
mod handlers;
mod models;
mod scoring;
mod util;
#[cfg(test)]
mod debug_tests;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    handlers::configure(cfg);
}
