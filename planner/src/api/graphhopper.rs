use serde_json::{json, Value};

fn route_endpoint(gh_base: &str) -> String {
    format!("{}/route", gh_base.trim_end_matches('/'))
}

pub(super) fn build_basic_route_url(gh_base: &str, start: (f64, f64), end: (f64, f64)) -> String {
    let endpoint = route_endpoint(gh_base);
    format!(
        "{endpoint}?profile=bike&point={},{}&point={},{}&calc_points=true&points_encoded=false&instructions=true",
        start.0, start.1, end.0, end.1
    )
}

pub(super) fn build_roundtrip_json_body(start: (f64, f64)) -> Value {
    let distance_m = 100000.0; // 100 km
    let seed = 112;
    // let heading_deg = 90; // optional: initial direction hint (east)

    log::info!(
        "Building round-trip JSON body for start {},{}",
        start.0,
        start.1
    );
    print!("Building round-trip JSON body for start {},{}", start.0, start.1);

    json!({
        "profile": "bike",
        "algorithm": "round_trip",
        "points": [[start.1, start.0]],
        // "headings": [heading_deg],

        "round_trip" : true,
        "round_trip.distance": distance_m,
        "round_trip.seed": seed,

        // Output controls (these are body fields for POST)
        "calc_points": true,
        "points_encoded": false,
        "instructions": true,

        // Ask for per-edge attributes so you can post-score.
        // Keep this aligned with encoded values available in your GH profile.
        "details": ["road_class"],

        "custom_model": {
            "priority": [
                { "if": "road_class == MOTORWAY || road_class == TRUNK", "multiply_by": 0.0 },
                { "if": "road_class == PRIMARY", "multiply_by": 0.4 },
                { "if": "road_class == SECONDARY", "multiply_by": 1.0 },
                { "if": "road_class == TERTIARY", "multiply_by": 1.0 },
                { "if": "road_class == RESIDENTIAL", "multiply_by": 0.8 },
                { "if": "road_class == SERVICE", "multiply_by": 0.7 },
                { "if": "road_class == UNCLASSIFIED", "multiply_by": 1.0 },
                { "if": "road_class == CYCLEWAY", "multiply_by": 1.0 },
                { "if": "road_class == FOOTWAY || road_class == PATH || road_class == PEDESTRIAN", "multiply_by": 1.0 },
                { "if": "surface != PAVED", "multiply_by": 0.6 }
            ]
        }

        // If you want hard “no cities”: add "avoid_polygons": { ...geojson... }
    })
}

pub(super) async fn call_graphhopper(url: &str, json: &Value) -> Result<String, String> {
    let endpoint = route_endpoint(url);
    log::info!("Calling GraphHopper: {endpoint}");

    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .json(json)
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

pub(super) async fn call_graphhopper_get(url: &str) -> Result<String, String> {
    log::info!("Calling GraphHopper: {url}");

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
