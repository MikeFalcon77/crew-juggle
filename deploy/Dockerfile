# Two images for every way of running crew-juggle (docker compose, a single
# VPS, Kubernetes):
#   web     Caddy with the built web app; passes API calls to the server
#           docker build -f deploy/Dockerfile --target web -t crew-juggle-web:local .
#   server  juggle-server, the API (the default target)
#           docker build -f deploy/Dockerfile -t crew-juggle:local .

FROM node:24-bookworm-slim AS web-build
WORKDIR /build/web
COPY web/package*.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
# Google sign-in is configured at build time (public Firebase web config).
# Leave these empty for a demo image: it signs in with demo characters.
# Build args are visible to RUN as environment variables, and Vite reads
# VITE_* from the environment; nothing here reaches the final image.
ARG VITE_FIREBASE_API_KEY=""
ARG VITE_FIREBASE_AUTH_DOMAIN=""
ARG VITE_FIREBASE_PROJECT_ID=""
RUN npm run build

FROM caddy:2-alpine AS web
COPY deploy/web/Caddyfile /etc/caddy/Caddyfile
COPY --from=web-build /build/web/dist /srv
EXPOSE 8080

FROM rust:1.98-bookworm AS rust
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake protobuf-compiler libprotobuf-dev \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY gears ./gears
COPY libs ./libs
ARG JUGGLE_BUILD_COMMIT=""
ENV JUGGLE_BUILD_COMMIT=$JUGGLE_BUILD_COMMIT
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --locked -p juggle-server \
 && cp target/release/juggle-server /build/juggle-server

FROM debian:bookworm-slim AS server
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=rust /build/juggle-server /usr/local/bin/juggle-server
COPY config /app/config
USER app
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=6 \
  CMD curl -fsS http://127.0.0.1:8080/readyz || exit 1
ENTRYPOINT ["/usr/local/bin/juggle-server"]
