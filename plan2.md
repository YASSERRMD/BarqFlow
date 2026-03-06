# BarqFlow Massive Implementation Architecture (50+ Phases)

This architecture document represents a hyper-granular, totally atomic 50+ phase implementation plan, replicating the entire feature set of n8n into a Rust virtual workspace. Every single phase operates completely independently, and every item is an atomic Git commit mapped to an isolated branch.

---

## Part 1: Core Primitives & Base Schemas (`crates/core`)

### Phase 2: Core Identification Types
- [ ] Define `ItemId` (UUID mapping)
- [ ] Define `RunId` (UUID mapping)
- [ ] Define `WorkflowId` (UUID mapping)
- [ ] Define `NodeId` (String alias)
- [ ] Write unit tests for ID serialization/deserialization

### Phase 3: Core Data Structures (JSON)
- [ ] Define `IDataObject` wrapping `serde_json::Map`
- [ ] Define `GenericValue` wrapping `serde_json::Value`
- [ ] Implement `From<serde_json::Value>` traits for `IDataObject`
- [ ] Write serialization tests for deeply nested JSON objects

### Phase 4: Core Data Structures (Binary)
- [ ] Define `BinaryFileType` Enum (text, json, image, audio, video, pdf, html)
- [ ] Define `IBinaryData` struct (mimeType, fileName, fileExtension)
- [ ] Implement abstract pointers for Binary data (memory buffer vs filesystem reference)
- [ ] Write unit tests for Binary metadata serialization

### Phase 5: Core Execution Data Models
- [ ] Define `INodeExecutionData` composed of JSON data and optional binary references
- [ ] Define `ITaskDataConnections` representing inputs passed into a node during execution
- [ ] Define `NodeExecutionHint` for UI messaging
- [ ] Write aggregation tests for `INodeExecutionData` arrays

### Phase 6: Core Graph Models
- [ ] Define `NodeConnectionType` Enum (main, trigger, catch)
- [ ] Define `IConnection` struct (target node, type, output index)
- [ ] Define `INodeConnections` map structure
- [ ] Define `INodeParameters` map structure for user-defined node config
- [ ] Define `IWorkflowSettings` (timezone, error workflow triggers, deduplication scope)
- [ ] Define `WorkflowDef` combining nodes, connections, and settings

### Phase 7: Core System Errors
- [ ] Implement `BarqError` Enum using `thiserror`
- [ ] Add `WorkflowActivationError` variant
- [ ] Add `ExecutionCancelledError` variant
- [ ] Add `NodeApiError` variant
- [ ] Add `ExpressionError` variant
- [ ] Add `NodeOperationError` variant
- [ ] Write error display trait implementations

### Phase 8: Core Interfaces & Traits
- [ ] Define `INodeType` async trait (execute methods, node metadata)
- [ ] Define `IExecuteFunctions` trait (context provided to running nodes)
- [ ] Define `ICredentialType` trait (authentication abstractions)
- [ ] Define `IPollFunctions` trait (specific context for polling triggers)

---

## Part 2: Graph Engine & Expression Resolver (`crates/flow`)

### Phase 9: Graph Infrastructure Initialization
- [ ] Setup `petgraph` dependency
- [ ] Define `GraphNode` and `GraphEdge` typed indices
- [ ] Implement `WorkflowToGraphParser` returning `petgraph::DiGraph`
- [ ] Write unit tests validating complex multi-branch graph generation

### Phase 10: Graph Traversal & Topo Sort
- [ ] Implement `isExecutableDAG` cycle detection validation
- [ ] Implement standard forward topological sort for sequential execution
- [ ] Implement parent/ancestor resolution mapping (finding all paths to Node A)
- [ ] Write tests ensuring triggers are always topologically sorted first

### Phase 11: Expression Syntax Parser (Rhai)
- [ ] Set up `rhai` Engine sandbox parameters
- [ ] Implement `Regex`-based extractor for `{{ }}` blocks inside generic strings
- [ ] Implement AST validator for isolated `rhai` script blocks
- [ ] Write unit tests testing valid vs invalid expression block syntax

### Phase 12: Expression Context Injection
- [ ] Create specialized `rhai::Scope` generator
- [ ] Inject `$json` mapping to the current item's execution data
- [ ] Inject `$binary` mapping to current item's binary keys
- [ ] Inject `$parameter` mapping to the active node's parameters
- [ ] Inject `$env` mapping to global system environment variables
- [ ] Write context injection safety bounds tests

