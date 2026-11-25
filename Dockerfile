FROM rust:latest AS chef
ENV CARGO_BUILD_JOBS=1
RUN --mount=type=cache,id=alg-trainers-chef-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=alg-trainers-chef-git,target=/usr/local/cargo/git \
    cargo install cargo-chef --version 0.1.72 --locked
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/migration/Cargo.toml backend/migration/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml
RUN mkdir -p backend/src backend/migration/src frontend/src \
    && touch backend/src/main.rs backend/migration/src/lib.rs frontend/src/lib.rs
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS frontend-tools
RUN rustup target add wasm32-unknown-unknown
ARG TARGETARCH
RUN set -eu; \
    case "${TARGETARCH:-amd64}" in \
      amd64) target=x86_64-unknown-linux-gnu ;; \
      arm64) target=aarch64-unknown-linux-gnu ;; \
      *) echo "Unsupported architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    curl --fail --location --retry 3 \
      "https://github.com/trunk-rs/trunk/releases/download/v0.22.2/trunk-${target}.tar.gz" \
      | tar -xz -C /usr/local/bin; \
    trunk --version
RUN --mount=type=cache,id=alg-trainers-frontend-target,target=/app/target \
    --mount=type=cache,id=alg-trainers-frontend-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=alg-trainers-frontend-git,target=/usr/local/cargo/git \
    cargo chef cook --recipe-path recipe.json --release \
      --target wasm32-unknown-unknown --package alg-trainers-frontend

FROM frontend-tools AS frontend-builder
COPY frontend frontend
COPY Trunk.toml Trunk.toml
RUN --mount=type=cache,id=alg-trainers-frontend-target,target=/app/target \
    --mount=type=cache,id=alg-trainers-frontend-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=alg-trainers-frontend-git,target=/usr/local/cargo/git \
    cd frontend && trunk build --release

FROM chef AS backend-builder
COPY --from=chef /app/recipe.json recipe.json
RUN --mount=type=cache,id=alg-trainers-backend-target,target=/app/target \
    --mount=type=cache,id=alg-trainers-backend-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=alg-trainers-backend-git,target=/usr/local/cargo/git \
    cargo chef cook --recipe-path recipe.json --release --package alg-trainers-server
COPY backend backend
RUN --mount=type=cache,id=alg-trainers-backend-target,target=/app/target \
    --mount=type=cache,id=alg-trainers-backend-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=alg-trainers-backend-git,target=/usr/local/cargo/git \
    cargo build --release -p alg-trainers-server \
    && cp /app/target/release/alg-trainers-server /app/alg-trainers-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-builder /app/alg-trainers-server /app/alg-trainers-server
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
COPY static /app/static
COPY style /app/style
COPY manifest.json icon.png mobile.png /app/
EXPOSE 8000
CMD ["/app/alg-trainers-server"]
