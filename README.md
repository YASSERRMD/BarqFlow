<div align="center">
  <img src="web/public/logo.png" alt="BarqFlow Logo" width="120" height="120" />
  <h1>BarqFlow</h1>
  <p><strong>Rust-based workflow automation platform with a Vue 3 visual editor</strong></p>
</div>

## Overview
BarqFlow is a workflow automation engine built with Rust (Axum/Tokio) and Vue 3.

It is inspired by node-based automation products, including n8n, at the product and UX level (visual canvas, trigger/action model, workflow graph execution). It is an independent implementation in this repository and is not a copy/paste of n8n source code.

## What BarqFlow Includes
- Workflow execution engine with graph traversal, branching, and scoped node test execution
- REST API under `/rest` for users, workflows, executions, credentials, settings, and health
- Webhook runtime under `/webhook/{path}`
- Wait/resume execution flow with resume tokens
- Runtime execution stop/cancellation support
- Node registry + schema-driven node forms in the editor
- Credential storage with encrypted payloads (`BARQFLOW_ENCRYPTION_KEY`)
- JWT-based auth with hardened production secret handling
- Dockerized deployment path

## Runtime Endpoints
- API base: `/rest`
- Webhook base: `/webhook`
- Static UI assets are served from `web/dist` by the backend in production mode

## Architecture
```text
crates/
  core/        Shared types, traits, schema contracts
  flow/        Expression engine and flow helpers
  exec/        Workflow runner, context, checkpointing
  registry/    Node/Credential registries
  nodes/       Node implementations and node schema registration
  db/          Database models and access layer
  api/         Axum controllers, routes, repositories
  server/      Boot sequence and app state wiring
bin/
  barqflow/    CLI entry wrapper
web/
  src/         Vue 3 + Pinia + Vue Router UI
  public/      Static assets
docker/
  Dockerfile
  docker-compose.yml
```

## Quick Start (Docker)
```bash
git clone https://github.com/YASSERRMD/BarqFlow.git
cd BarqFlow
./deploy.sh
```

Then open:
- `http://localhost:3000`

## Local Development
### Prerequisites
- Rust `1.88+`
- Node.js `20+`
- `pnpm`
- PostgreSQL `15+`

### Environment Variables
Set these in `.env` (root):

| Variable | Required | Notes |
|---|---|---|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `BARQFLOW_ENCRYPTION_KEY` | Yes | Must be exactly 32 characters |
| `PORT` | No | Default `3000` |
| `RUST_LOG` | No | Example: `info,barqflow=debug` |
| `JWT_SECRET` | Required in production | In development, an ephemeral secret is generated if missing |
| `BARQFLOW_ENV` | No | Use `production` to enforce strict JWT secret requirement |

### Database Migrations
```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run --source crates/api/migrations
```

### Run Backend
```bash
cargo run -p barqflow-server
```

### Run Frontend (dev server)
```bash
cd web
pnpm install
pnpm dev
```

### Build Frontend for Backend Static Serving
```bash
cd web
pnpm build
```

## API Surface (Current)
- Auth/User
  - `POST /rest/users`
  - `POST /rest/login`
  - `GET /rest/users/me`
- Workflows
  - `GET/POST /rest/workflows`
  - `GET/PUT/DELETE /rest/workflows/{id}`
  - `PUT /rest/workflows/{id}/activate`
  - `POST /rest/workflows/{id}/duplicate`
- Executions
  - `GET /rest/executions`
  - `POST /rest/executions/workflow/{workflow_id}`
  - `POST /rest/executions/workflow/{workflow_id}/test-node/{node_id}`
  - `POST /rest/executions/{id}/stop`
  - `POST /rest/executions/{id}/retry`
  - `POST /rest/executions/{id}/resume/{resume_token}`
- Credentials
  - `GET/POST /rest/credentials`
  - `DELETE /rest/credentials/{id}`
  - `GET /rest/credentials/types`
  - `POST /rest/credentials/test`
- Nodes
  - `GET /rest/nodes`
- Health
  - `GET /rest/health/triggers`
- Webhook
  - `ANY /webhook/{path}`

## Production Notes
- Keep `JWT_SECRET` and `BARQFLOW_ENCRYPTION_KEY` managed in a secrets manager.
- Do not commit `.env` files with real secrets.
- Frontend build artifacts (`web/dist`) are generated during deployment/build.

## Contributing
- Follow the repository git workflow conventions in `git_workflow.md`.
- Keep changes atomic and tested.
- Prefer adding or updating tests with behavior changes.

## License
See [LICENSE](LICENSE). If you are packaging or redistributing, validate license metadata across repository files and Cargo package metadata.
