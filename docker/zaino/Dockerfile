# docker/zingo-wallet/Dockerfile
FROM rust:1.85-slim-bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    protobuf-compiler \
    libssl-dev \
    curl \
    git \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

# Clone OFFICIAL zingolib dev branch with the latest fixes
WORKDIR /build
RUN git clone https://github.com/zingolabs/zingolib.git && \
    cd zingolib && \
    git checkout dev && \
    rustup set profile minimal && \
    cargo build --release --package zingo-cli && \
    mv target/release/zingo-cli /usr/local/bin/ && \
    chmod +x /usr/local/bin/zingo-cli

RUN mkdir -p /var/zingo && chmod 777 /var/zingo

WORKDIR /var/zingo

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]