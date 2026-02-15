$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path "data\osm" | Out-Null

$url = "https://download.geofabrik.de/europe/germany/bayern-latest.osm.pbf"
$out = "data\osm\bavaria.osm.pbf"

Write-Host "Downloading Bavaria PBF to $out"
Invoke-WebRequest -Uri $url -OutFile $out

Write-Host "Done."