# BarqFlow vs n8n Gap Analysis and Implementation Plan

## Goal

Compare the current Rust/Vue BarqFlow implementation in this repository against the reference n8n repository in `./tmp/n8n`, identify the real product/runtime gaps, and define a practical implementation plan that reaches:

1. n8n-like core parity
2. frontend parity for daily use
3. a separate "advanced BarqFlow" track that goes beyond parity

This document is based on the current repository state on 2026-03-10.

## Measured Snapshot

### BarqFlow current size

- Rust/backend source files: `141`
- Frontend source files: `22`
- Frontend routes: `7`
- UI-exposed node types: `60`
- Frontend test files: `0`

### n8n reference size

- `editor-ui` source files: `2005`
- Approximate route/path entries in router: `114`
- Frontend feature files: `1322`
- Frontend app/core files: `561`
- Frontend experiment files: `81`
- Frontend test definitions: `10739`
- Built-in node files in `nodes-base`: `529`
- AI/LangChain node files: `121`
- Credential definition files: `387`

## Anchor Files Reviewed

BarqFlow:

- `web/src/router/index.ts`
- `web/src/views/WorkflowEditor.vue`
- `web/src/views/ExecutionViewer.vue`
- `web/src/views/Credentials.vue`
- `web/src/components/NodePanel.vue`
- `web/src/components/Canvas.vue`
- `crates/api/src/routes.rs`
- `crates/api/src/controllers/workflows.rs`
- `crates/api/src/controllers/executions.rs`
- `crates/api/src/controllers/credentials.rs`
- `crates/api/src/controllers/settings.rs`
- `crates/exec/src/runner.rs`
- `crates/nodes/src/lib.rs`
- `crates/nodes/src/code.rs`
- `crates/nodes/src/trigger.rs`

n8n reference:

- `tmp/n8n/packages/frontend/editor-ui/src/app/router.ts`
- `tmp/n8n/packages/frontend/editor-ui/src/features/workflows/*`
- `tmp/n8n/packages/frontend/editor-ui/src/features/ndv/*`
- `tmp/n8n/packages/frontend/editor-ui/src/features/execution/*`
- `tmp/n8n/packages/frontend/editor-ui/src/features/settings/*`
- `tmp/n8n/packages/frontend/editor-ui/src/features/collaboration/*`
- `tmp/n8n/packages/frontend/editor-ui/src/features/credentials/*`
- `tmp/n8n/packages/frontend/editor-ui/src/features/ai/*`
- `tmp/n8n/packages/cli/src/controllers/*`
- `tmp/n8n/packages/core/src/*`
- `tmp/n8n/packages/workflow/src/*`

## What BarqFlow Already Has

BarqFlow is not empty. It already contains a real base platform:

- Rust workspace split by domain: `core`, `flow`, `exec`, `registry`, `nodes`, `db`, `api`, `server`, `polling`
- DAG parsing and topological workflow execution
- partial execution to a target node
- workflow CRUD and activation
- manual execution, retry, stop, wait/resume, sub-workflow hooks
- encrypted credential persistence
- webhook routing
- a Vue 3 editor shell with workflow list, editor, execution history, credentials, login, and settings views
- roughly `60` node types surfaced in the UI

The problem is not "nothing exists". The problem is that the implementation depth is uneven:

- some backend subsystems are real
- some nodes are broad but shallow
- the frontend is far behind n8n in architecture and features
- operational/admin/collaboration capabilities are mostly missing

## Evidence of Structural Gaps

### Frontend

- BarqFlow router only exposes login, workflows, workflow editor, executions, credentials, and settings.
- n8n has a full application router with templates, onboarding, history, settings subsections, project views, AI views, auth variants, execution views, resource center, and more.
- BarqFlow frontend has `22` source files total.
- n8n `editor-ui` alone has `2005` source files.
- BarqFlow has no frontend test suite.

### Backend/API

