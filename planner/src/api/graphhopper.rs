pub(super) fn build_basic_route_url(gh_base: &str, start: (f64, f64), end: (f64, f64)) -> String {
    format!(
        "{gh_base}/route?profile=bike&point={},{}&point={},{}&calc_points=true&points_encoded=false&instructions=true",
        start.0, start.1, end.0, end.1
    )
}

pub(super) fn build_suggestion_route_url(
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

pub(super) async fn call_graphhopper(url: &str) -> Result<String, String> {
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