### Phase 13: Expression Node State Traversal
- [ ] Implement `$item("NodeName").$json` abstraction to search backward in the graph
- [ ] Implement graph backward traversal caching to optimize ancestor lookups
- [ ] Hook the graph traversal results into the `rhai::Scope` functions
- [ ] Write integration block tests for complex `$item` expressions

### Phase 14: Built-in Expression Functions
- [ ] Register `now()` equivalent in Rhai
- [ ] Register `today()` equivalent in Rhai
- [ ] Register `hash()` manipulation functions in Rhai
- [ ] Register URL decoding/encoding string functions in Rhai
- [ ] Write tests validating that custom Rhai functions execute correctly

---

## Part 3: Node Ecosystem & Security (`crates/registry`)

### Phase 15: Node Form Descriptions (UI Contracts)
- [ ] Define `INodeProperties` structs representing UI components (Strings, Booleans, Options, Collections)
- [ ] Define `NodePropertyOptions` for dropdowns
- [ ] Define `NodeDisplayOptions` (show parameter X only if parameter Y is Z)
- [ ] Write serialization tests ensuring UI contracts map perfectly to JSON

### Phase 16: Node Type Registry
- [ ] Implement `NodeRegistry` thread-safe `RwLock<HashMap>`
- [ ] Implement `register_node` macro/function
- [ ] Implement `get_node_by_name` lookup
- [ ] Implement versioning alias resolution (falling back to v1 if v2 missing)
- [ ] Write duplicate registration conflict tests

### Phase 17: Credential Form Descriptions (UI Contracts)
- [ ] Define `ICredentialsProperties` structs
- [ ] Define generic `OAuth2` specific pre-built form models
- [ ] Define `Authenticate` generic injection properties (headers vs body vs query)
- [ ] Write serialization tests for Authentication UI configurations

### Phase 18: Credential Registry & Validation
- [ ] Implement `CredentialRegistry` `RwLock<HashMap>`
- [ ] Implement `register_credential` function
- [ ] Implement `ICredentialTestRequest` rules engine to ping APIs for validation
- [ ] Write unit tests testing validation rule processing

---

## Part 4: Execution Engine & Lifecycle (`crates/exec`)

### Phase 19: Binary Data Filesystem Abstraction
- [ ] Implement `BinaryStorageConfig` (path directory resolution)
- [ ] Implement `store_binary_to_fs` taking `axum::body::Bytes`
- [ ] Implement `read_binary_from_fs` returning streaming chunks
- [ ] Implement `delete_binary_from_fs` for cleanup
- [ ] Write integration tests for temp file lifecycle

### Phase 20: Execution State Management
- [ ] Define `RunExecutionData` struct representing a full execution log
- [ ] Implement `ExecutionStateManager` wrapping active runs
- [ ] Implement metrics counters for `Nodes Executed`, `Data Processed`
- [ ] Write thread-safety tests for state managers updating the same run

### Phase 21: Node Execution Context (IExecuteFunctions)
- [ ] Implement `GetNodeParameter` utility tying into the Rhai expression engine
- [ ] Implement `EnsureType` coercions inside `GetNodeParameter`
- [ ] Implement `GetCredentials` mapping to SQLx DB (mocked for now)
- [ ] Implement Logger Proxies injecting `RunId` and `NodeId` into standard tracing spans
- [ ] Write parameter resolution tests

### Phase 22: Execution Router (The Core Loop)
- [ ] Implement single-thread `run_node` wrapper that traps panics
- [ ] Implement topological graph walker consuming topological index arrays
- [ ] Implement mapping between `Node A`'s output array and `Node B`'s input items
- [ ] Add item loop constraints (Iterating over multiple items inside a single node execution)
- [ ] Write tests ensuring inputs properly map to outputs without loss

### Phase 23: Branching & Conditional Routing
- [ ] Modifying graph walker to process nested arrays (Output Index 0 vs Index 1)
- [ ] Handle `If` / `Switch` node behaviors where path execution splits
- [ ] Handle `Merge` nodes waiting for multiple incoming branches to resolve
- [ ] Write complex branching execution tests

### Phase 24: Execution Suspend & Resume (Checkpointing)
- [ ] Implement `Wait` node serialization hooks
- [ ] Suspend execution loop, serialize state to DB, and drop the Tokio task
- [ ] Implement `Resume` hook reconstructing the execution context from DB
- [ ] Write integration test pausing and resuming a mock execution

