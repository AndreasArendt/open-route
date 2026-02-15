use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;

#[derive(Deserialize)]
struct RouteQuery {
    start: String, // "lat,lon"
    end: String,   // "lat,lon"
}

#[derive(Debug, Deserialize)]
struct SuggestionRequest {
    start: String, // "lat,lon"
    end: String,   // "lat,lon"
    #[serde(default = "default_max_suggestions")]
    max_suggestions: usize,
    #[serde(default)]
    preferences: SuggestionPreferences,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SuggestionPreferences {
    // 0.0 = low fitness (prefer flatter routes), 1.0 = high fitness
    fitness_level: f64,
    // 0.0 = no scenic preference, 1.0 = strongly scenic
    scenic_preference: f64,
    // 0.0 = no avoidance, 1.0 = strongly avoid major roads
    avoid_main_roads: f64,
    // 0.0 = distance priority, 1.0 = time priority
    time_priority: f64,
}

#[derive(Clone, Serialize)]
struct SuggestionMetrics {
    distance_m: f64,
    duration_s: f64,
    ascend_m: f64,
    scenic_ratio: f64,
    major_road_ratio: f64,
}

#[derive(Serialize)]
struct RouteSuggestion {
    id: String,
    score: f64,
    explanation: String,
    metrics: SuggestionMetrics,
    route: Value,
}

impl Default for SuggestionPreferences {
    fn default() -> Self {
        Self {
            fitness_level: 0.5,
            scenic_preference: 0.5,
            avoid_main_roads: 0.5,
            time_priority: 0.5,
        }
    }
}

impl SuggestionPreferences {
    fn clamped(self) -> Self {
        Self {
            fitness_level: clamp01(self.fitness_level),
            scenic_preference: clamp01(self.scenic_preference),
            avoid_main_roads: clamp01(self.avoid_main_roads),
            time_priority: clamp01(self.time_priority),
        }
    }
}

fn default_max_suggestions() -> usize {
    3
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

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        0.5
    } else {
        v.clamp(0.0, 1.0)
    }
}

fn normalize(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        0.5
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn min_max(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values {
        if *value < min {
            min = *value;
        }
        if *value > max {
            max = *value;
        }
    }
    (min, max)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn build_basic_route_url(gh_base: &str, start: (f64, f64), end: (f64, f64)) -> String {
    format!(
        "{gh_base}/route?profile=bike&point={},{}&point={},{}&calc_points=true&points_encoded=false&instructions=true",
        start.0, start.1, end.0, end.1
    )
}

fn build_suggestion_route_url(
    gh_base: &str,
    start: (f64, f64),
    end: (f64, f64),
    max_suggestions: usize,
) -> String {
    format!(
        "{gh_base}/route?profile=bike&point={},{}&point={},{}&calc_points=true&points_encoded=false&instructions=true&details=road_class&algorithm=alternative_route&alternative_route.max_paths={max_suggestions}&ch.disable=true",
        start.0, start.1, end.0, end.1
    )
}

async fn call_graphhopper(url: &str) -> Result<String, String> {
    info!("Calling GraphHopper: {url}");

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("GraphHopper request failed: {e}"))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("GraphHopper read failed: {e}"))?;

    if !status.is_success() {
        return Err(format!("GraphHopper error ({status}): {text}"));
    }

    Ok(text)
}

fn road_class_ratios(path: &Value) -> (f64, f64) {
    // Proxy for "scenic" vs "main roads" from GraphHopper road_class details.
    let scenic_classes = [
        "cycleway",
        "path",
        "track",
        "living_street",
        "residential",
        "service",
    ];
    let major_classes = ["motorway", "trunk", "primary", "secondary", "tertiary"];

    let mut scenic_total = 0.0;
    let mut major_total = 0.0;
    let mut total = 0.0;

    if let Some(entries) = path.pointer("/details/road_class").and_then(Value::as_array) {
        for entry in entries {
            let Some(tuple) = entry.as_array() else {
                continue;
            };
            if tuple.len() < 3 {
                continue;
            }
            let Some(from) = tuple[0].as_u64() else {
                continue;
            };
            let Some(to) = tuple[1].as_u64() else {
                continue;
            };
            let Some(class_name) = tuple[2].as_str() else {
                continue;
            };

            let segment = to.saturating_sub(from) as f64;
            if segment <= 0.0 {
                continue;
            }

            total += segment;

            if scenic_classes.contains(&class_name) {
                scenic_total += segment;
            }
            if major_classes.contains(&class_name) {
                major_total += segment;
            }
        }
    }

    if total <= 0.0 {
        return (0.5, 0.5);
    }

    (scenic_total / total, major_total / total)
}