- BarqFlow API controllers are limited to workflows, executions, credentials, users, nodes, settings, oauth2, webhooks, and health.
- n8n backend exposes many more domains: API keys, MFA, invitations, projects, roles, tags, folders, dynamic node params, workflow statistics, binary data, source control, orchestration, telemetry, translation, AI, security settings, and more.

### Node/runtime depth

- BarqFlow exposes `60` node types in UI, but n8n has hundreds of built-in nodes plus AI nodes.
- BarqFlow `Code` node is not parity-complete: it maps "JavaScript mode" to Rhai execution and explicitly rejects Python mode.
- Trigger implementations are minimal compared with n8n's trigger lifecycle, setup UX, and activation model.

### Product depth

- n8n has large dedicated feature areas for:
  - workflow canvas and ready-to-run workflows
  - NDV (node detail view) parameters, settings, and run data
  - execution logs and insights
  - templates and workflow history
  - projects/collaboration
  - settings subsections and security/admin
  - credentials quick connect and richer forms
  - AI assistant, chat hub, evaluations, MCP access
- BarqFlow currently has none of these as first-class product areas.

## Detailed Gap Matrix

| Area | BarqFlow status | Gap level | Notes |
|---|---|---:|---|
| Workflow engine core | Partial/real | Medium | DAG execution exists, but parity gaps remain around advanced execution semantics, richer expression/runtime compatibility, and large-scale orchestration. |
| Expression engine | Partial | Medium | Custom expression support exists, but n8n-compatible parameter semantics and helper surface are still much smaller. |
| Base nodes | Partial | Medium | Core nodes exist, but behavior depth, edge cases, and compatibility are incomplete. |
| Integration nodes | Broad but shallow | High | Many integrations exist as files, but overall breadth and operation depth are far below n8n. |
| Trigger lifecycle | Partial | High | Manual/webhook/cron exist, but setup UX, statefulness, polling richness, dedupe, and activation polish lag. |
| Execution persistence | Partial | Medium | Basic executions and wait resumes exist, but no execution log richness, pin data, streaming, pruning policies, or analytics parity. |
| Credentials | Partial | High | CRUD/test exists, but UX, OAuth richness, sharing, rotation, quick connect, and external secret support are missing. |
| Auth and identity | Minimal | High | Basic login/register exists; MFA, SSO, invitations, project membership, password flows, API keys, RBAC are missing. |
| Workflow management | Minimal | High | No folders, tags UI, templates, history, diff, projects, sharing, versions, import/export UX, or search richness. |
| Frontend editor canvas | Minimal shell | Critical | Visual editor exists, but n8n-grade canvas/NDV behavior is mostly absent. |
| Node configuration UX | Partial | Critical | Basic schema form rendering exists, but no NDV architecture, input/output inspection, parameter sections, or dynamic parameter resolution. |
| Execution UX | Minimal | Critical | List/detail view exists, but no live streaming, logs, per-node inspection, replay tools, or insights. |
| Settings/admin | Minimal | High | Only runtime settings screen exists. |
| Collaboration/projects | Missing | High | No projects, roles, sharing, comments, presence, ownership boundaries. |
| AI product features | Missing | High | No assistant, AI builder, AI node packs, evaluations, or MCP-style access. |
| Plugin/extensibility model | Missing/limited | High | Registry exists internally, but there is no external extension SDK or marketplace-grade packaging story. |
| Frontend quality gates | Missing | Critical | No frontend tests, limited state architecture, no i18n, no feature-flag framework. |
| Observability/ops | Minimal | High | Lacks telemetry, metrics dashboards, log streaming, scaling controls, and worker orchestration features. |

## Current Repo Findings by Area

### 1. Frontend Architecture

Current BarqFlow frontend is a compact app with:

- `web/src/views/Login.vue`
- `web/src/views/WorkflowList.vue`
- `web/src/views/WorkflowEditor.vue`
- `web/src/views/ExecutionViewer.vue`
- `web/src/views/Credentials.vue`
- `web/src/views/Settings.vue`

