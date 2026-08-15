.PHONY: up down build test clean dev seed lint check

# Build all workspace binaries
build:
	cargo build --release --workspace

# Run all workspace unit tests and chaos test suites
test:
	cargo test --workspace

# Check workspace compilation
check:
	cargo check --workspace

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
