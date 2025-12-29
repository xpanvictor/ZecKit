# Custom Zebra build with internal-miner for regtest
FROM rust:1.80-bookworm as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    libclang-dev \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Clone Zebra
WORKDIR /build
RUN git clone https://github.com/ZcashFoundation/zebra.git
WORKDIR /build/zebra

# Build with internal-miner feature (with cache)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/zebra/target \
    cargo build --release --features internal-miner --bin zebrad && \
    cp target/release/zebrad /tmp/zebrad

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy zebrad binary
COPY --from=builder /tmp/zebrad /usr/local/bin/zebrad

# Create directories
RUN mkdir -p /var/zebra/state /root/.cache/zebra

# Copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /var/zebra

EXPOSE 8232 8233

ENTRYPOINT ["/entrypoint.sh"]
CMD ["zebrad", "start"]