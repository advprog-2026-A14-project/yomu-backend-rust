# Multi-stage build for Railway deployment
# Use official Rust image for native Linux build
FROM rust:1.93-bookworm AS chef

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE=true

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
  cargo build --release --bin yomu-backend-rust

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    curl \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --shell /bin/false yomuuser \
    && chown -R yomuuser:yomuuser /app

USER yomuuser

COPY --from=builder /app/target/release/yomu-backend-rust /app/yomu-backend-rust

EXPOSE 8080

ARG IMAGE_SOURCE="https://github.com/advprog-2026-A14-project/yomu-backend-rust"
ARG IMAGE_DESCRIPTION="Yomu Backend Rust - Gamification Engine"
ARG IMAGE_LICENSES="MIT"

LABEL org.opencontainers.image.source="${IMAGE_SOURCE}"
LABEL org.opencontainers.image.description="${IMAGE_DESCRIPTION}"
LABEL org.opencontainers.image.licenses="${IMAGE_LICENSES}"

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/yomu-backend-rust"]