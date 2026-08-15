# FlowForge — Production-Grade CI/CD & Engineering Pipeline Report

This document records the complete architecture, implementation details, security governance, and operational workflows of the **FlowForge Continuous Integration and Continuous Delivery (CI/CD) system**.

---

## 1. Executive Summary

FlowForge has transitioned from a single baseline workflow into a multi-tier, enterprise-grade CI/CD and distribution pipeline. All pipelines are **100% operational, validated, and passing on GitHub Actions**.

```mermaid
graph TD
    A[Git Push / PR / Tag] --> B[Workflow Engine Router]
    
    B --> C[CI Pipeline (.github/workflows/ci.yml)]
    B --> D[Security & Supply Chain (.github/workflows/security.yml)]
    B --> E[E2E & Integration (.github/workflows/e2e.yml)]
    B --> F[Release & Distribution (.github/workflows/release.yml)]
    
    subgraph "CI Pipeline"
        C --> C1[cargo fmt --check]
        C --> C2[cargo clippy -D warnings]
        C --> C3[cargo check --all-targets --all-features]
        C --> C4[cargo test --workspace Matrix: Linux, Windows, macOS]
        C --> C5[Frontend Typecheck & Vitest: tsc, vitest, vite build]
        C --> C6[Helm Linting & Dry-Run Template: deploy/helm/flowforge]
    end

    subgraph "Security & Supply Chain"
        D --> D1[Cargo Audit: RustSec Advisory Database]
        D --> D2[Cargo Deny: License Compliance & Dependency Bans]
        D --> D3[NPM Audit: Zero High/Critical Vulnerabilities]
        D --> D4[Gitleaks: Secret & Credential Scanning]
        D --> D5[Trivy: Multi-Stage Container CVE Scanner]
    end

    subgraph "E2E & Integration"
        E --> E1[Docker Compose: PostgreSQL 16 + NATS JetStream + MinIO]
        E --> E2[Database Migrations: migrations/*.sql]
        E --> E3[Live DB & Messaging Test Suites]
        E --> E4[Container Smoke Test: HTTP Health Probe /api/v1/health/live]
    end

    subgraph "Release (v*.*.*)"
        F --> F1[Cross-Platform Binaries: Linux, Windows, macOS]
        F --> F2[GHCR Multi-Arch Container Images: amd64, arm64]
        F --> F3[Helm Chart Packaging: flowforge-v*.tgz]
        F --> F4[GitHub Release Publication + SHA256SUMS.txt]
    end
```

---

## 2. Pipeline Workflows