Main issues:

- no modular feature architecture
- no NDV split between parameters/settings/run data
- no real-time execution channel
- no shared design system
- no i18n
- no test harness
- no command bar, context menu, onboarding, templates, workflow history, tags, or collaboration UI

Important cleanup item:

- `web/src/components/Canvas.vue` is still a mock prototype with hardcoded nodes and wires. It should either be removed or replaced by the production editor architecture to avoid confusion and divergence.

### 2. Workflow Editor and Node UX

Current editor strengths:

- loads/saves workflows
- adds nodes
- connects nodes
- shows basic side panel for node parameters
- supports workflow execution and single-node execution

Current editor weaknesses:

- no n8n-style Node Detail View
- no split panes for parameters/settings/output
- no input/output schema browsing
- no pin data
- no expression editor UX
- no connection validation and guidance UX
- no template insertion flow
- no execution path highlighting
- no run-data timeline
- no undo/redo history system
- no multi-selection/group operations
- no keyboard-heavy power-user workflow

### 3. Runtime and Execution

What is real today:

- graph parsing
- topological execution
- partial execution to a node
- wait/resume support
- stop/cancel support
- subworkflow hook points

What is missing or not yet n8n-grade:

- richer execution event model
- live push transport to the UI
- detailed node logs
- pinned data and replayable execution context
- advanced error handling/reporting UX
- worker isolation/scaling strategy
- full trigger state management lifecycle
- richer binary data handling and browsing

### 4. Nodes and Integrations

Current condition:

- BarqFlow chose breadth early
- many integration files exist
- likely maintenance and parity cost is now higher than the current supporting UX/runtime

Main issue:

The product currently advertises many integrations, but the surrounding product systems are not mature enough yet:

- credential selection UX is basic
- operation discovery is limited
- docs/help inside UI are limited
- testing and run-data inspection are minimal
- edge-case behavior across integrations is not yet backed by product tooling

Recommendation:

- stop expanding node count until the editor/runtime contract is stabilized
- define a "Tier 1 supported node pack" first
- make the UI/runtime excellent for those nodes before widening again

### 5. Data Model and API Domains

Current core entities are limited to:

- users
- workflows
- executions
- credentials
- static data
- wait resumes

Major missing domains relative to n8n:

- projects/workspaces
- folders
- tags as a first-class API/UI domain
- role assignments and RBAC
- invitations and membership lifecycle
- API keys
- MFA
- password reset/change flows
- user settings
- workflow history/diff/versioning
- usage analytics and workflow statistics
- binary data access APIs
- environment variables/external secrets
- community nodes/module management

### 6. Quality, Tests, and Release Safety

Current state:

- backend has useful tests
- frontend has no tests
- there is no parity safety net comparable to n8n

This means frontend development will stay fragile until:

- unit tests exist for stores/composables/components
- workflow editor interactions get component tests
- backend/frontend contract tests are introduced
- a small E2E suite validates the critical user journeys

## Recommended Product Strategy

Do not try to clone n8n package-by-package.

The correct path is:

1. stabilize BarqFlow's own backend/frontend contract
2. reach parity in the highest-value workflows
3. widen operational/admin/product surface
4. add advanced BarqFlow-native features after the base is stable

The sequencing matters. If frontend parity is attempted before the core workflow document model, execution event model, and node detail contract are solid, the UI will become another scaffold layer.

## Implementation Plan

## Track A: Core Parity

### Phase 1: Freeze the Contract Surface

Objective:

Define the canonical workflow document, node schema contract, execution event contract, and credential binding contract that both backend and frontend will follow.

Atomic tasks:

- write a formal workflow JSON contract document
- write node schema JSON response examples from the current API
- define execution event payloads for queued/running/node-start/node-finish/wait/error/stopped/success
- define a credential binding contract that supports selection, validation, masking, and future sharing
- mark deprecated prototype UI pieces for removal

