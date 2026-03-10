export type ExecutionStatus =
  | 'queued'
  | 'running'
  | 'waiting'
  | 'success'
  | 'failed'
  | 'stopped'
  | 'cancelled'

export type ExecutionEventType =
  | 'queued'
  | 'started'
  | 'nodeStarted'
  | 'nodeFinished'
  | 'waiting'
  | 'resumed'
  | 'failed'
  | 'stopped'
  | 'completed'

export interface NodePropertyOption {
  name: string
  value: unknown
  description?: string | null
}

export interface NodeDisplayCondition {
  property: string
  values: unknown[]
}

export interface NodeDisplayOptions {
  show?: NodeDisplayCondition | null
}

export type NodePropertyType =
  | 'string'
  | 'text'
  | 'boolean'
  | 'number'
  | 'options'
  | 'collection'
  | 'multiSelect'
  | 'loadOptions'
  | {
      type: string
      values: string[]
    }

export interface NodeProperty {
  displayName: string
  name: string
  type: NodePropertyType
  default?: unknown
  description?: string | null
  hint?: string | null
  required?: boolean
  options?: NodePropertyOption[] | null
  displayOptions?: NodeDisplayOptions | null
}

export interface CredentialReference {
  credentialType: string
  required: boolean
  displayName: string
}

export type NodeSupportTier = 'supported' | 'beta' | 'hidden'

export interface NodeSchemaContract {
  name: string
  displayName: string
  description: string
  category: string
  supportTier: NodeSupportTier
  supportNote?: string | null
  isTrigger: boolean
  typeVersion: number
  maxInputs: number
  documentationUrl?: string | null
  defaults: Record<string, unknown>
  properties: NodeProperty[]
  credentials: CredentialReference[]
}

export interface NodeCatalogEntry {
  name: string
  type: string
  kind: 'trigger' | 'action' | 'manipulation'
  description: string
  isTrigger: boolean
  schema: NodeSchemaContract
  category: string
  supportTier: NodeSupportTier
  supportNote?: string | null
}

export interface NodeDynamicOptionsRequest {
  nodeType: string
  propertyName: string
  currentParameters: Record<string, unknown>
}

export interface NodeDynamicOptionsResponse {
  options: NodePropertyOption[]
  source: string
  note?: string | null
}

export interface NodeCredentialBinding {
  nodeId: string
  credentialType: string
  credentialId: string
}

export interface WorkflowNode {
  id: string
  name: string
  type: string
  typeVersion: number
  position: [number, number]
  parameters: Record<string, unknown>
  credentials: NodeCredentialBinding[]
  disabled: boolean
}

export interface WorkflowConnection {
  node: string
  type: 'main'
  index: number
}

export type WorkflowConnections = Record<
  string,
  {
    main?: WorkflowConnection[][]
  }
>

export interface WorkflowSettings {
  timezone?: string | null
  saveDataErrorExecution?: string | null
  errorWorkflow?: string | null
  saveExecutionProgress?: boolean | null
  saveManualExecutions?: boolean | null
  callerPolicy?: string | null
}

export interface WorkflowTag {
  id: string
  name: string
  createdAt: string
  updatedAt: string
}

export interface TagRecord extends WorkflowTag {
  workflowCount: number
}

export interface WorkflowSummary {
  nodeCount: number
  triggerCount: number
  credentialBindingCount: number
  tagCount: number
  latestVersion: number
}

export interface WorkflowRecord {
  id: string
  name: string
  active: boolean
  tags: WorkflowTag[]
  summary: WorkflowSummary
  nodes: WorkflowNode[]
  connections: WorkflowConnections
  settings: WorkflowSettings
  createdAt: string
  updatedAt: string
}

export interface WorkflowUpsertRequest {
  name: string
  nodes: WorkflowNode[]
  connections: WorkflowConnections
  settings: WorkflowSettings
  tags?: string[]
}

export interface WorkflowImportDocument {
  name: string
  nodes: WorkflowNode[]
  connections: WorkflowConnections
  settings: WorkflowSettings
  tags?: string[]
}

