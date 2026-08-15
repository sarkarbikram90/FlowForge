# ─── Multi-Stage Dockerfile for FlowForge Platform ───

# Stage 1: Build Rust Binaries
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static perl make

WORKDIR /usr/src/flowforge

COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates

RUN cargo build --release --workspace

# Stage 2: CLI
FROM alpine:3.20 AS cli
RUN apk add --no-cache ca-certificates libgcc
WORKDIR /app
COPY --from=builder /usr/src/flowforge/target/release/flowforge-cli /app/flowforge
ENTRYPOINT ["/app/flowforge"]

# Stage 3: Scheduler
FROM alpine:3.20 AS scheduler
RUN apk add --no-cache ca-certificates libgcc
WORKDIR /app
COPY --from=builder /usr/src/flowforge/target/release/flowforge-scheduler /app/flowforge-scheduler
ENTRYPOINT ["/app/flowforge-scheduler"]

# Stage 4: Worker
FROM alpine:3.20 AS worker
RUN apk add --no-cache ca-certificates libgcc python3 bash curl docker-cli
WORKDIR /app
COPY --from=builder /usr/src/flowforge/target/release/flowforge-worker /app/flowforge-worker
ENTRYPOINT ["/app/flowforge-worker"]

# Stage 5: API (Default Final Stage)
FROM alpine:3.20 AS api
RUN apk add --no-cache ca-certificates libgcc curl
WORKDIR /app
COPY --from=builder /usr/src/flowforge/target/release/flowforge-api /app/flowforge-api
EXPOSE 8080
ENTRYPOINT ["/app/flowforge-api"]
