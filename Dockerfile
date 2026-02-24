FROM rust:1.93-slim AS chef

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --bin yomu-backend-rust

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*


RUN useradd -ms /bin/bash yomuuser \
    && chown -R yomuuser:yomuuser /app

USER yomuuser


COPY --from=builder /app/target/release/yomu-backend-rust /app/yomu-backend-rust


COPY --from=builder /app/.env.example /app/.env.example

EXPOSE 8080

CMD ["/app/yomu-backend-rust"]