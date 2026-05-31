<div align="center">
  <img src="docs/assets/banner.png" alt="BarqFlow" width="100%" />

  <p><strong>Rust-native workflow automation and orchestration platform</strong></p>
  <p>Visual workflow design, queue-backed execution, workspace-aware operations, and signed extension packs in a single control plane.</p>

  <p>
    <img alt="Rust" src="https://img.shields.io/badge/Rust-1.88%2B-000000?logo=rust" />
    <img alt="Vue" src="https://img.shields.io/badge/Vue-3.5-42b883?logo=vue.js&logoColor=white" />
    <img alt="Axum" src="https://img.shields.io/badge/Axum-0.8-5e4cdb" />
    <img alt="PostgreSQL" src="https://img.shields.io/badge/PostgreSQL-15%2B-336791?logo=postgresql&logoColor=white" />
    <img alt="License" src="https://img.shields.io/badge/License-Elastic--2.0-blue" />
  </p>
</div>

---

## Table of Contents

- [Overview](#overview)
- [Why BarqFlow](#why-barqflow)
- [Architecture](#architecture)
- [Product Surfaces](#product-surfaces)
- [Core Capabilities](#core-capabilities)
- [Quick Start (Docker)](#quick-start-docker)
- [Local Development](#local-development)
- [Configuration Reference](#configuration-reference)
- [HTTP Surface](#http-surface)
- [Repository Layout](#repository-layout)
- [Technology Stack](#technology-stack)
- [Development Workflow](#development-workflow)
- [Security Notes](#security-notes)
- [License](#license)

---

## Overview

BarqFlow is a Rust-first automation platform for building, operating, and governing workflow-driven integrations.

A single control plane unifies a full-screen visual workflow designer, a queue-backed execution engine with dedicated run and trigger workers, credential lifecycle management, workspace-aware access control, and signed built-in extension packs.

BarqFlow draws on the UX of modern node-based automation products, but it is implemented as its own Rust and Vue platform in this repository.

## Why BarqFlow

- **Rust performance and safety end to end** — the control plane, execution engine, and runtime are written in Rust on Axum and Tokio for predictable latency and memory safety.
- **Durable, queue-backed execution** — manual runs and trigger-driven runs are dispatched through a persistent queue with separate worker lanes, leasing, and backpressure.
- **Workspace-aware governance** — identity, membership, API keys, policies, approvals, and audit posture are first-class concerns, not bolted on.
- **Signed extensibility** — built-in extension packs are verified against trusted public keys and run with capability-scoped actions.
- **Container-native deployment** — a single command brings up the full stack via Docker Compose.

## Architecture

<div align="center">
  <img src="docs/assets/architecture.png" alt="BarqFlow Architecture" width="100%" />
</div>

The Vue 3 control plane talks to the Axum REST and webhook runtime. Platform services handle auth, workspaces, governance, and the node and credential registry, and enqueue work onto the execution dispatch queue. Run and trigger workers drain the queue, execute against the node runtime and integrations, and persist state to PostgreSQL.

<details>
<summary>Text fallback (Mermaid)</summary>

```mermaid
flowchart LR
    Browser["Vue 3 Control Plane"] --> API["Axum REST API / Webhook Runtime"]
    API --> Auth["Auth / Workspaces / Governance"]
    API --> Queue["Execution Dispatch Queue"]
    API --> Registry["Node and Credential Registry"]
    Queue --> Workers["Run Workers / Trigger Workers"]
    Workers --> Nodes["Node Runtime / Integrations / Code"]
    API --> Postgres["PostgreSQL"]
    Workers --> Postgres
    Registry --> Nodes
```

</details>

## Product Surfaces

### Workflow Operations
- Workflow catalog with search, tags, templates, import/export, and history
- Full-screen editor with visual graph design and node detail tabs
- Workflow activation, trigger management, and execution launch controls

### Execution and Runtime
- Durable execution dispatch queue
- Separate worker lanes for manual runs and trigger-driven runs
- Execution event streaming, history, retry, stop, and resume flows
- Webhook and cron trigger handling

### Credential and Identity
- Encrypted credential storage
- Credential testing, rotation, lifecycle metadata, and external secret bindings
- Workspace-aware identity, membership, and API key management

### Observability and Governance
- Execution log inspection and live timelines
- Observability dashboards for latency, bottlenecks, failures, and credential health
- Governance controls for policies, approvals, promotions, and audit visibility

### Automation Studio
- Prompt-to-workflow draft generation
- Signed built-in extension pack discovery
- Capability-scoped extension action invocation for trusted bundles

## Core Capabilities

| Area | Current Coverage |
|---|---|
| Workflow design | Visual canvas, node detail view, parameter/settings/run-data tabs |
| Execution control | Manual run, test node, stop, retry, wait/resume, streamed events |
| Runtime scale | Durable queue, run/trigger worker split, queue metrics |
| Credentials | CRUD, testing, rotation, bindings, external secret references |
| Identity | Login, registration, workspaces, members, API keys |
| Operations | Execution monitor, runtime settings, structured logs |
| Observability | Latency histograms, bottleneck ranking, failure clustering |
| Governance | Audit posture, policies, approvals, promotion controls |
| Extensibility | Signed built-in extension packs with capability-scoped actions |
| AI workflow drafting | Prompt-based starter workflow generation in Automation Studio |

## Quick Start (Docker)

Docker deployment is the recommended way to run BarqFlow. The repository ships a deployment helper in [`deploy.sh`](deploy.sh), a container build in [`docker/Dockerfile`](docker/Dockerfile), and a multi-service environment in [`docker/docker-compose.yml`](docker/docker-compose.yml).

### Prerequisites

- Docker
- Docker Compose

### Standard deployment

```bash
git clone https://github.com/YASSERRMD/BarqFlow.git
cd BarqFlow
./deploy.sh
```

The deployment script stops existing BarqFlow containers, rebuilds the images with `docker/docker-compose.yml`, and starts the stack in detached mode. Then open `http://localhost:3000`.

### Manual Docker Compose workflow

```bash
git clone https://github.com/YASSERRMD/BarqFlow.git
cd BarqFlow
docker-compose -f docker/docker-compose.yml down
docker-compose -f docker/docker-compose.yml build --no-cache
docker-compose -f docker/docker-compose.yml up -d
```

### What the stack starts

- `db`: PostgreSQL 15
- `engine`: the BarqFlow Rust server plus compiled frontend assets

The compose file exposes PostgreSQL on `5432` and BarqFlow on `3000`. The default compose file includes development-style inline secrets and must be replaced or overridden for production deployments. For production, set strong values for `JWT_SECRET`, `BARQFLOW_ENCRYPTION_KEY`, and `DATABASE_URL`.

## Local Development

Use local Rust and frontend development only when you are actively modifying the platform.

### Prerequisites

- Rust `1.88+`
- Node.js `20+`
- npm `10+`
- PostgreSQL `15+`

### 1. Configure environment

Create a root `.env` file:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/barqflow
BARQFLOW_ENCRYPTION_KEY=replace-with-32-byte-secret-key
JWT_SECRET=replace-with-strong-jwt-secret
PORT=3000
RUST_LOG=info,barqflow=debug
BARQFLOW_ENV=development
```

### 2. Run database migrations

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run --source crates/api/migrations
```

### 3. Start the backend

```bash
cargo run -p barqflow-server
```

### 4. Start the frontend

```bash
cd web
npm install
npm run dev
```

The frontend dev server runs on `http://localhost:3001` and proxies API and webhook traffic to the Rust backend on port `3000`.

### Frontend production build

```bash
cd web
npm install
npm run build
```

The production backend serves compiled frontend assets from `web/dist`.

## Configuration Reference

### Required secrets

| Variable | Required | Notes |
|---|---|---|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `BARQFLOW_ENCRYPTION_KEY` | Yes | Must be exactly 32 characters |
| `JWT_SECRET` | Required in production | Development may use an ephemeral fallback |

### Common runtime controls

| Variable | Default | Purpose |
|---|---:|---|
| `PORT` | `3000` | HTTP server port |
| `RUST_LOG` | `info` | Rust logging filter |
| `BARQFLOW_ENV` | `development` | Enables production enforcement paths |
| `BARQFLOW_EXECUTION_RUN_WORKER_CONCURRENCY` | `4` | Run worker pool size |
| `BARQFLOW_EXECUTION_TRIGGER_WORKER_CONCURRENCY` | `2` | Trigger worker pool size |
| `BARQFLOW_EXECUTION_QUEUE_CAPACITY` | `128` | Queue backpressure ceiling |
| `BARQFLOW_EXECUTION_POLL_INTERVAL_MS` | `750` | Dispatch worker polling interval |
| `BARQFLOW_EXECUTION_WORKER_LEASE_SECONDS` | `300` | Queue lease timeout |
| `BARQFLOW_TRACING_ENABLED` | `true` | Request/runtime tracing toggle |
| `BARQFLOW_EXTENSION_TRUSTED_KEYS_FILE` | `extensions/trusted-public-keys.json` | Trusted public key manifest for extension verification |

## HTTP Surface

| Base | Path |
|---|---|
| REST | `/rest` |
| Webhook | `/webhook` |

### Major API domains

- Auth and user profile
- Workspaces and membership
- Workflows, templates, history, and diff
- Executions, logs, and event streams
- Credentials and credential types
- Settings and runtime posture
- Observability overview
- Governance controls
- Automation studio and extension runtime actions

## Repository Layout

```text
bin/
  barqflow/                 CLI wrapper
crates/
  api/                      REST controllers, routes, repositories, governance, observability
  core/                     Shared contracts, types, traits, schema primitives
  db/                       Database pool helpers and models
  exec/                     Workflow runner, runtime context, polling, checkpoints
  flow/                     Graph helpers and expression handling
  nodes/                    Built-in nodes, credentials, trigger/runtime implementations
  polling/                  Polling-related crate support
  registry/                 Node and credential registries
  server/                   Boot sequence and top-level app state
extensions/
  ai-automation-pack/       Built-in signed AI extension pack
  ops-observability-pack/   Built-in signed operations extension pack
web/
  public/                   Static assets
  src/                      Vue application, views, feature modules, contracts
```

## Technology Stack

- **Backend:** Rust, Axum, Tokio, SQLx, PostgreSQL
- **Frontend:** Vue 3, TypeScript, Pinia, Vue Router, Vue Flow, Tailwind CSS
- **Runtime:** queue-backed execution workers, cron scheduling, webhook dispatch, SSE execution streams
- **Security:** JWT auth, encrypted credentials, signed extension manifests, workspace scoping

## Development Workflow

- Follow the phased git workflow in [`git_workflow.md`](git_workflow.md)
- Keep changes small, atomic, and committed per task
- Use branch names prefixed with `codex/` for automation-driven work
- Do not merge or push directly to `main`

## Security Notes

- Do not commit real `.env` files or live secrets
- Keep `JWT_SECRET` and `BARQFLOW_ENCRYPTION_KEY` in a secrets manager in production
- Treat extension trust keys as controlled deployment artifacts
- Review credential bindings and governance policies before enabling production workflows

## License

BarqFlow is licensed under `Elastic License 2.0`.

This repository is source-available, but it is not licensed for offering BarqFlow as a hosted or managed commercial service. See [LICENSE](LICENSE) for the governing terms.

---

<div align="center">
  <sub>Designed and engineered by <strong>Mohamed Yasser</strong> · Solutions Architect</sub>
</div>
