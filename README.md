# ALLG Trainer

## Docker deployment

1. Copy `.env.example` to `.env` and set the WCA OAuth application credentials.
2. Register `http://localhost:8000/api/auth/wca/callback` (or the public equivalent) as the WCA OAuth redirect URI.
3. Run `docker compose up --build -d`.
4. Open `http://localhost:8000`.

The stack starts PostgreSQL and the Rocket server. SeaORM migrations run when the server starts.
Trainer definitions are imported into PostgreSQL before the server starts
accepting connections on the first startup. Follow progress with
`docker compose logs -f alg-trainers-server`. Set
`FORCE_CATALOG_SEED=1` when intentionally rebuilding catalog content from the
checked-in trainer JSON files.

Docker dependency layers are split for the native backend and WASM frontend and
reuse BuildKit cache mounts. Keep `Cargo.lock` committed when changing Rust
dependencies so the toolchain and resolved crate graph remain reproducible.

## Architecture

- Rocket serves the Yew WASM application and all authenticated APIs.
- SeaORM migrations create and update the PostgreSQL schema.
- Trainer cases, groups, algsets, algorithms, scrambles, and SVG diagrams are
  imported into PostgreSQL from the checked-in catalog on initial startup.
- Browser state remains in IndexedDB under the existing trainer storage keys.
  A debounced, rate-limited sync engine sends dirty settings, selections,
  presets, custom algorithms, and times to `/api/sync`.
- The runtime image does not expose the previous JavaScript trainer routes.

## WCA OAuth environment variables

- `WCA_BASE_URL`
- `WCA_CLIENT_ID`
- `WCA_CLIENT_SECRET`
- `WCA_REDIRECT_URL`
