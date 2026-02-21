# BarqFlow Complete Implementation Plan (Deep n8n Architecture)

This detailed plan breaks down the entire system into highly granular, independently verifiable phases based heavily on the actual n8n codebase (specifically `packages/core` and `packages/workflow`).

Each phase builds upon the previous one without cyclic dependencies. The tasks inside each phase represent the atomic commits that will be made to branch `phase_X_xyz`.

---

## Phase 2: Core Primitives & Interfaces (`crates/core`)
*Purpose: Define the absolutely lowest-level data structures based on n8n's `workflow/src/interfaces.ts`. Zero dependencies.*

- [ ] Define `ItemId` and `RunId` UUID aliases in `types.rs`
- [ ] Define `IBinaryData` struct matching n8n (mimeType, fileExtension, base64 payload vs fs reference) in `types.rs`
- [ ] Define `IDataObject` wrapper (mapping to `serde_json::Value` / `HashMap`) in `types.rs`
- [ ] Define `INodeExecutionData` struct (json data + binary data references) in `schema.rs`
- [ ] Define `INodeParameters` enum for raw input values in `schema.rs`
- [ ] Define `INodeConnections` map structure representing the graph edges in `schema.rs`
- [ ] Define `INode` struct representing a single step definition in `schema.rs`
- [ ] Define `IWorkflow` struct (the entire graph of nodes and connections) in `schema.rs`
- [ ] Implement `BarqError` variants matching n8n: `ExpressionError`, `NodeApiError`, `NodeOperationError` in `errors.rs`
- [ ] Define strictly typed `INodeType` trait (the logic implementation of a Node) in `traits.rs`
- [ ] Define strictly typed `IExecuteFunctions` trait (the context passed to a running node) in `traits.rs`
- [ ] Implement unit tests for `INodeExecutionData` serialization

## Phase 3: Graph Traversal Engine (`crates/flow`)
*Purpose: Process the `IWorkflow` struct. Convert connections into an executable graph. Independent of nodes.*

- [ ] Initialize `petgraph` Directed Acyclic Graph (DAG) structures
- [ ] Implement function to parse `IWorkflow` into `petgraph::DiGraph` covering Main/Alternative routing
- [ ] Implement Topological Sort to generate strict execution ordering
- [ ] Implement Cycle Detection (Return `BarqError::WorkflowOperationError` if cycle found)
- [ ] Implement Parent Node Resolver (find ancestors of a specific node for data resolution)
- [ ] Write suite of unit tests for Graph topological sorting
- [ ] Write suite of unit tests for Cycle Detection

## Phase 4: Expression Evaluation Engine (`crates/flow`)
*Purpose: Implement n8n's `{{ $json.data }}` expression syntax using Rhai. Independent of node logic.*

- [ ] Set up `rhai` Engine instance with secure sandboxing
- [ ] Implement Expression Parser to extract `{{ }}` blocks from strings
- [ ] Create Rhai `Scope` builder to inject `$json`, `$binary`, `$env`, `$parameter`
- [ ] Implement Context Traversal logic (fetching `$json` from previously executed nodes)
- [ ] Add Rhai registered functions mapping to n8n built-ins (e.g., `$today`, `$now`, `$evaluateExpression`)
- [ ] Implement deep-object recursive expression evaluation
- [ ] Write comprehensive unit tests for syntax parsing
- [ ] Write comprehensive unit tests for Context interpolation

## Phase 5: Executable Node Context & Binary Data (`crates/exec`)
*Purpose: Build the `IExecuteFunctions` concrete implementation that wraps a running node, and handle binary streams.*

- [ ] Implement `BinaryDataManager` mapped to local `fs` (to abstract away large buffer memory limits)
- [ ] Implement `GetNodeParameter` utility with Rhai expression evaluation hooks
- [ ] Implement `GetNodeParameter` type assertions (`ensureType: string/number/boolean`)
- [ ] Implement `InputData` router (merging outputs from multiple upstream branches)
- [ ] Implement Context Logger proxy (redirecting node logs to system tracing spans)
- [ ] Implement `ReturnJsonArray` standardized helper
- [ ] Write tests validating that Binary streams are correctly written and read from the temp FS

## Phase 6: Core Execution Loop & Sub-Workflows (`crates/exec`)
*Purpose: The main runner that walks the topological graph and invokes nodes sequentially.*

- [ ] Define `IRun` and `IRunExecutionData` structs for state tracking
- [ ] Create `ExecutionState` manager to track memory buffers per node
- [ ] Implement the `runNode` method (calls the active trait and catches panics/errors)
- [ ] Implement the main topological execution while-loop (forward propagation)
- [ ] Implement Conditional Branching logic (handling nodes that return multi-index arrays like IF/Switch)
- [ ] Implement **Sub-Workflow Runner Hook** (allowing a node to spawn a child execution tracker)
- [ ] Implement Checkpointing hook system (for pausing/resuming async workflows/waiting for webhooks)
- [ ] Implement Error Routing (if node fails, trigger `Error Trigger` node if exists)
- [ ] Write end-to-end unit tests executing a dummy 3-node graph