### Phase 25: Error Triggers & Handlers
- [ ] Implement conditional `ErrorTrigger` checks
- [ ] If execution fails, spawn secondary execution with `ErrorTrigger` workflow
- [ ] Implement `Continue On Fail` boolean flag handling for standard nodes
- [ ] Write tests ensuring errors correctly trigger the fallback loops

### Phase 26: Sub-Workflow Invocation
- [ ] Implement `ExecuteWorkflow` runtime hook
- [ ] Allow a node to push a new Execution ID onto the stack and await its `RunExecutionData` output
- [ ] Aggregate child sub-workflow outputs back into the parent node output
- [ ] Write nested workflow integration tests

---

## Part 5: Node Sub-Systems & Triggers (`crates/nodes`)

### Phase 27: Base Manipulation Nodes
- [ ] Implement `SetNode` (Overriding fields, resolving expressions)
- [ ] Implement `FilterNode` (Removing items from the array based on expressions)
- [ ] Implement `ItemListsNode` (Splitting items into batches)
- [ ] Write isolated execution tests for manipulation elements

### Phase 28: Base Logic Nodes
- [ ] Implement `IfNode` (Boolean routing to paths 0 and 1)
- [ ] Implement `SwitchNode` (String multi-routing mapping values to paths)
- [ ] Implement `MergeNode` (Joining two diverse branches into one output array)
- [ ] Write isolated branching tests

### Phase 29: Base Interaction Nodes
- [ ] Implement `HttpRequestNode` (reqwest integration with complex JSON/Form parsing)
- [ ] Implement URL encoding/decoding, redirect following, proxy config
- [ ] Implement Binary file downloading inside `HttpRequestNode`
- [ ] Write network mocking tests (using mockito/wiremock)

### Phase 30: Base Trigger Nodes
- [ ] Implement `ManualTriggerNode` (For UI-based tests)
- [ ] Implement `WebhookNode` (Static configuration mapping methods to logic)
- [ ] Write structural tests for Webhook route validation

### Phase 31: Scheduled Polling Engine
- [ ] Implement `tokio-cron-scheduler` core event loop
- [ ] Setup memory map of active Cron schedules linked to `WorkflowId`
- [ ] Implement `CronTriggerNode`
- [ ] Write automated tick tests for cron triggers

### Phase 32: Arbitrary Polling Triggers
- [ ] Define `IPollFunctions` trait
- [ ] Implement generic polling loop (e.g., checking an API every X minutes)
- [ ] Hook polling errors into the standard telemetry stream
- [ ] Write tests simulating a mock polling trigger

### Phase 33: Deduplication Service
- [ ] Implement `DeduplicationManager`
- [ ] Handle `incremented_key` vs `array_of_ids` deduplication modes
- [ ] Wrap polling nodes in Deduplication wrappers so events fire only once
- [ ] Write hash-based deduplication verification tests

### Phase 34: Sandbox Execution Nodes
- [ ] Implement `CodeNode` (User-defined rhai/JS evaluation)
- [ ] Implement deep isolation parameters to prevent arbitrary execution
- [ ] Implement structured return mapping (forcing Code node to return valid DataObject Arrays)
- [ ] Write sandbox escape prevention tests

---

## Part 6: Persistence & ORM Layers (`crates/api`)

### Phase 35: DB Core Infrastucture
- [ ] Setup `sqlx` Postgres Config with max connection limits
- [ ] Setup `uuid` generating functions internally
- [ ] Implement automatic DB Migration execution on boot
- [ ] Write basic DB connectivity test logic

### Phase 36: Workflow CRUD Entities
- [ ] Database migration for `workflow_entity`
- [ ] Database migration for `tag_entity` and many-to-many relationships
- [ ] Implement `WorkflowRepository` (Find, Create, Update, Delete)
- [ ] Implement `findAllByActive` specialized query
- [ ] Write SQLx test macros for workflow entity lifecycle

### Phase 37: Execution CRUD Entities
- [ ] Database migration for `execution_entity`
- [ ] Implement `ExecutionRepository`
- [ ] Implement specific update queries optimizing massive JSONB payload storage
- [ ] Implement pruning jobs to delete old executions
- [ ] Write tests for execution fetching and payload sizes

