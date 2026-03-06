# Build the Frontend Web UI
FROM node:20-alpine AS web-builder
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Build the Rust Backend
FROM rust:1.88-slim-bookworm AS rust-builder
WORKDIR /app
# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY bin/ bin/
RUN cargo build --release --bin barqflow

# Final Runtime Image
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy Rust Binary
COPY --from=rust-builder /app/target/release/barqflow /usr/local/bin/barqflow
# Copy Vue Frontend Dist
COPY --from=web-builder /app/web/dist /app/web/dist

ENV RUST_LOG="info,barqflow=debug"
ENV PORT=3000
EXPOSE 3000

CMD ["barqflow"]