fn explanation_for(
    metrics: &SuggestionMetrics,
    preferences: &SuggestionPreferences,
    distance_score: f64,
    duration_score: f64,
    climb_score: f64,
) -> String {
    let mut reasons: Vec<&str> = Vec::new();

    if preferences.time_priority >= 0.6 && duration_score >= 0.5 {
        reasons.push("keeps travel time lower");
    } else if preferences.time_priority < 0.6 && distance_score >= 0.5 {
        reasons.push("keeps total distance shorter");
    }

    if preferences.scenic_preference >= 0.6 && metrics.scenic_ratio >= 0.45 {
        reasons.push("uses more scenic road segments");
    }

    if preferences.avoid_main_roads >= 0.6 && metrics.major_road_ratio <= 0.35 {
        reasons.push("stays away from main roads");
    }

    if preferences.fitness_level < 0.5 && climb_score >= 0.5 {
        reasons.push("reduces climbing effort");
    }

    if reasons.is_empty() {
        return "Balanced option based on your current preference mix.".to_string();
    }
    if reasons.len() == 1 {
        return format!("Selected because it {}.", reasons[0]);
    }

    format!("Selected because it {} and {}.", reasons[0], reasons[1])
}

fn build_suggestions(
    paths: &[Value],
    preferences: SuggestionPreferences,
    limit: usize,
) -> Vec<RouteSuggestion> {
    if paths.is_empty() {
        return Vec::new();
    }

    let mut metrics_by_path: Vec<SuggestionMetrics> = Vec::with_capacity(paths.len());
    for path in paths {
        let distance_m = path.get("distance").and_then(Value::as_f64).unwrap_or(0.0);
        let duration_s = path.get("time").and_then(Value::as_f64).unwrap_or(0.0) / 1000.0;
        let ascend_m = path.get("ascend").and_then(Value::as_f64).unwrap_or(0.0);
        let (scenic_ratio, major_road_ratio) = road_class_ratios(path);
        metrics_by_path.push(SuggestionMetrics {
            distance_m,
            duration_s,
            ascend_m,
            scenic_ratio,
            major_road_ratio,
        });
    }

    let distances: Vec<f64> = metrics_by_path.iter().map(|m| m.distance_m).collect();
    let durations: Vec<f64> = metrics_by_path.iter().map(|m| m.duration_s).collect();
    let climbs: Vec<f64> = metrics_by_path.iter().map(|m| m.ascend_m).collect();

    let (min_distance, max_distance) = min_max(&distances);
    let (min_duration, max_duration) = min_max(&durations);
    let (min_climb, max_climb) = min_max(&climbs);

    let mut ranked: Vec<RouteSuggestion> = paths
        .iter()
        .zip(metrics_by_path.into_iter())
        .enumerate()
        .map(|(index, (path, metrics))| {
            let distance_score = 1.0 - normalize(metrics.distance_m, min_distance, max_distance);
            let duration_score = 1.0 - normalize(metrics.duration_s, min_duration, max_duration);
            let climb_score = 1.0 - normalize(metrics.ascend_m, min_climb, max_climb);

            let travel_efficiency =
                (1.0 - preferences.time_priority) * distance_score + preferences.time_priority * duration_score;
            let climb_weight = 1.0 - preferences.fitness_level;

            let total_score = (travel_efficiency * 0.45)
                + (metrics.scenic_ratio * (0.2 + 0.3 * preferences.scenic_preference))
                + ((1.0 - metrics.major_road_ratio) * (0.2 + 0.3 * preferences.avoid_main_roads))
                + (climb_score * (0.15 + 0.25 * climb_weight));

            let explanation = explanation_for(
                &metrics,
                &preferences,
                distance_score,
                duration_score,
                climb_score,
            );

            RouteSuggestion {
                id: format!("candidate-{}", index + 1),
                score: round3(total_score),
                explanation,
                metrics: SuggestionMetrics {
                    distance_m: round2(metrics.distance_m),
                    duration_s: round2(metrics.duration_s),
                    ascend_m: round2(metrics.ascend_m),
                    scenic_ratio: round3(metrics.scenic_ratio),
                    major_road_ratio: round3(metrics.major_road_ratio),
                },
                route: path.clone(),
            }
        })
        .collect();

    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    ranked.truncate(limit);

    for (index, suggestion) in ranked.iter_mut().enumerate() {
        suggestion.id = format!("suggestion-{}", index + 1);
    }

    ranked
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
    let text = match call_graphhopper(&url).await {
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
    let (e_lat, e_lon) = match parse_latlon(&payload.end) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("end must be 'lat,lon'"),
    };

    let max_suggestions = payload.max_suggestions.clamp(1, 6);
    let preferences = payload.preferences.clamped();
    let gh_base = std::env::var("GH_BASE_URL").unwrap_or_else(|_| "http://localhost:8989".into());

    let url = build_suggestion_route_url(&gh_base, (s_lat, s_lon), (e_lat, e_lon), max_suggestions);
    let text = match call_graphhopper(&url).await {
        Ok(t) => t,
        Err(e) => return HttpResponse::BadGateway().body(e),
    };

    let gh_json: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::BadGateway().body(format!(
                "GraphHopper response was not valid JSON: {e}"
            ));
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    log::info!("Planner starting on 0.0.0.0:8080");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .service(health)
            .service(route)
            .service(suggestions)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
