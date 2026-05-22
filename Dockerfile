# Multi-stage build for yomu-backend-rust
FROM rust:1.93-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --bin yomu-backend-rust

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    curl \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /var/log/yomu /app \
    && addgroup --system --gid 1001 appgroup \
    && adduser --system --uid 1001 --ingroup appgroup --shell /bin/false yomuuser \
    && chown -R yomuuser:appgroup /app /var/log/yomu

USER yomuuser

COPY --from=builder /app/target/release/yomu-backend-rust /app/yomu-backend-rust

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/yomu-backend-rust"]
