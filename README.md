# ALLG Trainer

## Local frontend development

For live frontend editing against the Docker backend (PostgreSQL + Rocket API):

```bash
./dev.sh
```

This ensures the Docker backend is running on `http://127.0.0.1:8000` (with
`style/` bind-mounted via `docker-compose.dev.yml` so CSS edits apply
immediately), waits for `/api/trainers` to respond, and launches `trunk serve`
on `http://127.0.0.1:8080` with `/api`, `/catalog`, `/style`, and icon/manifest
requests proxied to the backend.

- Pass `--build` to rebuild the Docker backend image first (`./dev.sh --build`).
- Pass `--open` to open the browser automatically (`./dev.sh --open`).
- If the Docker backend is already running on port `8000`, `trunk serve` from
  the repository root also uses the proxy settings in `Trunk.toml` directly.

## Docker deployment

1. Copy `.env.example` to `.env` and set the WCA OAuth application credentials.
2. Register `http://localhost:8000/api/auth/wca/callback` (or the public equivalent) as the WCA OAuth redirect URI.
3. Run `docker compose up --build -d`.
4. Open `http://localhost:8000`.

The stack starts PostgreSQL, a one-shot database bootstrap service, and then the
Rocket server. SeaORM migrations and trainer catalog imports finish successfully
before the server starts accepting connections. Follow progress with
`docker compose logs -f alg-trainers-migrate`. Set
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
- Docker Compose runs migrations and catalog initialization in a dedicated
  one-shot service so multiple web replicas do not perform startup DDL.
- Trainer cases, groups, algsets, algorithms, scrambles, and SVG diagrams are
  imported into PostgreSQL from the checked-in `static/` catalog on initial startup.
- Browser state remains in IndexedDB under the existing trainer storage keys.
  A debounced, rate-limited sync engine sends dirty settings, selections,
  presets, custom algorithms, and times to `/api/sync`.
  Hold down delay and base, timer, and scramble sizes stay on each device
  instead of syncing; they are still included in backups.
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

## Database environment variables

`DATABASE_URL` selects PostgreSQL. Pool behavior is configurable per process:

- `DATABASE_MAX_CONNECTIONS` (default `20`)
- `DATABASE_MIN_CONNECTIONS` (default `2`)
- `DATABASE_CONNECT_TIMEOUT_SECS` (default `5`)
- `DATABASE_ACQUIRE_TIMEOUT_SECS` (default `5`)
- `DATABASE_IDLE_TIMEOUT_SECS` (default `300`)
- `DATABASE_MAX_LIFETIME_SECS` (default `1800`)

Set the connection maximum low enough to leave PostgreSQL capacity for every
replica plus operational headroom. Standalone server runs migrate by default;
set `DATABASE_RUN_MIGRATIONS=false` when bootstrap is managed separately.
`DATABASE_MIGRATE_ONLY=true` runs migrations and catalog initialization and exits.
