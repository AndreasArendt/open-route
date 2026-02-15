# open-route

`open-route` is a local, Docker-based bike routing backend.

It consists of:

- **GraphHopper** (Java) – routing engine using OpenStreetMap data  
- **Planner** (Rust / Actix-web) – API layer that forwards and later enhances route requests  

This project is the foundation for building a Komoot-style route suggestion system.

---

## Requirements

- Docker Desktop (with Compose v2)

---

## Setup

1. Place a Bavaria OSM extract at:

   `./data/osm/bavaria.osm.pbf`

2. Start the services:

   ```bash
   docker compose up --build
   ```

3. Verify the services:

   ```bash
   curl http://localhost:8989/health
   curl http://localhost:8080/health
   ```

4. Request a route:

   ```bash
   curl "http://localhost:8080/route?start=48.137154,11.576124&end=48.370545,10.897790"
   ```

## Optional: Download OSM Data

You can use the helper scripts to download Bavaria data automatically:

```bash
./scripts/setup.sh
```

On Windows PowerShell:

```powershell
./scripts/setup.ps1
```

## Notes

- GraphHopper can take additional time on the first run while route graph data is prepared.
- Graph cache is persisted in the Docker volume `gh-cache`.
