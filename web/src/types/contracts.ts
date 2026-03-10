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

export interface NodeSchemaContract {
  name: string
  displayName: string
  description: string
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

export interface CredentialSummary {
  id: string
  name: string
  credentialType: string
  data: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface CredentialTypeContract {
  name: string
  displayName: string
  notice?: string | null
  properties: NodeProperty[]
  documentationUrl?: string | null
}

export interface UserProfile {
  id: string
  email: string
  role: string
}

export interface AuthResponse {
  token: string
  userId: string
  user: UserProfile
}
