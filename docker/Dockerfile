# Build the Frontend Web UI
FROM node:20-alpine AS web-builder
WORKDIR /app/web
# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm run build

# Build the Rust Backend
FROM rust:1.80-slim-bookworm AS rust-builder
WORKDIR /app
# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY bin/ bin/
# Build production release
RUN cargo build --release --bin barqflow-server

# Final Runtime Image
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy Rust Binary
COPY --from=rust-builder /app/target/release/barqflow-server /usr/local/bin/barqflow-server
# Copy Vue Frontend Dist Dist
COPY --from=web-builder /app/web/dist /app/web/dist

ENV RUST_LOG="info,barqflow=debug"
ENV PORT=3000
EXPOSE 3000

CMD ["barqflow-server"]
