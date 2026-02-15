import { FormEvent, KeyboardEvent, useEffect, useMemo, useState } from 'react';
import RouteMap from './RouteMap';

type Preferences = {
  fitness_level: number;
  scenic_preference: number;
  avoid_main_roads: number;
  time_priority: number;
};

type SuggestionMetrics = {
  distance_m: number;
  duration_s: number;
  ascend_m: number;
  scenic_ratio: number;
  major_road_ratio: number;
};

type Suggestion = {
  id: string;
  score: number;
  explanation: string;
  metrics: SuggestionMetrics;
  route: unknown;
};

type SuggestionResponse = {
  suggestions: Suggestion[];
  meta?: {
    source_paths: number;
    returned_suggestions: number;
  };
};

const DEFAULT_START = '48.137154,11.576124';
const DEFAULT_END = '48.370545,10.897790';

const ROUTE_COLORS = ['#13a574', '#1697a6', '#3273dc', '#ec7a08', '#c66d3d', '#8844b0'];

const DEFAULT_PREFS: Preferences = {
  fitness_level: 0.5,
  scenic_preference: 0.7,
  avoid_main_roads: 0.7,
  time_priority: 0.4,
};

function sliderLabel(value: number): string {
  return `${Math.round(value * 100)}%`;
}

function km(meters: number): string {
  return `${(meters / 1000).toFixed(1)} km`;
}

function mins(seconds: number): string {
  return `${Math.round(seconds / 60)} min`;
}

function routeColor(index: number): string {
  return ROUTE_COLORS[index % ROUTE_COLORS.length];
}

