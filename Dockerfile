# Stage 1: Build SvelteKit static assets
FROM oven/bun:1.2 AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend/ .
RUN bun run build

# Stage 2: Build Rust binary
FROM rust:1.87-slim AS backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --workspace

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN useradd --system --no-create-home arra
WORKDIR /app

COPY --from=backend /app/target/release/arra-export /app/arra-export
COPY --from=frontend /app/frontend/build /app/frontend/build

USER arra
ENV ARRA_EXPORT_PORT=4778
ENV ARRA_EXPORT_FRONTEND_DIST=/app/frontend/build
EXPOSE 4778

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -sf http://localhost:4778/api/health || exit 1

ENTRYPOINT ["/app/arra-export"]