Exit criteria:

- backend and frontend use one documented contract set
- no mock-only editor path remains ambiguous

### Phase 2: Rebuild the Frontend Into Feature Modules

Objective:

Replace the current page-centric frontend with a feature architecture closer to n8n's real product boundaries.

Atomic tasks:

- split `web/src` into feature modules: auth, workflows, canvas, ndv, executions, credentials, settings, shared
- introduce typed API client modules instead of one generic axios wrapper
- replace `any`-heavy stores with typed stores/composables
- add frontend test runner and initial component/store tests
- introduce a shared UI token/component layer

Exit criteria:

- new frontend architecture exists
- tests run for key stores/components

### Phase 3: Implement a Real Node Detail View

Objective:

Move from a simple side panel to a proper Node Detail View.

Atomic tasks:

- split node editing into tabs: Parameters, Settings, Credentials, Run Data
- implement typed property renderers for string, text, number, boolean, options, collection, fixed collection
- support conditional visibility rules and defaults fully
- add inline docs/help and credential status indicators
- add node validation and required-field feedback

Exit criteria:

- node editing no longer depends on raw generic forms only
- most Tier 1 nodes are fully configurable from NDV

### Phase 4: Execution UX Parity Foundation

Objective:

Make workflow runs inspectable and debuggable from the UI.

Atomic tasks:

- add execution event streaming from backend to frontend
- show per-node status on canvas during runs
- add node run data panel with input/output/error tabs
- add stop/retry/resume actions directly in execution UI
- add run filtering by workflow/status/time
- add execution timeline and duration summaries

Exit criteria:

- a user can run, inspect, and debug a workflow without raw JSON digging

### Phase 5: Workflow Management Parity

Objective:

Add the minimum workflow-management features required for serious use.

Atomic tasks:

- promote tags to first-class API and UI behavior
- add workflow search, sort, filters, and metadata summaries
- add workflow duplication/import/export UX
- add workflow history snapshots and visual diff
- add onboarding/new-workflow starter flows
- add template gallery import path

Exit criteria:

- workflow list and editor feel like a working product, not only a canvas demo

### Phase 6: Credential System Parity

Objective:

Make credentials safe, testable, and convenient enough for broad integration use.

Atomic tasks:

- add credential picker and per-node binding UX
- add OAuth flow polish and callback handling
- add credential usage metadata and last-tested state
- support secret masking, patching, rotation, and re-test flows
- add quick-connect patterns for the top integrations

Exit criteria:

- credentials stop being a blocking product weakness

### Phase 7: Tier 1 Node Pack Hardening

Objective:

Choose a smaller set of nodes and make them production-grade end to end.

Recommended Tier 1 pack:

- Manual Trigger
- Webhook
- Cron Trigger
- HTTP Request
- Set
- Filter
- If
- Switch
- Merge
- Code
- Wait
- Execute Workflow
- Postgres
- Slack
- GitHub
- OpenAI

Atomic tasks:

- add contract tests per Tier 1 node
- verify credential flows end to end
- improve error messages and docs
- normalize parameter naming and defaults
- add integration test fixtures for the top operations

Exit criteria:

- these nodes are dependable in the editor, runtime, and execution inspector

### Phase 8: Identity and Workspace Basics

Objective:

Introduce the minimum multi-user platform features.

Atomic tasks:

- add password reset/change
- add API keys
- add project/workspace model
- add workflow ownership and sharing model
- add roles and basic RBAC

Exit criteria:

- BarqFlow can serve teams, not only a single local user

### Phase 9: Operations and Reliability

Objective:

Make the system operable in real deployments.

Atomic tasks:

- add execution pruning policies
- add health/metrics endpoints with richer runtime data
- add structured execution logs
- add worker mode or queue-backed execution isolation
- add telemetry and tracing hooks

