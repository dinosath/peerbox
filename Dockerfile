ARG RUST_VERSION=1

FROM rust:${RUST_VERSION}-slim-bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY tests/ tests/

ARG APP_VERSION=0.1.0

RUN cargo build --release -p peerbox-server -p peerbox-cli

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN adduser \
    --disabled-password \
    --gecos "" \
    --uid 10001 \
    peerbox

COPY --from=builder /app/target/release/peerbox-server /usr/local/bin/peerbox-server
COPY --from=builder /app/target/release/peerbox /usr/local/bin/peerbox

ENV LISTEN_ADDR=0.0.0.0:8080

USER peerbox

EXPOSE 8080

ENTRYPOINT ["peerbox-server"]
