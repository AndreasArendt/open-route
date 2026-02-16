use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::{json, Value};

use super::graphhopper::{
    build_basic_route_url, build_roundtrip_json_body, call_graphhopper, call_graphhopper_get,
};
use super::models::{RouteQuery, SuggestionRequest};
use super::scoring::build_suggestions;
use super::util::parse_latlon;

pub(super) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health).service(route).service(suggestions);
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[get("/route")]
async fn route(q: web::Query<RouteQuery>) -> impl Responder {
    let (s_lat, s_lon) = match parse_latlon(&q.start) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("start must be 'lat,lon'"),
    };
    let (e_lat, e_lon) = match parse_latlon(&q.end) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("end must be 'lat,lon'"),
    };

    let gh_base = std::env::var("GH_BASE_URL").unwrap_or_else(|_| "http://localhost:8989".into());
    let url = build_basic_route_url(&gh_base, (s_lat, s_lon), (e_lat, e_lon));

    let text = match call_graphhopper_get(&url).await {
        Ok(t) => t,
        Err(e) => return HttpResponse::BadGateway().body(e),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(text)
}

#[post("/suggestions")]
async fn suggestions(req: web::Json<SuggestionRequest>) -> impl Responder {
    let payload = req.into_inner();
    let (s_lat, s_lon) = match parse_latlon(&payload.start) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("start must be 'lat,lon'"),
    };
    let (_e_lat, _e_lon) = match parse_latlon(&payload.end) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("end must be 'lat,lon'"),
    };

    let max_suggestions = payload.max_suggestions.clamp(1, 6);
    let preferences = payload.preferences.clamped();
    let gh_base = std::env::var("GH_BASE_URL").unwrap_or_else(|_| "http://localhost:8989".into());
    let json = build_roundtrip_json_body((s_lat, s_lon));

    let text = match call_graphhopper(&gh_base, &json).await {
        Ok(t) => t,
        Err(e) => return HttpResponse::BadGateway().body(e),
    };

    let gh_json: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::BadGateway()
                .body(format!("GraphHopper response was not valid JSON: {e}"));
        }
    };

    let Some(paths) = gh_json.get("paths").and_then(Value::as_array) else {
        return HttpResponse::BadGateway().body("GraphHopper response missing paths array");
    };

    let ranked = build_suggestions(paths, preferences.clone(), max_suggestions);
    HttpResponse::Ok().json(json!({
        "suggestions": ranked,
        "meta": {
            "source_paths": paths.len(),
            "returned_suggestions": ranked.len()
        },
        "preferences": {
            "fitness_level": preferences.fitness_level,
            "scenic_preference": preferences.scenic_preference,
            "avoid_main_roads": preferences.avoid_main_roads,
            "time_priority": preferences.time_priority
        }
    }))
}
