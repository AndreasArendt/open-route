import { useEffect, useRef } from 'react';
import L from 'leaflet';

type RouteMapSuggestion = {
  id: string;
  route: unknown;
};

type RouteMapProps = {
  suggestions: RouteMapSuggestion[];
  activeSuggestionId: string | null;
  onSelectSuggestion: (id: string) => void;
};

const ROUTE_COLORS = ['#13a574', '#1697a6', '#3273dc', '#ec7a08', '#c66d3d', '#8844b0'];

function extractLatLngs(route: unknown): [number, number][] {
  if (!route || typeof route !== 'object') {
    return [];
  }

  const maybePoints = (route as { points?: unknown }).points;
  if (!maybePoints || typeof maybePoints !== 'object') {
    return [];
  }

  const maybeCoordinates = (maybePoints as { coordinates?: unknown }).coordinates;
  if (!Array.isArray(maybeCoordinates)) {
    return [];
  }

  const latLngs: [number, number][] = [];

  for (const coordinate of maybeCoordinates) {
    if (!Array.isArray(coordinate) || coordinate.length < 2) {
      continue;
    }

    const lon = Number(coordinate[0]);
    const lat = Number(coordinate[1]);

    if (!Number.isFinite(lat) || !Number.isFinite(lon)) {
      continue;
    }

    latLngs.push([lat, lon]);
  }

  return latLngs;
}

export default function RouteMap({
  suggestions,
  activeSuggestionId,
  onSelectSuggestion,
}: RouteMapProps) {
  const mapContainerRef = useRef<HTMLDivElement | null>(null);
  const mapRef = useRef<L.Map | null>(null);
  const layerRef = useRef<L.LayerGroup | null>(null);

  useEffect(() => {
    if (!mapContainerRef.current || mapRef.current) {
      return;
    }

    const map = L.map(mapContainerRef.current, {
      preferCanvas: true,
      zoomControl: false,
    }).setView([48.137154, 11.576124], 10);

    L.control.zoom({ position: 'bottomright' }).addTo(map);

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution:
        '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
      maxZoom: 19,
    }).addTo(map);

    mapRef.current = map;
    layerRef.current = L.layerGroup().addTo(map);

    return () => {
      map.remove();
      mapRef.current = null;
      layerRef.current = null;
    };
  }, []);

  useEffect(() => {
    const map = mapRef.current;
    const layer = layerRef.current;
    if (!map || !layer) {
      return;
    }

    layer.clearLayers();

    let allBounds: L.LatLngBounds | null = null;
    let activeBounds: L.LatLngBounds | null = null;
    let activeLine: [number, number][] = [];

    for (const [index, suggestion] of suggestions.entries()) {
      const latLngs = extractLatLngs(suggestion.route);
      if (latLngs.length < 2) {
        continue;
      }

      const isActive = suggestion.id === activeSuggestionId;
      const color = ROUTE_COLORS[index % ROUTE_COLORS.length];

      const polyline = L.polyline(latLngs, {
        color,
        weight: isActive ? 7 : 4,
        opacity: isActive ? 0.95 : 0.45,
        lineCap: 'round',
        lineJoin: 'round',
      });

      polyline.on('click', () => onSelectSuggestion(suggestion.id));
      polyline.addTo(layer);

      const bounds = L.latLngBounds(latLngs);
      allBounds = allBounds ? allBounds.extend(bounds) : bounds;

      if (isActive) {
        activeBounds = bounds;
        activeLine = latLngs;
        polyline.bringToFront();
      }
    }

    if (activeLine.length >= 2) {
      const start = activeLine[0];
      const end = activeLine[activeLine.length - 1];

      L.circleMarker(start, {
        radius: 6,
        color: '#0f766e',
        weight: 2,
        fillColor: '#ffffff',
        fillOpacity: 1,
      }).addTo(layer);

      L.circleMarker(end, {
        radius: 6,
        color: '#ec7a08',
        weight: 2,
        fillColor: '#ffffff',
        fillOpacity: 1,
      }).addTo(layer);
    }

    const targetBounds = activeBounds ?? allBounds;
    if (targetBounds) {
      map.fitBounds(targetBounds.pad(0.12), { padding: [24, 24], maxZoom: 14 });
    }
  }, [suggestions, activeSuggestionId, onSelectSuggestion]);

  return <div ref={mapContainerRef} className="map-canvas" />;
}