export default function App() {
  const [start, setStart] = useState(DEFAULT_START);
  const [end, setEnd] = useState(DEFAULT_END);
  const [maxSuggestions, setMaxSuggestions] = useState(3);
  const [preferences, setPreferences] = useState<Preferences>(DEFAULT_PREFS);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<SuggestionResponse | null>(null);
  const [activeSuggestionId, setActiveSuggestionId] = useState<string | null>(null);

  const apiBase = useMemo(() => {
    const meta = import.meta as ImportMeta & {
      env?: { VITE_API_BASE_URL?: string };
    };
    return meta.env?.VITE_API_BASE_URL ?? 'http://localhost:8080';
  }, []);

  const suggestions = response?.suggestions ?? [];
  const activeSuggestion =
    suggestions.find((item) => item.id === activeSuggestionId) ??
    suggestions[0] ??
    null;

  useEffect(() => {
    if (suggestions.length === 0) {
      setActiveSuggestionId(null);
      return;
    }

    if (!activeSuggestionId || !suggestions.some((item) => item.id === activeSuggestionId)) {
      setActiveSuggestionId(suggestions[0].id);
    }
  }, [suggestions, activeSuggestionId]);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const res = await fetch(`${apiBase}/suggestions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          start,
          end,
          max_suggestions: maxSuggestions,
          preferences,
        }),
      });

      if (!res.ok) {
        const body = await res.text();
        throw new Error(body || `Request failed (${res.status})`);
      }

      const data = (await res.json()) as SuggestionResponse;
      setResponse(data);
      setActiveSuggestionId(data.suggestions[0]?.id ?? null);
    } catch (requestError) {
      const message =
        requestError instanceof Error ? requestError.message : 'Unknown network error';
      setError(message);
    } finally {
      setLoading(false);
    }
  }

  function updatePreference(key: keyof Preferences, value: number) {
    setPreferences((prev) => ({ ...prev, [key]: value }));
  }

  function onCardKeyDown(event: KeyboardEvent<HTMLElement>, id: string) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      setActiveSuggestionId(id);
    }
  }

  return (
    <div className="page">
      <header>
        <p className="eyebrow">open-route</p>
        <h1>Bike Route Suggestions</h1>
        <p className="subcopy">
          Tune your route profile, then rank alternatives by speed, terrain, and road feel.
        </p>
      </header>

      <div className="layout">
        <form className="panel control-panel" onSubmit={onSubmit}>
          <h2>Request</h2>

          <label>
            Start (lat,lon)
            <input
              value={start}
              onChange={(e) => setStart(e.target.value)}
              placeholder="48.137154,11.576124"
            />
          </label>

          <label>
            End (lat,lon)
            <input
              value={end}
              onChange={(e) => setEnd(e.target.value)}
              placeholder="48.370545,10.897790"
            />
          </label>

          <label>
            Alternatives
            <input
              type="number"
              min={1}
              max={6}
              value={maxSuggestions}
              onChange={(e) =>
                setMaxSuggestions(Math.min(6, Math.max(1, Number(e.target.value))))
              }
            />
          </label>

          <div className="slider">
            <label>
              Fitness level
              <span>{sliderLabel(preferences.fitness_level)}</span>
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={preferences.fitness_level}
              onChange={(e) => updatePreference('fitness_level', Number(e.target.value))}
            />
          </div>

          <div className="slider">
            <label>
              Scenic preference
              <span>{sliderLabel(preferences.scenic_preference)}</span>
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={preferences.scenic_preference}
              onChange={(e) => updatePreference('scenic_preference', Number(e.target.value))}
            />
          </div>

          <div className="slider">
            <label>
              Avoid main roads
              <span>{sliderLabel(preferences.avoid_main_roads)}</span>
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={preferences.avoid_main_roads}
              onChange={(e) => updatePreference('avoid_main_roads', Number(e.target.value))}
            />
          </div>

          <div className="slider">
            <label>
              Time priority
              <span>{sliderLabel(preferences.time_priority)}</span>
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={preferences.time_priority}
              onChange={(e) => updatePreference('time_priority', Number(e.target.value))}
            />
          </div>

          <button type="submit" disabled={loading}>
            {loading ? 'Ranking routes...' : 'Suggest routes'}
          </button>

          <p className="hint">API base: {apiBase}</p>
          {error && <p className="error">{error}</p>}
        </form>

        <div className="planner-main">
          <section className="panel map-panel">
            <div className="map-head">
              <h2>Route Map</h2>
              <p className="map-sub">
                {activeSuggestion
                  ? `Active: ${activeSuggestion.id} (${km(activeSuggestion.metrics.distance_m)}, ${mins(
                      activeSuggestion.metrics.duration_s
                    )})`
                  : 'Request suggestions to render routes on the map.'}
              </p>
            </div>

            {suggestions.length > 0 && (
              <div className="route-chip-row">
                {suggestions.map((item, index) => (
                  <button
                    key={item.id}
                    type="button"
                    className={`route-chip ${
                      item.id === activeSuggestion?.id ? 'active' : ''
                    }`}
                    onClick={() => setActiveSuggestionId(item.id)}
                  >
                    <span
                      className="route-chip-dot"
                      style={{ backgroundColor: routeColor(index) }}
                    />
                    {item.id}
                  </button>
                ))}
              </div>
            )}

            <div className="map-shell">
              <RouteMap
                suggestions={suggestions}
                activeSuggestionId={activeSuggestion?.id ?? null}
                onSelectSuggestion={setActiveSuggestionId}
              />
              {suggestions.length === 0 && (
                <div className="map-empty">
                  Run a suggestion request to plot alternatives here.
                </div>
              )}
            </div>
          </section>

          <section className="panel results">
            <h2>Suggestions</h2>

            {!response && (
              <p className="hint">
                Submit a request to score and compare alternatives.
              </p>
            )}

            {response?.meta && (
              <p className="hint">
                Received {response.meta.source_paths} candidate path(s), returning{' '}
                {response.meta.returned_suggestions}.
              </p>
            )}

            <div className="cards">
              {suggestions.map((item, index) => (
                <article
                  className={`card ${
                    item.id === activeSuggestion?.id ? 'active' : ''
                  }`}
                  key={item.id}
                  role="button"
                  tabIndex={0}
                  onClick={() => setActiveSuggestionId(item.id)}
                  onKeyDown={(event) => onCardKeyDown(event, item.id)}
                >
                  <div className="card-head">
                    <div className="card-title-wrap">
                      <span
                        className="route-dot"
                        style={{ backgroundColor: routeColor(index) }}
                      />
                      <h3>{item.id}</h3>
                    </div>
                    <p className="score">score {item.score.toFixed(3)}</p>
                  </div>

                  <p className="explain">{item.explanation}</p>

                  <ul>
                    <li>Distance: {km(item.metrics.distance_m)}</li>
                    <li>ETA: {mins(item.metrics.duration_s)}</li>
                    <li>Ascent: {item.metrics.ascend_m.toFixed(0)} m</li>
                    <li>
                      Scenic ratio: {(item.metrics.scenic_ratio * 100).toFixed(1)}%
                    </li>
                    <li>
                      Main road ratio: {(item.metrics.major_road_ratio * 100).toFixed(1)}%
                    </li>
                  </ul>

                  <details>
                    <summary>Raw route JSON</summary>
                    <pre>{JSON.stringify(item.route, null, 2)}</pre>
                  </details>
                </article>
              ))}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
