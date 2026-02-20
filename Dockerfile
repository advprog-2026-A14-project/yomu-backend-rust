# Multi-stage build for Yomu Engine Rust
# Stage 1: Builder - Compile the application
FROM rust:1.93-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml ./
COPY Cargo.lock ./

# Create dummy src to build dependencies first
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Production Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/yomu-engine-rust /app/yomu-engine-rust

# Copy .env.example as default env file
COPY --from=builder /app/.env.example /app/.env.example

# Expose port
EXPOSE 8080

# Run the application
CMD ["/app/yomu-engine-rust"]