## Phase 7: Node & Credential Registry & Sandbox (`crates/registry`)
*Purpose: The runtime map of all installed nodes and security boundaries.*

- [ ] Define `INodeTypeDescription` structs for UI form generation (matching n8n's properties model)
- [ ] Implement `NodeRegistry` thread-safe Singleton (RwLock wrapping a HashMap of NodeTypes)
- [ ] Define `ICredentialType` structure for defining UI forms for Auth
- [ ] Implement credential Pre-Authentication and Validation hooks capability
- [ ] Implement `CredentialRegistry` thread-safe Singleton
- [ ] Create macros for static Node Registration at startup
- [ ] Write registry unit tests validating duplicate-registration handling

## Phase 8: Standard Built-in Nodes Foundation (`crates/nodes`)
*Purpose: Implement the absolute baseline nodes necessary to build basic workflows.*

- [ ] Implement `ManualTrigger` node (Entrypoint for testing)
- [ ] Implement `Set` node (Basic data mutation & expression usage)
- [ ] Implement `If` node (Basic boolean branching outputting to index 0 or 1)
- [ ] Implement `Switch` node (Multi-branch routing mapping strings to outputs)
- [ ] Implement `ExecuteWorkflow` node (Invokes the Sub-Workflow hooks from `crates/exec`)
- [ ] Write isolated execution tests for `Set`, `If`, and `Switch` nodes

## Phase 9: Complex Nodes & Deduplication (`crates/nodes`)
*Purpose: Implement nodes requiring external IO, scheduling, and stateful triggers.*

- [ ] Implement `HttpRequest` node using `reqwest` (GET/POST/Auth/JSON serialization)
- [ ] Implement `Webhook` trigger node (Requires tight integration with Server router)
- [ ] Implement `Cron` trigger node using `tokio-cron-scheduler`
- [ ] Implement `Code` node (Executing raw Javascript-like Rhai scripts safely)
- [ ] Implement `PollingTrigger` abstract logic, handling generic setInterval style runs
- [ ] Implement `DeduplicationService` (Ensuring polling triggers don't process the same ID twice)
- [ ] Write integration block tests for `HttpRequest` mock calls

## Phase 10: Database Access Layer (`crates/api`)
*Purpose: Persistence layer using SQLx. Strictly CRUD, no web routing.*

- [ ] Create SQLx Postgres Connection Pool builder
- [ ] Write DB Migration for `workflow_entities` (id, name, active, definitions, timestamps)
- [ ] Write DB Migration for `execution_entities` (id, workflow_id, status, data, wait_till, stopped_at)
- [ ] Write DB Migration for `credential_entities` (id, name, type, data_encrypted)
- [ ] Write DB Migration for `static_data` (Stores deduplication markers and polling cursors)
- [ ] Implement `WorkflowRepository` struct with CRUD operations
- [ ] Implement `ExecutionRepository` struct with insert and status-update operations
- [ ] Implement `AES-256-GCM` encryption/decryption module for credentials
- [ ] Implement `CredentialRepository` struct with implicit encryption
- [ ] Write integration tests for Repositories using SQLx mock/test macros

## Phase 11: REST API, Auth & OAuth2 Flows (`crates/api`)
*Purpose: Axum controllers exposing the repositories.*

- [ ] Set up `Argon2` password hashing module
- [ ] Construct JWT generation and validation layers
- [ ] Implement `POST /rest/login` controller
- [ ] Implement `GET /rest/workflows` and `POST /rest/workflows` controllers
- [ ] Implement `GET /rest/node-types` registry exposition controller for the UI
- [ ] Implement `POST /rest/credentials/test` endpoint to invoke the validation hooks
- [ ] Implement **OAuth2 Callback Endpoints** (`/rest/oauth2-credential/callback`)
- [ ] Implement `POST /rest/executions/` controller manual workflow triggering
- [ ] Implement `GET /rest/executions/:id` controller for polling UI
- [ ] Write Axum `Router` API unit tests using `tower::ServiceExt`

## Phase 12: Server Orchestration (`crates/server`)
*Purpose: The main entrypoint that wires Repositories, Registries, Exec Engine, and API together.*

- [ ] Initialize `tracing-subscriber` env logger
- [ ] Initialize SQLx DB pool and run automated migrations
- [ ] Populate `NodeRegistry` with all implementations from `crates/nodes`
- [ ] Setup Axum state wrapping DB Pool + Registry + Exec Engine
- [ ] Spawn detached Tokio task for the `Cron` trigger manager polling
- [ ] Dynamically mount `Webhook` trigger Axum routes mapping directly to the active workflow cache
- [ ] Start Axum HTTP listener on standard port
- [ ] Manually test server boot sequence

*(Web UI phase omitted from independent phased plan as per instructions)*
