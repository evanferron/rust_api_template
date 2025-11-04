# Multi-stage Dockerfile for building a Rust API
# Stage 1: build the binary
FROM rust:1.72-slim AS builder

# Install system dependencies needed to compile common crates
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       pkg-config \
       libssl-dev \
       ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Cache dependencies: copy manifest files and build a dummy binary to cache cargo registry
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() { println!(\"build-deps\"); }" > src/main.rs
RUN cargo build --release || true

# Copy the full source and build the release binary
COPY . .
RUN cargo build --release --locked

# Stage 2: create a small runtime image
FROM debian:bookworm-slim

# Install ca-certificates for TLS support at runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for better security
RUN useradd -m -u 1000 appuser

WORKDIR /usr/local/bin

# Copy the statically-built binary from the builder (release binary name = package name)
COPY --from=builder /usr/src/app/target/release/rust_api_template /usr/local/bin/rust_api_template
RUN chown appuser:appuser /usr/local/bin/rust_api_template

USER appuser

# Default port - adjust if your server uses another port
EXPOSE 8080

# Recommended environment variables (can also be provided through docker-compose or .env)
ENV RUST_LOG=info

# Run the binary
CMD ["/usr/local/bin/rust_api_template"]