Exit criteria:

- production behavior is measurable and controllable

### Phase 10: Widen the Node Ecosystem

Objective:

Only after the editor/runtime/credential system is stable, continue widening node coverage.

Atomic tasks:

- classify current nodes into Supported, Beta, Hidden
- graduate existing broad node set one group at a time
- add dynamic parameter support where needed
- expand credential schemas and operation coverage

Exit criteria:

- node surface area grows without creating more scaffolding debt

## Track B: Advanced BarqFlow

This track should start only after Track A Phases 1-6 are largely complete.

### Advanced 1: Rust-Native Scaled Execution Fabric

Build what n8n does not do as well:

- durable queue-backed execution workers
- separate trigger workers and run workers
- resumable event-sourced execution records
- backpressure and concurrency controls per node/workflow/project

### Advanced 2: WASM or Capability-Sandboxed Extension Runtime

Instead of copying community-node behavior directly:

- define a Rust-first plugin SDK
- consider WASM sandboxing for third-party nodes
- support signed plugin bundles and permission-scoped capabilities

### Advanced 3: AI-Native Builder

BarqFlow can go beyond n8n if it treats AI as a product layer, not just nodes:

- workflow generation assistant
- parameter autofill from API docs/OpenAPI
- run failure diagnosis and fix suggestions
- prompt-to-workflow starter generation
- execution summarization

### Advanced 4: Deep Observability

- per-node latency histograms
- workflow bottleneck detection
- execution flamegraph-like views
- failure clustering
- credential health dashboards

### Advanced 5: Governance and Enterprise Controls

- audit logs
- secret providers and vault integration
- policy controls for node usage
- environment promotion and source control sync
- deployment approval workflow

## Priority Order

If only one roadmap is followed, use this order:

1. Phase 1: contract freeze
2. Phase 2: frontend feature architecture
3. Phase 3: NDV
4. Phase 4: execution UX
5. Phase 6: credentials
6. Phase 7: Tier 1 node hardening
7. Phase 5: workflow management parity
8. Phase 8: identity/workspaces
9. Phase 9: operations/reliability
10. Phase 10: wider node ecosystem
11. Advanced track

## Recommended Immediate Next Work

The next implementation branch should not add more integrations.

It should do these first:

1. formalize the workflow/node/execution contracts
2. remove or isolate prototype frontend pieces
3. create the new frontend feature layout
4. implement NDV plus execution event streaming

That sequence gives the rest of the roadmap a stable base.

## Mapping to Git Workflow

Per `git_workflow.md`, the implementation should be done as distinct phases with atomic tasks committed immediately after completion.

Recommended branch pattern:

- `codex/phase_50_contract_surface`
- `codex/phase_51_frontend_feature_architecture`
- `codex/phase_52_ndv_foundation`
- `codex/phase_53_execution_streaming_and_inspection`
- `codex/phase_54_workflow_management_parity`
- `codex/phase_55_credential_system_parity`
- `codex/phase_56_tier1_node_hardening`
- `codex/phase_57_identity_and_workspaces`
- `codex/phase_58_operations_and_reliability`
- `codex/phase_59_node_ecosystem_expansion`
- `codex/phase_60_advanced_barqflow`

Each phase should be split into small commits. Example for Phase 50:

- commit 1: add contract docs
- commit 2: add typed shared frontend types
- commit 3: align backend API payloads
- commit 4: add contract tests

## Bottom Line

BarqFlow already has a serious backend foundation, but it is still a compact product shell compared with n8n.

The biggest gaps are not only "more nodes". The critical missing layers are:

- frontend architecture
- Node Detail View
- execution inspection/debug UX
- credential UX depth
- workflow/product management features
- identity/workspace/admin capabilities
- reliability and testing infrastructure

If these are solved first, the existing Rust foundation becomes an advantage. If more integrations are added before these layers are solved, the repo will continue to grow in breadth faster than in product completeness.