### A. Main CI Pipeline (`.github/workflows/ci.yml`)
- **Triggers**: `push` to `main`, `pull_request` to `main`, `workflow_dispatch`.
- **Rust Quality Assurance**:
  - `cargo fmt --all -- --check` (Strict formatting verification).
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` (Zero warnings policy).
  - `cargo check --workspace --all-targets --all-features` (Full workspace compilation).
- **Cross-Platform Matrix Testing**:
  - Matrix across `ubuntu-latest`, `windows-latest`, and `macos-latest`.
  - Runs all workspace unit tests and chaos fault-injection suites (`flowforge-chaos-tests`).
- **Frontend UI Quality**:
  - Strict TypeScript validation (`tsc --noEmit`).
  - Component and UI unit testing via `vitest` + `jsdom`.
  - Production Vite bundle compilation (`npm run build`).
- **Helm Chart Validation**:
  - Linting (`helm lint deploy/helm/flowforge`) and dry-run rendering (`helm template flowforge deploy/helm/flowforge --debug`).

---

### B. Security & Supply Chain (`.github/workflows/security.yml`)
- **Triggers**: `push` to `main`, `pull_request` to `main`, daily cron (`0 2 * * *`), `workflow_dispatch`.
- **Permissions**: `contents: read`, `security-events: write`, `checks: write`.
- **RustSec Cargo Audit**:
  - Audits dependencies against the RustSec Advisory Database.
  - Configured with explicit advisory exception rules in `audit.toml` and `.cargo/audit.toml`.
- **Cargo Deny Governance (`deny.toml`)**:
  - Enforces canonical SPDX license allowlists (`MIT`, `Apache-2.0`, `MPL-2.0`, `BSD-3-Clause`, `ISC`, `Unicode-3.0`, `OpenSSL`, `Zlib`, `CC0-1.0`).
  - Detects duplicate dependencies and bans unapproved sources.
- **Frontend Security Audit**:
  - `npm audit --audit-level=high` ensuring 0 high/critical vulnerabilities.
  - Dependencies upgraded to `vite@8.2.1` and `esbuild@0.28.2` (GHSA-67mh-4wv8-2f99 resolved).
- **Gitleaks Secret Scanning**:
  - Automated scanning of repository commit history for accidentally leaked tokens, API keys, or certificates.
- **Trivy Container Scanning**:
  - Scans multi-stage production container images for OS and library CVEs with table and SARIF reporting.

---

### C. End-to-End & Integration Testing (`.github/workflows/e2e.yml`)
- **Triggers**: `push` to `main`, `pull_request` to `main`, `workflow_dispatch`.
- **Deterministic Backing Services**:
  - Bootstraps real backing infrastructure via `docker compose up -d postgres nats minio`:
    - **PostgreSQL 16 Alpine** (with readiness healthchecks via `pg_isready -h 127.0.0.1 -U flowforge`).
    - **NATS JetStream 2.10** (with persistent streams enabled `-js`).
    - **MinIO S3 Object Storage** (`server /data`).
- **In-Container Database Schema Migrations**:
  - Applies versioned SQL migrations (`migrations/*.sql`) directly inside the PostgreSQL container:
    ```bash
    cat "$migration" | docker compose exec -T -e PGPASSWORD=flowforge postgres psql -h 127.0.0.1 -U flowforge -d flowforge
    ```
- **Live Integration & Chaos Tests**:
  - Runs end-to-end multi-step DAG execution, leader failover elections, and stale worker lease recovery sweeps.
- **Container HTTP Health Probe Smoke Test**:
  - Builds the production API container image (`docker build --target api -t flowforge-smoke:latest -f Dockerfile .`).
  - Boots the container and polls `/api/v1/health/live` to verify runtime health.

---

### D. Multi-Architecture Release Pipeline (`.github/workflows/release.yml`)
- **Triggers**: Tag push matching `v*.*.*`, `workflow_dispatch`.
- **Permissions**: `contents: write`, `packages: write`, `id-token: write`.
- **Cross-Platform Binary Compilation**:
  - **Linux x86_64** (`x86_64-unknown-linux-gnu`)
  - **Linux ARM64** (`aarch64-unknown-linux-gnu`) via `cross`
  - **Windows x86_64** (`x86_64-pc-windows-msvc`)
  - **macOS Intel** (`x86_64-apple-darwin`)
  - **macOS Apple Silicon** (`aarch64-apple-darwin`)
- **Multi-Architecture Container Publishing**:
  - Builds and publishes multi-platform container images (`linux/amd64`, `linux/arm64`) to `ghcr.io/sarkarbikram90/flowforge`.
- **Helm OCI Packaging**:
  - Packages and uploads versioned Helm charts (`flowforge-<version>.tgz`).
- **Release Assets & Checksums**:
  - Generates automated release notes, publishes binary tarballs/zip archives, and attaches `SHA256SUMS.txt`.

---

## 3. Docker Multi-Stage Architecture

The [`Dockerfile`](../Dockerfile) uses a multi-stage Alpine build with musl static linking:

1. **Stage 1: `builder` (`rust:alpine`)**:
   - Installs `musl-dev`, `pkgconfig`, `openssl-dev`, `openssl-libs-static`, `perl`, and `make`.
   - Compiles all workspace binaries in release mode (`cargo build --release --workspace`).
2. **Stage 2: `cli` (`alpine:3.20`)**:
   - Contains `/app/flowforge` (the `flowforge-cli` management binary).
3. **Stage 3: `scheduler` (`alpine:3.20`)**:
   - Contains `/app/flowforge-scheduler` (HA leader election and progression engine).
4. **Stage 4: `worker` (`alpine:3.20`)**:
   - Contains `/app/flowforge-worker` with `python3`, `bash`, `curl`, and `docker-cli`.
5. **Stage 5: `api` (`alpine:3.20` - Default Final Stage)**:
   - Contains `/app/flowforge-api` (HTTP/REST/SSE Gateway, port 8080).

---

## 4. Local Development & Validation Commands

All CI/CD steps can be executed locally using the repository [`Makefile`](../Makefile):

```bash
# Format & Lint
make fmt-check
make clippy

# Build & Test
make check
make test
make test-chaos

# Frontend Checks
make ui-check
make ui-test
make ui-build

# Infrastructure & Smoke Testing
make helm-lint
make docker-build
make up
```

---

## 5. Pipeline Validation Results

| Test / Gate | Command / Target | Result |
| :--- | :--- | :--- |
| **Rust Formatter** | `cargo fmt --all -- --check` | **PASSED** (0 deviations) |
| **Strict Clippy** | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **PASSED** (0 warnings) |
| **Workspace Tests** | `cargo test --workspace` | **PASSED** (All unit & chaos tests passed) |
| **Frontend Typecheck** | `npm run typecheck` (`ui/`) | **PASSED** (0 TypeScript errors) |
| **Frontend Tests** | `npm test` (`ui/` Vitest) | **PASSED** (3/3 component tests passed) |
| **Frontend Production Build** | `npm run build` (`ui/` Vite) | **PASSED** (Built in 4.21s) |
| **NPM Audit** | `npm audit --audit-level=high` | **PASSED** (0 vulnerabilities) |
| **Cargo Deny** | `cargo-deny check` | **PASSED** (Licenses & bans compliant) |
| **Container Smoke Test** | `curl -f http://localhost:8080/api/v1/health/live` | **PASSED** (HTTP 200 OK) |