export interface WorkflowImportRequest {
  workflow: WorkflowImportDocument
  nameOverride?: string
}

export interface WorkflowHistoryEntry {
  version: number
  source: string
  name: string
  active: boolean
  tags: string[]
  summary: WorkflowSummary
  createdAt: string
}

export interface WorkflowNodeChange {
  nodeId: string
  nodeName: string
  changedFields: string[]
}

export interface WorkflowHistoryDiff {
  workflowId: string
  fromVersion: number
  toVersion: number
  fromName: string
  toName: string
  nameChanged: boolean
  activeChanged: boolean
  tagsAdded: string[]
  tagsRemoved: string[]
  settingsChanged: string[]
  nodesAdded: string[]
  nodesRemoved: string[]
  nodesChanged: WorkflowNodeChange[]
  connectionsAdded: string[]
  connectionsRemoved: string[]
}

export interface WorkflowTemplateRecord {
  id: string
  name: string
  description: string
  category: string
  difficulty: string
  tags: string[]
  highlights: string[]
  summary: WorkflowSummary
  nodes: WorkflowNode[]
  connections: WorkflowConnections
  settings: WorkflowSettings
}

export interface WorkflowExportRecord {
  format: string
  exportedAt: string
  workflow: WorkflowRecord
}

export interface ExecutionNodeSummary {
  nodeName: string
  success: boolean
  error?: string | null
  outputsCount: number
}

export interface ExecutionRecord {
  id: string
  workflowId: string
  status: string
  data: Record<string, unknown>
  startedAt: string
  stoppedAt?: string | null
}

export interface CreateExecutionRequest {
  manual?: boolean
  stopAtNodeId?: string
}

export interface ExecutionEvent {
  executionId: string
  workflowId: string
  runId: string
  eventType: ExecutionEventType
  status: ExecutionStatus
  nodeId?: string | null
  nodeName?: string | null
  message: string
  timestamp: string
  sequence: number
  data: Record<string, unknown>
}

export interface ExecutionLogRecord {
  id: string
  executionId: string
  workflowId: string
  level: string
  eventType?: string | null
  message: string
  nodeId?: string | null
  nodeName?: string | null
  payload: Record<string, unknown>
  createdAt: string
}

export interface RuntimeSettings {
  serverTime: string
  environment: string
  nodeTypesCount: number
  credentialTypesCount: number
  encryptionKeyConfigured: boolean
  executionMode: string
  workerConcurrency: number
  queueCapacity: number
  pruningEnabled: boolean
  executionRetentionDays: number
  tracingEnabled: boolean
  traceFormat: string
}

export interface ExecutionDispatchMetrics {
  mode: string
  workerConcurrency: number
  queueCapacity: number
  queuedCount: number
  runningCount: number
  totalEnqueued: number
  totalStarted: number
  totalFinished: number
  totalFailedToDispatch: number
  lastEnqueuedAt?: string | null
  lastStartedAt?: string | null
  lastFinishedAt?: string | null
}

export interface ExecutionPruningStatus {
  enabled: boolean
  retentionDays: number
  intervalMinutes: number
  lastRunAt?: string | null
  lastCutoffAt?: string | null
  lastExecutionsDeleted: number
  lastWaitResumesDeleted: number
  lastLogsDeleted: number
}

export interface TelemetrySettings {
  enabled: boolean
  format: string
  serviceName: string
  environment: string
  requestIdHeader: string
}

export interface OperationsOverview {
  dispatch: ExecutionDispatchMetrics
  pruning: ExecutionPruningStatus
  telemetry: TelemetrySettings
  activeExecutions: number
  webhookEndpointCount: number
  webhookWorkflowCount: number
  cronWorkflowCount: number
  cronJobCount: number
  nodeTypesCount: number
  credentialTypesCount: number
  generatedAt: string
}

export interface LatencyBucket {
  label: string
  count: number
}

