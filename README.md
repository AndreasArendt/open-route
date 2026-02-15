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
