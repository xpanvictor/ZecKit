FROM rust:1.85-slim-bookworm

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    protobuf-compiler \
    libssl-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Clone and build zingolib
WORKDIR /build
RUN git clone https://github.com/zingolabs/zingolib.git && \
    cd zingolib && \
    cargo build --release --package zingo-cli && \
    mv target/release/zingo-cli /usr/local/bin/ && \
    chmod +x /usr/local/bin/zingo-cli

# Create data directory
RUN mkdir -p /var/zingo && chmod 777 /var/zingo

WORKDIR /var/zingo

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]