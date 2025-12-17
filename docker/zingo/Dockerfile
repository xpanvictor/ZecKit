# docker/zaino/Dockerfile
FROM rust:1.85-slim-bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    curl \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

# Clone and build Zaino indexer
WORKDIR /build
RUN git clone https://github.com/zingolabs/zaino.git && \
    cd zaino && \
    cargo build --release --bin zainod && \
    mv target/release/zainod /usr/local/bin/ && \
    chmod +x /usr/local/bin/zainod

# Create data directory
RUN mkdir -p /var/zaino && chmod 777 /var/zaino

WORKDIR /var/zaino

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]