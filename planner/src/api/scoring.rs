use serde_json::Value;
use std::cmp::Ordering;

use super::models::{RouteSuggestion, SuggestionMetrics, SuggestionPreferences};
use super::util::{min_max, normalize, round2, round3};

fn road_class_ratios(path: &Value) -> (f64, f64) {
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

pub(super) fn build_suggestions(
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

            let travel_efficiency = (1.0 - preferences.time_priority) * distance_score
                + preferences.time_priority * duration_score;
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
