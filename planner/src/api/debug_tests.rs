use serde_json::{json, Value};

use super::models::SuggestionPreferences;
use super::scoring::build_suggestions;
use super::util::parse_latlon;

#[test]
fn parse_latlon_handles_valid_and_invalid_inputs() {
    assert_eq!(parse_latlon("48.137154,11.576124"), Some((48.137154, 11.576124)));
    assert_eq!(parse_latlon("48.137154"), None);
    assert_eq!(parse_latlon("x,y"), None);
}

#[test]
fn scoring_builds_ranked_limited_suggestions() {
    let paths: Vec<Value> = vec![
        json!({
            "distance": 1000.0,
            "time": 300000.0,
            "ascend": 10.0,
            "details": { "road_class": [[0, 6, "cycleway"], [6, 10, "residential"]] }
        }),
        json!({
            "distance": 900.0,
            "time": 260000.0,
            "ascend": 45.0,
            "details": { "road_class": [[0, 4, "primary"], [4, 10, "secondary"]] }
        }),
        json!({
            "distance": 1100.0,
            "time": 330000.0,
            "ascend": 12.0,
            "details": { "road_class": [[0, 3, "path"], [3, 10, "service"]] }
        }),
    ];

    let preferences = SuggestionPreferences {
        fitness_level: 0.2,
        scenic_preference: 0.9,
        avoid_main_roads: 0.9,
        time_priority: 0.4,
    };

    let ranked = build_suggestions(&paths, preferences, 2);

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].id, "suggestion-1");
    assert_eq!(ranked[1].id, "suggestion-2");
    assert!(ranked[0].score >= ranked[1].score);
}
