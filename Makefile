.PHONY: help build test test-chaos check fmt fmt-check lint clippy ui-check ui-test ui-build up down clean dev-infra dev-api dev-scheduler dev-worker dev-ui seed run-sample status helm-lint docker-build

# Default target
help:
	@echo "FlowForge Build & CI/CD Tooling"
	@echo "  make fmt            - Format all Rust code"
	@echo "  make fmt-check      - Verify Rust code formatting"
	@echo "  make clippy         - Run Clippy with -D warnings"
	@echo "  make check          - Check Rust workspace compilation"
	@echo "  make test           - Run full workspace tests"
	@echo "  make test-chaos     - Run chaos and fault-injection tests"
	@echo "  make ui-check       - Run TypeScript typecheck on UI"
	@echo "  make ui-test        - Run Vitest component tests on UI"
	@echo "  make ui-build       - Build UI production bundle"
	@echo "  make build          - Build release binaries for workspace"
	@echo "  make docker-build   - Build multi-target Docker images"
	@echo "  make helm-lint      - Lint FlowForge Helm charts"
	@echo "  make up             - Start complete stack in Docker Compose"
	@echo "  make down           - Stop Docker Compose stack"

# Format all Rust code
fmt:
	cargo fmt --all

# Verify Rust code formatting
fmt-check:
	cargo fmt --all -- --check

# Run Clippy with zero warnings tolerated
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check workspace compilation
check:
	cargo check --workspace --all-targets --all-features

# Build all workspace binaries in release mode
build:
	cargo build --release --workspace

# Run all workspace unit and integration tests
test:
	cargo test --workspace

# Run chaos and failure injection tests
test-chaos:
	cargo test -p flowforge-chaos-tests -- --nocapture

# Frontend UI Typecheck
ui-check:
	cd ui && npm run typecheck

# Frontend UI Tests
ui-test:
	cd ui && npm test

# Frontend UI Production Build
ui-build:
	cd ui && npm run build

# Docker Build
docker-build:
	docker build -t flowforge:latest -f Dockerfile .

# Lint Helm Charts
helm-lint:
	helm lint deploy/helm/flowforge

# Start full stack with Docker Compose
up:
	docker compose up --build -d

# Stop all services
down:
	docker compose down

# Clean up everything including volumes
clean:
	docker compose down -v
	cargo clean

# Start infrastructure only (for local dev)
dev-infra:
	docker compose up -d postgres nats minio

# Run API locally
dev-api:
	cargo run -p flowforge-api

# Run scheduler locally
dev-scheduler:
	cargo run -p flowforge-scheduler

# Run worker locally
dev-worker:
	cargo run -p flowforge-worker

# Run Frontend UI
dev-ui:
	cd ui && npm run dev

# Seed example DAGs via CLI
seed:
	cargo run -p flowforge-cli -- workflow apply --file examples/daily-etl-pipeline.yaml
	cargo run -p flowforge-cli -- workflow apply --file examples/k8s-model-training.yaml
	cargo run -p flowforge-cli -- workflow apply --file examples/security-compliance-audit.yaml
	@echo "Workflows seeded successfully!"

# Trigger sample workflow run
run-sample:
	cargo run -p flowforge-cli -- run trigger daily-etl-pipeline

# View system status
status:
	cargo run -p flowforge-cli -- status