export interface NodeLatencyHistogram {
  workflowId: string
  workflowName: string
  nodeName: string
  nodeType: string
  samples: number
  failedRuns: number
  avgDurationMs: number
  p95DurationMs: number
  maxDurationMs: number
  histogram: LatencyBucket[]
}

export interface WorkflowBottleneck {
  workflowId: string
  workflowName: string
  nodeName: string
  nodeType: string
  samples: number
  failureCount: number
  avgDurationMs: number
  p95DurationMs: number
  contributionRate: number
}

export interface FailureCluster {
  clusterKey: string
  workflowId?: string | null
  workflowName?: string | null
  nodeName?: string | null
  nodeType?: string | null
  level: string
  eventType?: string | null
  message: string
  failureCount: number
  affectedExecutionCount: number
  lastSeenAt: string
}

export interface CredentialHealthRecord {
  credentialId: string
  name: string
  credentialType: string
  health: string
  issues: string[]
  lastTestStatus?: string | null
  lastTestedAt?: string | null
  lastUsedAt?: string | null
  rotatedAt?: string | null
  usageCount: number
}

export interface ExecutionFlamegraphSpan {
  nodeName: string
  nodeType: string
  offsetMs: number
  durationMs: number
  status: string
  startedAt: string
  finishedAt: string
  inputItems: number
  outputItems: number
}

export interface ExecutionFlamegraph {
  executionId: string
  workflowId: string
  workflowName: string
  status: string
  startedAt: string
  stoppedAt?: string | null
  totalDurationMs: number
  spans: ExecutionFlamegraphSpan[]
}

export interface ObservabilityOverview {
  workspaceId: string
  generatedAt: string
  windowHours: number
  workflowCount: number
  executionCount: number
  terminalExecutionCount: number
  successfulExecutionCount: number
  failedExecutionCount: number
  stoppedExecutionCount: number
  queuedExecutionCount: number
  runningExecutionCount: number
  waitingExecutionCount: number
  successRate: number
  failureRate: number
  averageExecutionDurationMs: number
  nodeLatencyHistograms: NodeLatencyHistogram[]
  workflowBottlenecks: WorkflowBottleneck[]
  failureClusters: FailureCluster[]
  credentialHealth: CredentialHealthRecord[]
  executionFlamegraphs: ExecutionFlamegraph[]
}

export interface PruneExecutionsResult {
  cutoff: string
  ranAt: string
  executionsDeleted: number
  waitResumesDeleted: number
  logsDeleted: number
}

export interface CredentialSummary {
  id: string
  name: string
  credentialType: string
  data: Record<string, unknown>
  createdAt: string
  updatedAt: string
  lastTestedAt?: string | null
  lastTestStatus?: string | null
  lastTestMessage?: string | null
  lastUsedAt?: string | null
  usageCount: number
  rotatedAt?: string | null
}

export interface CredentialAuthenticateContract {
  in: string
  name: string
  value: string
}

export interface CredentialTypeContract {
  name: string
  displayName: string
  notice?: string | null
  properties: NodeProperty[]
  documentationUrl?: string | null
  authenticate?: CredentialAuthenticateContract | null
}

export interface CredentialValidationResult {
  valid: boolean
  status: 'valid' | 'invalid' | 'error' | string
  message: string
  credentialId?: string | null
  credentialType?: string | null
}

export interface CredentialOAuthConnectResult {
  credentialId: string
  credentialType: string
  connectUrl: string
  redirectUri: string
  state: string
}

export interface WorkspaceSummary {
  id: string
  name: string
  slug: string
  role: string
  createdAt: string
  updatedAt: string
}

export interface WorkspaceMember {
  membershipId: string
  userId: string
  email: string
  firstName?: string | null
  lastName?: string | null
  role: string
  createdAt: string
  updatedAt: string
}

export interface ApiKeyRecord {
  id: string
  name: string
  keyPrefix: string
  workspaceId: string
  userId: string
  lastUsedAt?: string | null
  expiresAt?: string | null
  revokedAt?: string | null
  createdAt: string
  updatedAt: string
}

