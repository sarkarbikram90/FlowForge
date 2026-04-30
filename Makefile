.PHONY: up down build test clean dev seed

# Build all Rust binaries
build:
	cargo build --release

# Run tests
test:
	cargo test --workspace

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
	docker compose up -d postgres redis

# Run API locally (requires dev-infra)
dev-api:
	cargo run --bin flowforge-api

# Run scheduler locally (requires dev-infra)
dev-scheduler:
	cargo run --bin flowforge-scheduler

# Run worker locally (requires dev-infra)
dev-worker:
	cargo run --bin flowforge-worker

# Seed example DAGs via CLI
seed:
	cargo run --bin flowforge-cli -- submit --file examples/simple-hello.yaml
	cargo run --bin flowforge-cli -- submit --file examples/etl-pipeline.yaml
	cargo run --bin flowforge-cli -- submit --file examples/retry-demo.yaml
	@echo "DAGs seeded successfully!"

# Trigger a sample run
run-hello:
	cargo run --bin flowforge-cli -- trigger simple-hello

# View system status
status:
	cargo run --bin flowforge-cli -- status

# View logs
logs:
	docker compose logs -f