### Phase 38: Credential Cryptography Layer
- [ ] Implement `AES-256-GCM` generic encryption module
- [ ] Implement secret key derivation from system environment variable
- [ ] Serialize objects to/from encrypted Base64 strings
- [ ] Write encryption verification testing

### Phase 39: Credential CRUD Entities
- [ ] Database migration for `credential_entity`
- [ ] Implement `CredentialRepository`
- [ ] Integrate Cryptography layer automatically into Repository reads/writes
- [ ] Write tests ensuring plain text credentials never enter postgres queries

### Phase 40: Static Data Storage
- [ ] Database migration for `static_data` (Stores polling cursors, dedup keys)
- [ ] Implement `StaticDataRepository`
- [ ] Link `StaticDataRepository` directly into `crates/exec` polling logic
- [ ] Write data continuity tests

### Phase 41: User CRUD Entities
- [ ] Database migration for `user_entity`
- [ ] Implement `UserRepository`
- [ ] Write basic access tests

---

## Part 7: Auth, REST API, & Webhook Orchestration (`crates/api`)

### Phase 42: Authentication Core
- [ ] Setup `argon2` for secure password hashing
- [ ] Setup `jsonwebtoken` for stateless session encoding
- [ ] Implement JWT decoding Axum Middleware (`tower` layer)
- [ ] Write unit tests for JWT expiration validation

### Phase 43: Auth/User Controllers
- [ ] Implement `POST /rest/login`
- [ ] Implement `POST /rest/users` (Registration)
- [ ] Implement `GET /rest/users/me` (Profile retrieval)
- [ ] Write route integration tests

### Phase 44: Workflow/Execution Controllers
- [ ] Implement `GET /rest/workflows` & `POST /rest/workflows`
- [ ] Implement `PUT /rest/workflows/:id/activate` (State toggling)
- [ ] Implement `GET /rest/executions` & `GET /rest/executions/:id`
- [ ] Implement `POST /rest/executions/:workflowId` (Manual trigger)
- [ ] Write route integration tests

### Phase 45: Credential Controllers & Testing
- [ ] Implement `GET /rest/credentials` & `POST /rest/credentials`
- [ ] Implement `POST /rest/credentials/test` triggering the validation hooks
- [ ] Write route integration tests

### Phase 46: OAuth2 Callback Mechanics
- [ ] Implement `GET /rest/oauth2-credential/callback`
- [ ] Exchange authorization codes dynamically using reqwest server-side
- [ ] Update credential entities with live auth tokens automatically
- [ ] Write mock OAuth2 callback tests

### Phase 47: Dynamic Webhook Routing
- [ ] Implement a dynamic router traversing the `active_workflows` cache
- [ ] Register `GET/POST /webhook/:path` catch-all endpoints dynamically
- [ ] Translate Axum HTTP Request entities into n8n Webhook Trigger payloads
- [ ] Write end-to-end tests sending a mock HTTP request to a dynamic webhook

---

## Part 8: Server Orchestration & Web UI (`crates/server` / `web/`)

### Phase 48: Global State Registration
- [ ] Initialize `tracing` subscriber stack
- [ ] Initialize SQLx Pool
- [ ] Inject SQLx traits into Node Registry and Credential Registry
- [ ] Boot standard Execution Manager State
- [ ] Write boot sequence tests

### Phase 49: Active Workflow Boot Sequence
- [ ] Query DB for all active workflows
- [ ] Register all active Webhooks in the Axum Router
- [ ] Register all active Crons in the `CronScheduler`
- [ ] Register all active Pollers in the `StaticData` loop
- [ ] Write tests ensuring deactivated workflows are suspended successfully

### Phase 50: Production Assembly
- [ ] Mount REST API behind `/rest/` prefix
- [ ] Mount Webhooks behind `/webhook/` prefix
- [ ] Configure graceful shutdown hooks terminating pending executions
- [ ] Write E2E startup-to-shutdown test in memory

### Phase 51+: Web UI Development
- [ ] Initialize Vue 3 Single Page Application (Vite)
- [ ] Build Vue Router configurations mapping to n8n routes
- [ ] Implement UI State Management using Pinia for UI Auth/Flows
- [ ] Implement `editor-ui` flow canvas component
- [ ] Implement generic dynamic property form renderer for Nodes
- [ ] Implement dynamic credential form renderer
- [ ] Wire Web UI to build static assets and serve them via Axum in production mode