export interface ApiKeyCreateResult {
  apiKey: string
  key: ApiKeyRecord
}

export interface UserProfile {
  id: string
  email: string
  firstName?: string | null
  lastName?: string | null
  role: string
  workspaceRole: string
  activeWorkspace: WorkspaceSummary
  workspaces: WorkspaceSummary[]
}

export interface AuthResponse {
  token: string
  userId: string
  user: UserProfile
}

export interface ChangePasswordRequest {
  currentPassword: string
  newPassword: string
}

export interface CreateWorkspaceRequest {
  name: string
}

export interface AddWorkspaceMemberRequest {
  email: string
  role: string
}

export interface CreateApiKeyRequest {
  name: string
  expiresAt?: string | null
}

export interface SecretProviderRecord {
  id: string
  workspaceId: string
  name: string
  providerType: string
  config: Record<string, unknown>
  status: string
  lastValidatedAt?: string | null
  lastError?: string | null
  createdAt: string
  updatedAt: string
}

export interface WorkspacePolicyRecord {
  workspaceId: string
  blockedNodeTypes: string[]
  blockedSupportTiers: string[]
  approvalRequiredNodeTypes: string[]
  maxWorkflowNodes?: number | null
  createdAt: string
  updatedAt: string
}

export interface PromotionTargetRecord {
  id: string
  workspaceId: string
  name: string
  environment: string
  gitRepoUrl?: string | null
  gitBranch?: string | null
  requiresApproval: boolean
  createdAt: string
  updatedAt: string
}

export interface PromotionRequestRecord {
  id: string
  workspaceId: string
  workflowId: string
  targetId: string
  requestedByUserId?: string | null
  approvedByUserId?: string | null
  status: string
  sourceControlRef?: string | null
  workflowSnapshot: Record<string, unknown>
  notes?: string | null
  requestedAt: string
  approvedAt?: string | null
}

export interface AuditLogRecord {
  id: string
  workspaceId: string
  actorUserId?: string | null
  actorEmail?: string | null
  action: string
  resourceType: string
  resourceId?: string | null
  summary: string
  metadata: Record<string, unknown>
  createdAt: string
}

export interface CreateSecretProviderRequest {
  name: string
  providerType: string
  config: Record<string, unknown>
}

export interface UpdateWorkspacePolicyRequest {
  blockedNodeTypes: string[]
  blockedSupportTiers: string[]
  approvalRequiredNodeTypes: string[]
  maxWorkflowNodes?: number | null
}

export interface CreatePromotionTargetRequest {
  name: string
  environment: string
  gitRepoUrl?: string | null
  gitBranch?: string | null
  requiresApproval?: boolean
}

export interface CreatePromotionRequestPayload {
  workflowId: string
  targetId: string
  sourceControlRef?: string | null
  notes?: string | null
}

export interface ApprovePromotionRequestPayload {
  notes?: string | null
}

export interface ExtensionPermissionScope {
  network: string[]
  credentials: string[]
  workflow: string[]
  filesystem: string[]
}

export interface ExtensionProvidedAssets {
  nodes: string[]
  templates: string[]
  panels: string[]
}

export interface ExtensionBundleRecord {
  id: string
  name: string
  vendor: string
  version: string
  runtime: string
  description: string
  homepage?: string | null
  entrypoint?: string | null
  capabilities: string[]
  permissions: ExtensionPermissionScope
  providedAssets: ExtensionProvidedAssets
  sourcePath: string
  digest: string
  status: 'validated' | 'validatedWithWarnings' | 'needsAttention' | string
  warnings: string[]
}

export interface WorkflowDraftRequest {
  prompt: string
}

export interface WorkflowDraftRecord {
  generator: string
  name: string
  summary: string
  rationale: string[]
  warnings: string[]
  suggestedTags: string[]
  requiredCredentials: string[]
  recommendedExtensions: string[]
  nodes: WorkflowNode[]
  connections: WorkflowConnections
  settings: WorkflowSettings
}
