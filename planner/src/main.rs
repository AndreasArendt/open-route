use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct RouteQuery {
    start: String, // "lat,lon"
    end: String,   // "lat,lon"
}

fn parse_latlon(s: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let lat = parts[0].trim().parse::<f64>().ok()?;
    let lon = parts[1].trim().parse::<f64>().ok()?;
    Some((lat, lon))
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"ok": true}))
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

    // GraphHopper uses point=lat,lon parameters
    // We'll ask it for geometry (points) and instructions.
    let url = format!(
        "{}/route?profile=bike&point={},{}&point={},{}&calc_points=true&points_encoded=false&instructions=true",
        gh_base, s_lat, s_lon, e_lat, e_lon
    );

    info!("Calling GraphHopper: {}", url);

    let client = reqwest::Client::new();
    let resp = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => return HttpResponse::BadGateway().body(format!("GraphHopper request failed: {e}")),
    };

    let status = resp.status();
    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::BadGateway().body(format!("GraphHopper read failed: {e}")),
    };

    if !status.is_success() {
        return HttpResponse::BadGateway().body(format!("GraphHopper error ({status}): {text}"));
    }

    // Pass-through for now. Later: generate multiple candidates + score + explain.
    HttpResponse::Ok()
        .content_type("application/json")
        .body(text)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    log::info!("Planner starting on 0.0.0.0:8080");

    HttpServer::new(|| App::new().service(health).service(route))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
