# Dockerfile
FROM rust:1.80-slim AS builder
WORKDIR /usr/src/app
COPY . .
# RUN cargo build --release

FROM debian:bookworm-slim
# COPY --from=builder /usr/src/app/target/release/barqflow /usr/local/bin/
CMD ["barqflow"]
