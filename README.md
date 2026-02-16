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

4. Request a direct route:

   ```bash
   curl "http://localhost:8080/route?start=48.137154,11.576124&end=48.370545,10.897790"
   ```

5. Request ranked suggestions from preferences:

   ```bash
   curl -X POST http://localhost:8080/suggestions \
     -H "Content-Type: application/json" \
     -d '{
       "start":"48.137154,11.576124",
       "end":"48.370545,10.897790",
       "max_suggestions":3,
       "preferences":{
         "fitness_level":0.4,
         "scenic_preference":0.8,
         "avoid_main_roads":0.8,
         "time_priority":0.3
       }
     }'
   ```

6. Open the UI:

   - [http://localhost:5173](http://localhost:5173)
   - Suggestions are plotted directly on an interactive map.
   - Click a route card or route chip to focus that alternative on the map.

## IDE-First Development (Recommended)

If you want simpler debugging in your IDE, run only GraphHopper in Docker and run Planner/UI locally.

### Option A: Single command

```bash
./scripts/dev.sh
```

This does:
- `docker compose up -d graphhopper`
- `docker compose stop planner ui` (avoids local port conflicts)
- `cargo run` in `planner` (with `GH_BASE_URL=http://localhost:8989`)
- `npm run dev` in `ui`

Stop GraphHopper when you are done:

```bash
./scripts/dev-stop.sh
```

### Option B: Manual (best for breakpoints)

1. Start only GraphHopper:

   ```bash
   docker compose up -d graphhopper
   ```

2. In your IDE, run/debug Planner from `/planner`:

   ```bash
   cargo run
   ```

3. In your IDE terminal, start UI from `/ui`:

   ```bash
   npm install
   npm run dev -- --host 0.0.0.0 --port 5173
   ```

4. Open [http://localhost:5173](http://localhost:5173)

All `println!`/`dbg!` output from planner appears directly in your IDE run/debug console.

## Full-Docker Auto-Update (Optional)

If you prefer running planner + UI in Docker, use Compose watch mode:

1. Start the stack:

   ```bash
   docker compose up --build -d
   ```

2. Start file watching in a second terminal:

   ```bash
   docker compose watch
   ```

What happens on changes:
- `planner/src` and planner build files trigger a `planner` image rebuild.
- `ui/src` and `ui/index.html` sync directly into the running UI container (Vite hot reload).
- `ui/package.json` or `ui/Dockerfile` triggers a `ui` image rebuild.

## Planner API

- `GET /health`
- `GET /route?start=lat,lon&end=lat,lon`
- `POST /suggestions`
  - `max_suggestions`: `1..6`
  - `preferences.fitness_level`: `0..1`
  - `preferences.scenic_preference`: `0..1`
  - `preferences.avoid_main_roads`: `0..1`
  - `preferences.time_priority`: `0..1`

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
