# Use Debian Bookworm for consistent glibc version
FROM debian:bookworm-slim AS chef

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && curl -sL https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init | sh -s -- -y --default-toolchain 1.75.0

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-chef

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE=true

# Build for musl (static linking) to avoid glibc version issues
RUN cargo build --release --target x86_64-unknown-linux-musl --bin yomu-backend-rust

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash yomuuser \
    && chown -R yomuuser:yomuuser /app

USER yomuuser

# Copy musl statically linked binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/yomu-backend-rust /app/yomu-backend-rust

COPY --from=builder /app/.env.example /app/.env.example

EXPOSE 8080

CMD ["/app/yomu-backend-rust"]
