# Multi-stage Rust build
FROM rust:1.82-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY common/ common/
COPY scheduler/ scheduler/
COPY worker/ worker/
COPY api/ api/
COPY cli/ cli/

RUN cargo build --release

# ─── Scheduler Image ───
FROM debian:bookworm-slim AS scheduler
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/flowforge-scheduler /usr/local/bin/
ENTRYPOINT ["flowforge-scheduler"]

# ─── Worker Image ───
FROM debian:bookworm-slim AS worker
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/flowforge-worker /usr/local/bin/
ENTRYPOINT ["flowforge-worker"]

# ─── API Image ───
FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/flowforge-api /usr/local/bin/
EXPOSE 8080
ENTRYPOINT ["flowforge-api"]

# ─── CLI Image ───
FROM debian:bookworm-slim AS cli
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/flowforge-cli /usr/local/bin/
ENTRYPOINT ["flowforge-cli"]
