# Stage 1: Build (Pake image Rust 1.93 resmi)
FROM rust:1.93-slim AS builder

WORKDIR /app

# Copy manifest dan source code
COPY Cargo.toml ./
# Jika ada Cargo.lock, uncomment baris di bawah
# COPY Cargo.lock ./ 
COPY src ./src

# Build aplikasi untuk release
RUN cargo build --release

# Stage 2: Runtime (Pake image OS yang sangat kecil)
FROM debian:bookworm-slim

WORKDIR /app

# Copy binary hasil build dari stage pertama
# Nama binary biasanya sama dengan nama project di Cargo.toml
COPY --from=builder /app/target/release/yomu-backend-rust .

# Jalankan binary-nya
CMD ["./yomu-backend-rust"]