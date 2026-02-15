#!/usr/bin/env bash
set -euo pipefail

mkdir -p data/osm

# Bavaria extract (choose latest "bayern-latest.osm.pbf" if you want daily, or pin a dated file)
# The Geofabrik page lists dated files; this uses the "latest" style URL pattern that Geofabrik supports for many regions.
# If your local "Bavaria subset" is already available, just place it at data/osm/bavaria.osm.pbf
URL="https://download.geofabrik.de/europe/germany/bayern-latest.osm.pbf"

echo "Downloading Bavaria PBF to ./data/osm/bavaria.osm.pbf"
curl -L "$URL" -o "data/osm/bavaria.osm.pbf"

echo "Done."