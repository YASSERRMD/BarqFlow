<div align="center">
  <img src="web/public/logo.png" alt="BarqFlow Logo" width="120" height="120" />
  <h1>BarqFlow</h1>
  <p><strong>A Hyper-Scalable, Production-Ready Agentic Workflow Automation Engine Written in Rust 🦀 & Vue 3 ⚡️</strong></p>

  <p>
    <a href="https://rustup.rs/"><img src="https://img.shields.io/badge/rust-1.80%2B-orange.svg" alt="Rust Version" /></a>
    <a href="https://vuejs.org/"><img src="https://img.shields.io/badge/vue-3.5%2B-brightgreen.svg" alt="Vue Version" /></a>
    <a href="https://github.com/YASSERRMD/BarqFlow/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" /></a>
  </p>
</div>

---

## ⚡️ What is BarqFlow?

BarqFlow is an ultra-fast, n8n-inspired workflow automation engine built entirely from scratch in **Rust**. It provides a pure stateless graph-execution pipeline utilizing a sophisticated **Rhai** scripting engine for arbitrary payload resolution.

With BarqFlow, you can construct massive multi-branch workflows with drag-and-drop web interfaces and execute them securely with zero-overhead async parallelization via **Tokio**.

### 🚀 Key Features

* **High-Performance Rust Core:** Powered by the Axum framework and Tokio async runtime, handling tens of thousands of simultaneous webhooks gracefully.
* **Advanced Graph Execution Engine:** Cycle-detection, topological sorting, state checkpointing, and branch merging mechanisms built directly into `petgraph`.
* **Dynamic Node Registry:** Safe schema parsing for dynamically generated components (HTTP Requests, Conditionals, Merges, Webhooks, Scripts).
* **Rhai Expression System:** Built-in powerful scripting expressions (`=$json.body.user.name`) seamlessly embedded directly into your execution components.
* **Secure Credential Injection:** Symmetric AES-256 encryption via `argon2` ensures total confidentiality for your OAuth tokens and DB credentials.
* **Vue 3 Beautiful Glassmorphism UI:** Complete frontend canvas leveraging rich SVGs, animations, and Pinia stores.
* **Ready for Production:** Included PostgreSQL schema, migrations, Axum static-file servicing, JWT Authentication, and native Docker configurations.

---

## 🛠 Architecture Overview

The codebase is organized as a unified Cargo Workspace spanning critical execution boundaries:

```text
packages/
 ├── crates/core      # Foundational Data Primitives, Graph Models & Errors
 ├── crates/db        # SQLite/PostgreSQL Repositories via SQLx + Migrations & Crypto
 ├── crates/flow      # Topo Execution Walkers, Graph Resolution & Rhai Expression Engine
 ├── crates/exec      # Tokio Runner, State Checkpointing & Telemetry State
 ├── crates/nodes     # Concrete implementations (Webhook, HTTP Request, Filter, If)
 ├── crates/api       # REST Router, Axum Controllers, & JWT Stateless Auth
 ├── crates/registry  # In-memory maps parsing Node Form UI Contracts
 └── crates/server    # Global Boot Sequence mapping API and Graph boundaries & Static File Serving
 
web/                  # The UI Frontend Canvas (Vue 3 / Vite)
```

---

## 🐳 Quick Start (Docker Compose)

The fastest and most reliable way to experience BarqFlow is via the provided `docker-compose.yml`.

1. **Clone the repository:**
   ```bash
   git clone https://github.com/YASSERRMD/BarqFlow.git
   cd BarqFlow
   ```

2. **Spin up the stack (Postgres + Engine + UI):**
   *(Note: The `docker-compose.yml` and `Dockerfile` are neatly organized under the `docker/` folder rather than cluttering the root directory!)*
   We've included a robust `deploy.sh` script to ensure you always build fresh images and wipe old cache variables:
   ```bash
   ./deploy.sh
   ```
   
   *(Alternatively, you can manually run `docker-compose -f docker/docker-compose.yml up --build -d`)*

3. **Access the application:**
   Navigate your browser to `http://localhost:3000`. You will see the login screen and can begin dragging nodes onto the canvas!

---

## 💻 Manual Developer Setup 

### Prerequisites

* Rust 1.80+ (via rustup)
* Node.js 20+ & `pnpm`
* PostgreSQL 15+

### 1. Database Configuration

Ensure PostgreSQL is running locally.

```bash
# Create a valid .env file in the repository root
echo 'DATABASE_URL=postgres://postgres:postgres@localhost:5432/barqflow' > .env
echo 'JWT_SECRET=super_secret_temporary_key_for_dev_only' >> .env
echo 'ENCRYPTION_KEY=super_secret_32_byte_aes_gcm_development_key!!' >> .env

# Install sqlx-cli to handle migrations
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx database create
sqlx migrate run --source crates/db/migrations
```

### 2. Build the Frontend Canvas

The UI must be built before the Axum server spins up, as Axum serves the compiled artifacts on `0.0.0.0:3000`.

```bash
cd web
pnpm install
pnpm run build
cd ..
```

### 3. Spin up the Core Engine

```bash
cargo run --bin barqflow-server
```

*(You can now access `http://localhost:3000` via your browser).*

---

## 🤝 Contribution

BarqFlow is open-source and welcomes pull requests! Specifically, we are always looking to expand our **Nodes Ecosystem**. Check out `crates/nodes/src` to see how you can develop a new `INodeType` integration with just a few lines of Rust!

## 📜 License

This project is licensed under the [MIT License](LICENSE).
