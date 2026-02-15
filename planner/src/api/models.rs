use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub(super) struct RouteQuery {
    pub(super) start: String,
    pub(super) end: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct SuggestionRequest {
    pub(super) start: String,
    pub(super) end: String,
    #[serde(default = "default_max_suggestions")]
    pub(super) max_suggestions: usize,
    #[serde(default)]
    pub(super) preferences: SuggestionPreferences,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(super) struct SuggestionPreferences {
    pub(super) fitness_level: f64,
    pub(super) scenic_preference: f64,
    pub(super) avoid_main_roads: f64,
    pub(super) time_priority: f64,
}

#[derive(Clone, Serialize)]
pub(super) struct SuggestionMetrics {
    pub(super) distance_m: f64,
    pub(super) duration_s: f64,
    pub(super) ascend_m: f64,
    pub(super) scenic_ratio: f64,
    pub(super) major_road_ratio: f64,
}

#[derive(Serialize)]
pub(super) struct RouteSuggestion {
    pub(super) id: String,
    pub(super) score: f64,
    pub(super) explanation: String,
    pub(super) metrics: SuggestionMetrics,
    pub(super) route: Value,
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
    pub(super) fn clamped(self) -> Self {
        Self {
            fitness_level: self.fitness_level.clamp(0.0, 1.0),
            scenic_preference: self.scenic_preference.clamp(0.0, 1.0),
            avoid_main_roads: self.avoid_main_roads.clamp(0.0, 1.0),
            time_priority: self.time_priority.clamp(0.0, 1.0),
        }
    }
}

fn default_max_suggestions() -> usize {
    3
}
