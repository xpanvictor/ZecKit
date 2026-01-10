# ========================================
# Builder Stage
# ========================================
FROM rust:1.85-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the real application
RUN touch src/main.rs && \
    cargo build --release

# ========================================
# Runtime Stage
# ========================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 2001 -s /bin/bash faucet

# Copy binary from builder
COPY --from=builder /build/target/release/zeckit-faucet /usr/local/bin/faucet
RUN chmod +x /usr/local/bin/faucet

# Create data directory
RUN mkdir -p /var/zingo && chown -R faucet:faucet /var/zingo

WORKDIR /var/zingo

# Switch to non-root user
USER faucet

EXPOSE 8080

# Set default environment
ENV RUST_LOG=info
ENV ZINGO_DATA_DIR=/var/zingo

# Health check
HEALTHCHECK --interval=30s --timeout=10s --retries=3 --start-period=90s \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["faucet"]
