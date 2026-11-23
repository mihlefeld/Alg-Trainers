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

## Backend security configuration

The production backend requires explicit `POSTGRES_PASSWORD` and
`ROCKET_SECRET_KEY` values. Generate a Rocket-compatible high-entropy secret
with `openssl rand -base64 32` and keep it stable across deployments.
`COOKIE_SECURE` defaults automatically from the callback URL; set it to `false`
for local HTTP development and to `true` behind production HTTPS. An explicit
`true` is ignored for an HTTP callback because browsers do not return `Secure`
OAuth state cookies over localhost HTTP.

The application applies per-IP API rate limits and a 1 MiB JSON request limit.
Deployments should additionally configure proxy-level connection, request, and
IP-aware rate limits.

## Architecture

- Rocket serves the Yew WASM application and all authenticated APIs.
- SeaORM migrations create and update the PostgreSQL schema.
- Trainer cases, groups, algsets, algorithms, scrambles, and SVG diagrams are
  imported into PostgreSQL from the checked-in `static/` catalog on initial startup.
- Browser state remains in IndexedDB under the existing trainer storage keys.
  A debounced, rate-limited sync engine sends dirty settings, selections,
  presets, custom algorithms, and times to `/api/sync`.
- The runtime image does not expose the previous JavaScript trainer routes.

## Cube colors

Trainer settings provide built-in default and color-blind palettes. Copy a
palette to create an editable named scheme, then assign it globally or override
it for a puzzle or individual trainer. Trainer assignments take precedence over
puzzle assignments, which take precedence over the global scheme. Deleting a
custom scheme clears its assignments. Schemes are included in settings sync
and trainer backups; previous custom cube colors are retained during migration.

## WCA OAuth environment variables

- `WCA_BASE_URL`
- `WCA_CLIENT_ID`
- `WCA_CLIENT_SECRET`
- `WCA_REDIRECT_URL`
