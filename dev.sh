#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

build_backend=false
start_backend=true
frontend_port="${FRONTEND_PORT:-8080}"
trunk_args=()

while (($# > 0)); do
    case "$1" in
        --build)
            build_backend=true
            shift
            ;;
        --no-backend)
            start_backend=false
            shift
            ;;
        --port)
            if (($# < 2)); then
                echo "Missing value for --port" >&2
                exit 1
            fi
            frontend_port="$2"
            shift 2
            ;;
        -h|--help)
            cat <<'EOF'
Usage: ./dev.sh [options] [-- trunk serve options]

Starts the Docker backend (PostgreSQL + Rocket server with live-mounted style/)
and launches `trunk serve` with live reload and backend proxying.

Options:
  --build         Rebuild the Docker backend image before starting
  --no-backend    Skip starting Docker Compose (assume backend is already running)
  --port <port>   Frontend dev server port (default: 8080 or $FRONTEND_PORT)
  --open          Open the browser automatically (passed through to trunk serve)
  -h, --help      Show this help message
EOF
            exit 0
            ;;
        *)
            trunk_args+=("$1")
            shift
            ;;
    esac
done

export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v trunk >/dev/null 2>&1; then
    echo "Error: 'trunk' is not installed or not in PATH." >&2
    echo "Install it with: cargo install --locked trunk (or download v0.22.2 from GitHub releases)" >&2
    exit 1
fi

backend_port="${ALG_TRAINERS_PORT:-}"
if [[ -z "$backend_port" && -f .env ]]; then
    backend_port="$(grep -E '^ALG_TRAINERS_PORT=' .env | tail -n 1 | cut -d= -f2- | tr -d '[:space:]"'"'" || true)"
fi
backend_port="${backend_port:-8000}"

if [[ "$start_backend" == true ]]; then
    if [[ ! -f .env ]]; then
        echo "Error: .env not found. Copy .env.example to .env and set POSTGRES_PASSWORD and ROCKET_SECRET_KEY." >&2
        exit 1
    fi

    compose_args=(-f docker-compose.yml -f docker-compose.dev.yml up -d)
    if [[ "$build_backend" == true ]]; then
        compose_args+=(--build)
    fi

    echo "Starting Docker backend on http://127.0.0.1:${backend_port}..."
    docker compose "${compose_args[@]}"

    echo -n "Waiting for backend catalog API"
    ready=false
    for _ in $(seq 1 60); do
        if curl -fsS -o /dev/null "http://127.0.0.1:${backend_port}/api/trainers" 2>/dev/null; then
            ready=true
            break
        fi
        echo -n "."
        sleep 1
    done
    echo

    if [[ "$ready" != true ]]; then
        echo "Error: backend did not become ready at http://127.0.0.1:${backend_port}/api/trainers" >&2
        echo "Check logs with: docker compose logs -f alg-trainers-migrate alg-trainers-server" >&2
        exit 1
    fi
fi

config_path="$repo_root/Trunk.toml"
tmp_config=""
cleanup() {
    if [[ -n "$tmp_config" && -f "$tmp_config" ]]; then
        rm -f "$tmp_config"
    fi
}
trap cleanup EXIT

if [[ "$backend_port" != "8000" ]]; then
    tmp_config="$(mktemp "${TMPDIR:-/tmp}/alg-trainers-trunk.XXXXXX.toml")"
    sed "s|http://127.0.0.1:8000|http://127.0.0.1:${backend_port}|g" "$repo_root/Trunk.toml" > "$tmp_config"
    config_path="$tmp_config"
fi

echo "Starting Trunk live dev server on http://127.0.0.1:${frontend_port} (proxying backend http://127.0.0.1:${backend_port})..."
trunk serve \
    --config "$config_path" \
    --dist "$repo_root/target/trunk-dist" \
    --port "$frontend_port" \
    "${trunk_args[@]}"
