<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import { ChevronLeft, History, Loader2, Play, Plus, Save, Tag, X } from 'lucide-vue-next'

import CustomNode from '../components/CustomNode.vue'
import NodeCreator from '../components/NodeCreator.vue'
import NodePanel from '../components/NodePanel.vue'
import { useWorkflowStore } from '../stores/workflows'
import { useNodeStore } from '../stores/nodes'
import { useRoute, useRouter } from 'vue-router'
import { listCredentials } from '../features/credentials/api'
import {
  createExecutionEventSource,
  getExecution,
} from '../features/executions/api'
import type { ExecutionEvent, WorkflowHistoryDiff, WorkflowHistoryEntry } from '../types/contracts'
import {
  isTerminalExecutionEvent,
  mergeExecutionEvents,
  resolveExecutionStatusFromEvent,
} from '../features/executions/helpers'
import ExecutionStatusBadge from '../features/executions/components/ExecutionStatusBadge.vue'
import ExecutionTimeline from '../features/executions/components/ExecutionTimeline.vue'
import WorkflowHistoryPanel from '../features/workflows/components/WorkflowHistoryPanel.vue'

const route = useRoute()
const router = useRouter()
const workflowStore = useWorkflowStore()
const nodeStore = useNodeStore()
const { onConnect, addEdges, toObject, setNodes, setEdges, screenToFlowCoordinate } = useVueFlow()

const nodes = ref<any[]>([])
const edges = ref<any[]>([])
const selectedNode = ref<any>(null)
const showNodeCreator = ref(false)
const executionNotice = ref<{ type: 'success' | 'error'; message: string; showCredentialsAction?: boolean } | null>(null)
const nodeTestState = ref<{ nodeId: string; status: 'running' | 'success' | 'error'; message: string } | null>(null)
const executionInProgress = ref(false)
const liveExecutionId = ref<string | null>(null)
const liveExecutionEvents = ref<ExecutionEvent[]>([])
const workflowDraftName = ref('Untitled Workflow')
const workflowTags = ref<string[]>([])
const workflowTagInput = ref('')
const showHistoryPanel = ref(false)
const historyLoading = ref(false)
const historyDiffLoading = ref(false)
const selectedHistoryFromVersion = ref<number | null>(null)
const selectedHistoryToVersion = ref<number | null>(null)
const activeHistoryDiff = ref<WorkflowHistoryDiff | null>(null)

let executionEventSource: EventSource | null = null

const NODE_CREDENTIAL_REQUIREMENTS: Record<
  string,
  Array<{ credentialType: string; displayName: string; required: boolean }>
> = {
  'barqflow-nodes.openai': [
    { credentialType: 'openAiApi', displayName: 'OpenAI API', required: true },
  ],
  'barqflow-nodes.postgres': [
    { credentialType: 'postgresApi', displayName: 'Postgres', required: true },
  ],
  'barqflow-nodes.slack': [
    { credentialType: 'slackApi', displayName: 'Slack API', required: true },
  ],
  'barqflow-nodes.github': [
    { credentialType: 'githubApi', displayName: 'GitHub API', required: true },
  ],
  'barqflow-nodes.discord': [
    { credentialType: 'discordApi', displayName: 'Discord API', required: true },
  ],
  'barqflow-nodes.notion': [
    { credentialType: 'notionApi', displayName: 'Notion API', required: true },
  ],
  'barqflow-nodes.jira': [
    { credentialType: 'jiraApi', displayName: 'Jira API', required: true },
  ],
  'barqflow-nodes.stripe': [
    { credentialType: 'stripeApi', displayName: 'Stripe API', required: true },
  ],
  'barqflow-nodes.sendGrid': [
    { credentialType: 'sendGridApi', displayName: 'SendGrid API', required: true },
  ],
  'barqflow-nodes.hubspot': [
    { credentialType: 'hubspotApi', displayName: 'HubSpot API', required: true },
  ],
  'barqflow-nodes.asana': [
    { credentialType: 'asanaApi', displayName: 'Asana API', required: true },
  ],
  'barqflow-nodes.telegram': [
    { credentialType: 'telegramApi', displayName: 'Telegram Bot API', required: true },
  ],
  'barqflow-nodes.airtable': [
    { credentialType: 'airtableApi', displayName: 'Airtable API', required: true },
  ],
  'barqflow-nodes.awsS3': [
    { credentialType: 'awsS3Api', displayName: 'AWS S3 API', required: true },
  ],
  'barqflow-nodes.bitbucket': [
    { credentialType: 'bitbucketApi', displayName: 'Bitbucket API', required: true },
  ],
  'barqflow-nodes.calendly': [
    { credentialType: 'calendlyApi', displayName: 'Calendly API', required: true },
  ],
  'barqflow-nodes.dropbox': [
    { credentialType: 'dropboxApi', displayName: 'Dropbox API', required: true },
  ],
  'barqflow-nodes.gitlab': [
    { credentialType: 'gitlabApi', displayName: 'GitLab API', required: true },
  ],
  'barqflow-nodes.gmail': [
    { credentialType: 'gmailApi', displayName: 'Gmail API', required: true },
  ],
  'barqflow-nodes.googleDrive': [
    { credentialType: 'googleDriveApi', displayName: 'Google Drive API', required: true },
  ],
  'barqflow-nodes.googleSheets': [
    { credentialType: 'googleSheetsApi', displayName: 'Google Sheets API', required: true },
  ],
  'barqflow-nodes.oneDrive': [
    { credentialType: 'oneDriveApi', displayName: 'OneDrive API', required: true },
  ],
  'barqflow-nodes.linear': [
    { credentialType: 'linearApi', displayName: 'Linear API', required: true },
  ],
  'barqflow-nodes.mysql': [
    { credentialType: 'mysqlApi', displayName: 'MySQL API', required: true },
  ],
  'barqflow-nodes.redis': [
    { credentialType: 'redisApi', displayName: 'Redis API', required: true },
  ],
  'barqflow-nodes.zendesk': [
    { credentialType: 'zendeskApi', displayName: 'Zendesk API', required: true },
  ],
  'barqflow-nodes.salesforce': [
    { credentialType: 'salesforceApi', displayName: 'Salesforce API', required: true },
  ],
  'barqflow-nodes.quickbooks': [
    { credentialType: 'quickbooksApi', displayName: 'QuickBooks API', required: true },
  ],
  'barqflow-nodes.zoom': [
    { credentialType: 'zoomApi', displayName: 'Zoom API', required: true },
  ],
  'barqflow-nodes.trello': [
    { credentialType: 'trelloApi', displayName: 'Trello API', required: true },
  ],
  'barqflow-nodes.outlook': [
    { credentialType: 'outlookApi', displayName: 'Outlook API', required: true },
  ],
  'barqflow-nodes.paypal': [
    { credentialType: 'paypalApi', displayName: 'PayPal API', required: true },
  ],
  'barqflow-nodes.intercom': [
    { credentialType: 'intercomApi', displayName: 'Intercom API', required: true },
  ],
  'barqflow-nodes.xero': [
    { credentialType: 'xeroApi', displayName: 'Xero API', required: true },
  ],
  'barqflow-nodes.mailchimp': [
    { credentialType: 'mailchimpApi', displayName: 'Mailchimp API', required: true },
  ],
  'barqflow-nodes.freshdesk': [
    { credentialType: 'freshdeskApi', displayName: 'Freshdesk API', required: true },
  ],
}

function normalizeNodeCredentials(rawCredentials: any, nodeId: string): Record<string, string> {
  const normalized: Record<string, string> = {}
  if (!Array.isArray(rawCredentials)) return normalized

  rawCredentials.forEach((binding: any) => {
    const bindingNodeId = String(binding?.nodeId || binding?.node_id || '')
    const bindingType = String(binding?.credentialType || binding?.credential_type || '')
    const bindingCredentialId = String(binding?.credentialId || binding?.credential_id || '')
    if (bindingType && bindingCredentialId && (!bindingNodeId || bindingNodeId === nodeId)) {
      normalized[bindingType] = bindingCredentialId
    }
  })

  return normalized
}

function nodeCredentialReferences(node: any): Array<{
  credentialType: string
  displayName: string
  required: boolean
}> {
  const schemaRefs = Array.isArray(node?.data?.schema?.credentials)
    ? node.data.schema.credentials
    : []

  if (schemaRefs.length > 0) {
    return schemaRefs.map((ref: any) => ({
      credentialType: String(ref?.credentialType || ref?.credential_type || ''),
      displayName: String(ref?.displayName || ref?.display_name || ref?.credentialType || ''),
      required: ref?.required !== false,
    }))
  }

  const nodeType = node?.data?.schema?.name || node?.data?.type
  return NODE_CREDENTIAL_REQUIREMENTS[nodeType] || []
}

function buildDefaultProperties(schema: any): Record<string, any> {
  const defaults: Record<string, any> = {}
  if (schema?.properties) {
    schema.properties.forEach((p: any) => {
      if (p.default === undefined) return

      if (p.default !== null && typeof p.default === 'object') {
        defaults[p.name] = JSON.parse(JSON.stringify(p.default))
      } else {
        defaults[p.name] = p.default
      }
    })
  }
  return defaults
}

function findTypeEntry(typeName: string) {
  return nodeStore.nodeTypes.find((n: any) => n.schema?.name === typeName) || null
}

function makeUniqueNodeLabel(baseName: string): string {
  const existing = new Set(nodes.value.map((n: any) => n?.data?.label))
  if (!existing.has(baseName)) return baseName

  let i = 2
  while (existing.has(`${baseName} ${i}`)) i += 1
  return `${baseName} ${i}`
}

function createNodeId(): string {
  return globalThis.crypto?.randomUUID?.() ?? `node-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

function toCanvasNode(inode: any): any {
  const nodeType = inode.type || ''
  const typeEntry = findTypeEntry(nodeType)
  const schema = typeEntry?.schema || null

  const positionArray = Array.isArray(inode.position) ? inode.position : [0, 0]
  const properties = {
    ...buildDefaultProperties(schema),
    ...(inode.parameters || {}),
  }

  return {
    id: inode.id,
    type: 'custom',
    position: {
      x: Number(positionArray[0] ?? 0),
      y: Number(positionArray[1] ?? 0),
    },
    data: {
      type: nodeType,
      kind: typeEntry?.kind || (schema?.isTrigger ? 'trigger' : 'action'),
      isTrigger: !!(typeEntry?.isTrigger || schema?.isTrigger),
      label: inode.name,
      description: typeEntry?.description || schema?.description || '',
      typeVersion: Number(inode?.typeVersion ?? schema?.typeVersion ?? 1),
      disabled: !!inode?.disabled,
      status: null,
      schema,
      properties,
      credentials: normalizeNodeCredentials(inode.credentials, String(inode.id)),
      runData: null,
    },
  }
}

function toWorkflowNode(flowNode: any): any {
  const nodeType = flowNode?.data?.schema?.name || flowNode?.data?.type
  const nodeId = String(flowNode.id)
  const credentialBindings = Object.entries(flowNode?.data?.credentials || {})
    .filter(([credentialType, credentialId]) => !!credentialType && !!credentialId)
    .map(([credentialType, credentialId]) => ({
      nodeId,
      credentialType,
      credentialId,
    }))

  return {
    id: nodeId,
    name: flowNode?.data?.label || nodeId,
    type: nodeType,
    typeVersion: Number(flowNode?.data?.typeVersion ?? 1),
    position: [
      Number(flowNode?.position?.x ?? 0),
      Number(flowNode?.position?.y ?? 0),
    ],
    parameters: flowNode?.data?.properties || {},
    credentials: credentialBindings,
    disabled: !!flowNode?.data?.disabled,
  }
}

function buildWorkflowConnections(flowNodes: any[], flowEdges: any[]) {
  const byId = new Map(flowNodes.map((n: any) => [String(n.id), n]))
  const connections: Record<string, { main: any[][] }> = {}

  flowEdges.forEach((edge: any) => {
    const sourceNode = byId.get(String(edge.source))
    const targetNode = byId.get(String(edge.target))
    if (!sourceNode || !targetNode) return

    const sourceName = sourceNode?.data?.label || String(sourceNode.id)
    const targetName = targetNode?.data?.label || String(targetNode.id)
    if (!connections[sourceName]) connections[sourceName] = { main: [[]] }

    connections[sourceName].main[0].push({
      node: targetName,
      type: 'main',
      index: 0,
    })
  })

  return connections
}

function buildCanvasEdges(loadedNodes: any[], rawConnections: any): any[] {
  const byName = new Map(loadedNodes.map((n: any) => [n?.data?.label, n]))

  const loadedEdges: any[] = []
  Object.keys(rawConnections || {}).forEach((sourceName) => {
    const sourceConn = rawConnections[sourceName]
    const outputGroups = sourceConn?.main || sourceConn?.Main || []

    outputGroups.forEach((targets: any[]) => {
      ;(targets || []).forEach((target: any) => {
        const sourceNode = byName.get(sourceName)
        const targetNode = byName.get(target?.node)
        if (!sourceNode || !targetNode) return

        loadedEdges.push({
          id: `e-${sourceNode.id}-${targetNode.id}-${loadedEdges.length}`,
          source: sourceNode.id,
          target: targetNode.id,
          animated: true,
          style: { stroke: '#0ea5e9', strokeWidth: 2 },
        })
      })
    })
  })

  return loadedEdges
}

function onNodeClick({ node }: any) {
  selectedNode.value = node
}

function normalizeWorkflowTags(rawTags: string[]): string[] {
  const next: string[] = []
  const seen = new Set<string>()

  rawTags.forEach((rawTag) => {
    const tag = String(rawTag || '').trim()
    if (!tag) return

    const key = tag.toLowerCase()
    if (seen.has(key)) return

    seen.add(key)
    next.push(tag)
  })

  return next
}

function syncWorkflowMetadataFromRecord(workflow: any) {
  if (workflow) {
    workflowDraftName.value = String(workflow?.name || 'Untitled Workflow')
    workflowTags.value = normalizeWorkflowTags(
      Array.isArray(workflow?.tags)
        ? workflow.tags
            .map((tag: any) => String(tag?.name || tag || ''))
            .filter((tag: string) => tag.trim().length > 0)
        : [],
    )
    return
  }

  workflowDraftName.value = 'Untitled Workflow'
  workflowTags.value = []
}

function getCurrentWorkflowName() {
  const name = workflowDraftName.value.trim()
  return name.length > 0 ? name : 'Untitled Workflow'
}

function getCurrentWorkflowId(): string | null {
  if (route.params.id && route.params.id !== 'new') {
    return route.params.id as string
  }
  return workflowStore.activeWorkflow?.id || null
}

function openCredentialsPage() {
  router.push('/credentials')
}

function addWorkflowTag(tagName = workflowTagInput.value) {
  const normalized = normalizeWorkflowTags([...workflowTags.value, tagName])
  workflowTags.value = normalized
  workflowTagInput.value = ''
}

function removeWorkflowTag(tagName: string) {
  workflowTags.value = workflowTags.value.filter((tag) => tag !== tagName)
}

watch(
  () => workflowStore.activeWorkflow,
  (workflow) => {
    syncWorkflowMetadataFromRecord(workflow)
  },
  { immediate: true },
)

function isCredentialErrorMessage(message: string): boolean {
  const text = message.toLowerCase()
  return (
    text.includes('no credential found') ||
    text.includes('credential binding missing') ||
    text.includes('select a credential') ||
    text.includes('missing openai api key') ||
    text.includes('missing postgres credential fields') ||
    text.includes('go to /credentials') ||
    text.includes('/credentials and bind') ||
    (text.includes('credential') && text.includes('/credentials'))
  )
}

function matchesDisplayValue(actual: any, expected: any): boolean {
  return JSON.stringify(actual) === JSON.stringify(expected)
}

function isPropertyVisibleForNode(prop: any, properties: Record<string, any>): boolean {
  const show = prop?.displayOptions?.show
  if (!show) return true

  const sourceName = String(show?.property || '')
  if (!sourceName) return true

  const expectedValues = Array.isArray(show?.values) ? show.values : []
  if (expectedValues.length === 0) return true

  const actualValue = properties?.[sourceName]
  return expectedValues.some((expected: any) => matchesDisplayValue(actualValue, expected))
}

function isMissingRequiredValue(prop: any, value: any): boolean {
  if (value === undefined || value === null) return true

  if (typeof value === 'string') {
    return value.trim().length === 0
  }

  const type = String(prop?.type || '')
  if ((type === 'collection' || type === 'fixedCollection') && Array.isArray(value)) {
    return value.length === 0
  }
  if (
    (type === 'collection' || type === 'fixedCollection') &&
    typeof value === 'object' &&
    !Array.isArray(value)
  ) {
    return Object.keys(value).length === 0
  }

  return false
}

function nodeHasBoundRequiredCredential(node: any): boolean {
  const bound = node?.data?.credentials || {}
  return nodeCredentialReferences(node)
    .filter((ref) => ref.required)
    .some((ref) => String(bound?.[ref.credentialType] || '').trim().length > 0)
}

function shouldBypassTokenRequirement(node: any, prop: any): boolean {
  const propName = String(prop?.name || '')
  if (propName !== 'authToken' && propName !== 'botToken') {
    return false
  }

  return nodeHasBoundRequiredCredential(node)
}

function collectMissingRequiredProperties(node: any): string[] {
  const schemaProperties = Array.isArray(node?.data?.schema?.properties)
    ? node.data.schema.properties
    : []
  const properties = node?.data?.properties || {}

  return schemaProperties
    .filter((prop: any) => prop?.required === true)
    .filter((prop: any) => isPropertyVisibleForNode(prop, properties))
    .filter((prop: any) => !shouldBypassTokenRequirement(node, prop))
    .filter((prop: any) => isMissingRequiredValue(prop, properties[prop.name]))
    .map((prop: any) => String(prop?.displayName || prop?.name || 'unknown'))
}

function validateRequiredParameters(scopeNodeIds?: Set<string>): { ok: boolean; message?: string } {
  const missingByNode: string[] = []

  nodes.value.forEach((node: any) => {
    const nodeId = String(node?.id || '')
    if (scopeNodeIds && !scopeNodeIds.has(nodeId)) return

    const missing = collectMissingRequiredProperties(node)
    if (missing.length > 0) {
      const label = String(node?.data?.label || nodeId || 'Node')
      missingByNode.push(`${label} -> ${missing.join(', ')}`)
    }
  })

  if (missingByNode.length > 0) {
    return {
      ok: false,
      message: `Missing required parameters: ${missingByNode.join('; ')}`,
    }
  }

  return { ok: true }
}

function collectExecutionScopeForTarget(targetNodeId: string): Set<string> {
  const scope = new Set<string>()
  const targetId = String(targetNodeId || '')
  if (!targetId) return scope

  const inboundByTarget = new Map<string, string[]>()
  edges.value.forEach((edge: any) => {
    const source = String(edge?.source || '')
    const target = String(edge?.target || '')
    if (!source || !target) return

    if (!inboundByTarget.has(target)) {
      inboundByTarget.set(target, [])
    }
    inboundByTarget.get(target)!.push(source)
  })

  const stack = [targetId]
  while (stack.length > 0) {
    const current = stack.pop()!
    if (scope.has(current)) continue
    scope.add(current)

    const parents = inboundByTarget.get(current) || []
    parents.forEach((parentId) => {
      if (!scope.has(parentId)) {
        stack.push(parentId)
      }
    })
  }

  return scope
}

async function ensureRequiredCredentialsPresent(
  scopeNodeIds?: Set<string>,
): Promise<{ ok: boolean; message?: string }> {
  const requiredByNode: Array<{
    nodeId: string
    nodeLabel: string
    credentialType: string
    displayName: string
  }> = []

  nodes.value.forEach((node: any) => {
    const nodeId = String(node?.id || '')
    if (scopeNodeIds && !scopeNodeIds.has(nodeId)) return

    const refs = nodeCredentialReferences(node)
    refs
      .filter((ref) => ref.required)
      .forEach((ref) =>
        requiredByNode.push({
          nodeId,
          nodeLabel: String(node?.data?.label || node?.id || 'Node'),
          credentialType: ref.credentialType,
          displayName: ref.displayName || ref.credentialType,
        }),
      )
  })

  if (requiredByNode.length === 0) {
    return { ok: true }
  }

  try {
    const response = await listCredentials()
    const availableByType = new Map<string, Set<string>>()
    ;(response.data || []).forEach((cred: any) => {
      const type = String(cred?.credentialType || '')
      const id = String(cred?.id || '')
      if (!type || !id) return
      if (!availableByType.has(type)) {
        availableByType.set(type, new Set<string>())
      }
      availableByType.get(type)!.add(id)
    })

    const missing: string[] = []
    requiredByNode.forEach((required) => {
      const node = nodes.value.find((n: any) => String(n.id) === required.nodeId)
      const selectedCredentialId = String(node?.data?.credentials?.[required.credentialType] || '')
      const availableSet = availableByType.get(required.credentialType)

      if (!selectedCredentialId) {
        missing.push(`${required.nodeLabel} -> ${required.displayName}`)
        return
      }

      if (!availableSet || !availableSet.has(selectedCredentialId)) {
        missing.push(`${required.nodeLabel} -> ${required.displayName}`)
      }
    })

    if (missing.length > 0) {
      return {
        ok: false,
        message: `Missing credential bindings: ${missing.join(', ')}. Open the node panel and select credentials.`,
      }
    }

    return { ok: true }
  } catch {
    // Do not block execution if preflight lookup fails; runtime handler will still surface concrete errors.
    return { ok: true }
  }
}

function applyExecutionResult(result: any, targetNodeId?: string) {
  nodes.value.forEach((n) => {
    const nodeName = n.data.label
    const nodeResult = result?.data?.[nodeName]
    if (nodeResult) {
      n.data.status = nodeResult.success ? 'success' : 'error'
      n.data.runData = buildRunDataPayload({
        source: targetNodeId ? 'nodeTest' : 'workflowExecution',
        status: nodeResult.success ? 'success' : 'error',
        executionId: result?.id || null,
        preview: extractNodeOutputPreview(nodeResult),
        payload: nodeResult,
      })
    } else if (!targetNodeId) {
      n.data.status = normalizeExecutionStatus(result?.status)
    } else if (String(n.id) === targetNodeId) {
      const resolvedStatus = normalizeExecutionStatus(result?.status)
      n.data.status = resolvedStatus
      n.data.runData = buildRunDataPayload({
        source: 'nodeTest',
        status: resolvedStatus,
        executionId: result?.id || null,
        preview: resolvedStatus === 'success' ? null : extractNodeError(result, nodeName),
        payload: result?.data || null,
      })
    }

    if (selectedNode.value && String(selectedNode.value.id) === String(n.id)) {
      selectedNode.value = n
    }
  })
}

function normalizeExecutionStatus(status: any): string {
  const normalized = String(status || '').toLowerCase()
  if (normalized === 'success') return 'success'
  if (normalized === 'running') return 'running'
  if (normalized === 'waiting') return 'waiting'
  return 'error'
}

function setNodeRunData(nodeId: string, runData: any) {
  const match = nodes.value.find((n: any) => String(n.id) === String(nodeId))
  if (!match) return

  match.data.runData = runData
  if (selectedNode.value && String(selectedNode.value.id) === String(nodeId)) {
    selectedNode.value = match
  }
}

function buildRunDataPayload({
  source,
  status,
  executionId,
  preview,
  payload,
}: {
  source: string
  status: string
  executionId?: string | null
  preview?: string | null
  payload?: any
}) {
  return {
    source,
    status,
    executionId: executionId || null,
    updatedAt: new Date().toISOString(),
    preview: preview || null,
    payload: payload ?? null,
  }
}

const recentLiveExecutionEvents = computed(() => {
  return [...liveExecutionEvents.value]
    .sort((left, right) => right.sequence - left.sequence)
    .slice(0, 6)
})

const liveExecutionStatus = computed(() => {
  const latestEvent = [...liveExecutionEvents.value].sort((left, right) => right.sequence - left.sequence)[0]
  return latestEvent ? resolveExecutionStatusFromEvent(latestEvent) : null
})

const currentWorkflowHistory = computed<WorkflowHistoryEntry[]>(() => {
  const workflowId = getCurrentWorkflowId()
  if (!workflowId) return []
  return workflowStore.workflowHistory[workflowId] || []
})

const availableWorkspaceTags = computed(() => {
  return workflowStore.workflowTags.filter((tag) => !workflowTags.value.includes(tag.name)).slice(0, 8)
})

const editorNodeCount = computed(() => nodes.value.length)

const editorTriggerCount = computed(() => {
  return nodes.value.filter((node: any) => node?.data?.isTrigger).length
})

const editorVersion = computed(() => workflowStore.activeWorkflow?.summary?.latestVersion || 0)

function stopExecutionEventStream() {
  if (executionEventSource) {
    executionEventSource.close()
    executionEventSource = null
  }
}

function syncSelectedNodeReference(node: any) {
  if (selectedNode.value && String(selectedNode.value.id) === String(node.id)) {
    selectedNode.value = node
  }
}

function findNodeForExecutionEvent(event: ExecutionEvent, targetNodeId?: string) {
  if (event.nodeId) {
    const byId = nodes.value.find((node: any) => String(node.id) === String(event.nodeId))
    if (byId) return byId
  }

  if (event.nodeName) {
    const byLabel = nodes.value.find((node: any) => String(node?.data?.label || '') === event.nodeName)
    if (byLabel) return byLabel
  }

  if (targetNodeId) {
    return nodes.value.find((node: any) => String(node.id) === String(targetNodeId)) || null
  }

  return null
}

function applyExecutionEventUpdate(event: ExecutionEvent, targetNodeId?: string) {
  liveExecutionId.value = event.executionId
  liveExecutionEvents.value = mergeExecutionEvents(liveExecutionEvents.value, [event])

  const matchedNode = findNodeForExecutionEvent(event, targetNodeId)
  if (matchedNode) {
    let nextStatus: string | null = null
    if (event.eventType === 'nodeStarted') nextStatus = 'running'
    if (event.eventType === 'nodeFinished') {
      nextStatus = event.data?.success === false ? 'error' : 'success'
    }
    if (event.eventType === 'waiting') nextStatus = 'waiting'
    if (event.eventType === 'failed' || event.eventType === 'stopped') nextStatus = 'error'
    if (event.eventType === 'completed' && targetNodeId) nextStatus = 'success'

    if (nextStatus) {
      matchedNode.data.status = nextStatus
    }

    matchedNode.data.runData = buildRunDataPayload({
      source: targetNodeId ? 'nodeTest' : 'workflowExecution',
      status: resolveExecutionStatusFromEvent(event),
      executionId: event.executionId,
      preview: event.message,
      payload: event.data,
    })
    syncSelectedNodeReference(matchedNode)
  }

  if (event.eventType === 'waiting') {
    executionNotice.value = {
      type: 'success',
      message: event.message,
    }
  }

  if (event.eventType === 'failed' || event.eventType === 'stopped') {
    executionNotice.value = {
      type: 'error',
      message: event.message,
      showCredentialsAction: isCredentialErrorMessage(event.message),
    }
  }
}

function isTerminalExecutionStatus(status: any) {
  return ['success', 'failed', 'error', 'stopped', 'cancelled', 'waiting'].includes(
    String(status || '').toLowerCase(),
  )
}

async function awaitExecutionCompletion(executionId: string, targetNodeId?: string) {
  const timeoutMs = 120_000
  stopExecutionEventStream()

  try {
    await Promise.race([
      new Promise<void>((resolve, reject) => {
        const source = createExecutionEventSource(executionId)
        executionEventSource = source
        let settled = false

        const finalize = () => {
          if (settled) return
          settled = true
          if (executionEventSource === source) {
            executionEventSource = null
          }
          source.close()
        }

        source.addEventListener('execution', (rawEvent) => {
          const parsed = JSON.parse((rawEvent as MessageEvent<string>).data) as ExecutionEvent
          applyExecutionEventUpdate(parsed, targetNodeId)
          if (isTerminalExecutionEvent(parsed)) {
            finalize()
            resolve()
          }
        })

        source.onerror = () => {
          finalize()
          reject(new Error('Execution event stream disconnected.'))
        }
      }),
      new Promise<never>((_, reject) => {
        window.setTimeout(() => reject(new Error('Execution stream timed out after 120 seconds.')), timeoutMs)
      }),
    ])
  } catch (streamError) {
    stopExecutionEventStream()
    const startedAt = Date.now()
    while (Date.now() - startedAt < timeoutMs) {
      await new Promise((resolve) => setTimeout(resolve, 500))
      const response = await getExecution(executionId)
      const latest = response.data
      applyExecutionResult(latest, targetNodeId)
      if (isTerminalExecutionStatus(latest?.status)) {
        return latest
      }
    }

    throw streamError
  }

  const response = await getExecution(executionId)
  return response.data
}

function extractNodeError(result: any, nodeLabel?: string): string {
  if (nodeLabel && result?.data?.[nodeLabel]?.error) {
    return String(result.data[nodeLabel].error)
  }

  if (result?.data?.error) {
    return String(result.data.error)
  }

  const firstFailureEntry = Object.entries(result?.data || {}).find(
    ([, entry]: [string, any]) => entry && entry.success === false,
  ) as [string, any] | undefined

  if (firstFailureEntry?.[1]?.error) {
    const [failedNodeName, failedNode] = firstFailureEntry
    const error = String(failedNode.error)
    return failedNodeName ? `${failedNodeName}: ${error}` : error
  }

  return 'Execution failed'
}

function truncatePreview(text: string, maxLength = 160): string {
  const normalized = text.replace(/\s+/g, ' ').trim()
  if (normalized.length <= maxLength) return normalized
  return `${normalized.slice(0, maxLength - 3)}...`
}

function extractNodeOutputPreview(nodeResult: any): string | null {
  const firstBranch = Array.isArray(nodeResult?.outputs) ? nodeResult.outputs[0] : null
  const firstItem = Array.isArray(firstBranch) ? firstBranch[0] : null
  const firstJson = firstItem?.json && typeof firstItem.json === 'object' ? firstItem.json : null
  if (!firstJson) return null

  const responseText = [
    firstJson?.responseText,
    firstJson?.text,
    firstJson?.output,
    firstJson?.body?.responseText,
    firstJson?.body,
  ].find((value) => typeof value === 'string' && String(value).trim().length > 0)

  if (typeof responseText === 'string') {
    return `Preview: ${truncatePreview(responseText)}`
  }

  if (Array.isArray(firstJson?.models) && firstJson.models.length > 0) {
    const modelNames = firstJson.models
      .slice(0, 3)
      .map((model: any) => String(model))
      .join(', ')
    return `Models: ${modelNames}`
  }

  const keys = Object.keys(firstJson || {})
  if (keys.length > 0) {
    return `Output keys: ${keys.slice(0, 4).join(', ')}`
  }

  return null
}

function initializeWorkflowHistorySelection(history: WorkflowHistoryEntry[]) {
  selectedHistoryToVersion.value = history[0]?.version ?? null
  selectedHistoryFromVersion.value = history[1]?.version ?? history[0]?.version ?? null
}

async function refreshWorkflowHistory(workflowId: string, autoCompare = false) {
  const history = await workflowStore.fetchWorkflowHistory(workflowId)

  const knownVersions = new Set(history.map((entry) => entry.version))
  if (
    selectedHistoryFromVersion.value === null ||
    !knownVersions.has(selectedHistoryFromVersion.value) ||
    selectedHistoryToVersion.value === null ||
    !knownVersions.has(selectedHistoryToVersion.value)
  ) {
    initializeWorkflowHistorySelection(history)
  }

  if (
    autoCompare &&
    selectedHistoryFromVersion.value !== null &&
    selectedHistoryToVersion.value !== null &&
    selectedHistoryFromVersion.value !== selectedHistoryToVersion.value
  ) {
    activeHistoryDiff.value = await workflowStore.fetchWorkflowHistoryDiff(
      workflowId,
      selectedHistoryFromVersion.value,
      selectedHistoryToVersion.value,
    )
  } else if (history.length < 2) {
    activeHistoryDiff.value = null
  }

  return history
}

async function openHistoryInspector() {
  let workflowId = getCurrentWorkflowId()

  if (!workflowId) {
    const saved = await handleSave()
    workflowId = saved?.id || getCurrentWorkflowId()
  }

  if (!workflowId) return

  showHistoryPanel.value = true
  historyLoading.value = true
  try {
    await refreshWorkflowHistory(workflowId, true)
  } finally {
    historyLoading.value = false
  }
}

async function loadWorkflowHistoryDiff() {
  const workflowId = getCurrentWorkflowId()
  if (!workflowId) return
  if (
    selectedHistoryFromVersion.value === null ||
    selectedHistoryToVersion.value === null ||
    selectedHistoryFromVersion.value === selectedHistoryToVersion.value
  ) {
    return
  }

  historyDiffLoading.value = true
  try {
    activeHistoryDiff.value = await workflowStore.fetchWorkflowHistoryDiff(
      workflowId,
      selectedHistoryFromVersion.value,
      selectedHistoryToVersion.value,
    )
  } finally {
    historyDiffLoading.value = false
  }
}

async function handleSave() {
  const flow = toObject()
  const payloadNodes = flow.nodes.map((n: any) => toWorkflowNode(n))
  const payloadConnections = buildWorkflowConnections(flow.nodes, flow.edges)

  const payload = {
    id:
      route.params.id && route.params.id !== 'new'
        ? String(Array.isArray(route.params.id) ? route.params.id[0] : route.params.id)
        : undefined,
    name: getCurrentWorkflowName(),
    nodes: payloadNodes,
    connections: payloadConnections,
    settings: workflowStore.activeWorkflow?.settings || {},
    tags: [...workflowTags.value],
  }

  const saved = await workflowStore.saveWorkflow(payload)
  syncWorkflowMetadataFromRecord(saved)

  if (route.params.id === 'new' && saved?.id) {
    await router.replace(`/workflow/${saved.id}`)
  }

  if (showHistoryPanel.value && saved?.id) {
    await refreshWorkflowHistory(saved.id, true)
  }

  return saved
}

async function runWorkflow(
  workflowId: string,
  payload: Record<string, any> = {},
  targetNodeId?: string,
) {
  liveExecutionEvents.value = []
  nodes.value.forEach((n) => {
    if (!targetNodeId || String(n.id) === targetNodeId) {
      n.data.status = 'running'
    }
  })

  const execution = targetNodeId
    ? await workflowStore.executeWorkflowToNode(workflowId, targetNodeId, payload)
    : await workflowStore.executeWorkflow(workflowId, payload)

  const executionId = String(execution?.id || '')
  if (!executionId) {
    applyExecutionResult(execution, targetNodeId)
    return execution
  }
  liveExecutionId.value = executionId

  executionInProgress.value = true
  try {
    const latest = await awaitExecutionCompletion(executionId, targetNodeId)
    applyExecutionResult(latest, targetNodeId)
    return latest
  } finally {
    executionInProgress.value = false
  }
}

async function handleExecute() {
  if (workflowStore.loading) return

  executionNotice.value = null
  nodeTestState.value = null

  let workflowId = getCurrentWorkflowId()
  if (!workflowId) {
    const saved = await handleSave()
    workflowId = saved?.id || getCurrentWorkflowId()
  }

  if (!workflowId) {
    executionNotice.value = {
      type: 'error',
      message: 'Save workflow first before execution.',
    }
    return
  }

  const requiredParams = validateRequiredParameters()
  if (!requiredParams.ok) {
    executionNotice.value = {
      type: 'error',
      message: requiredParams.message || 'Missing required parameters.',
    }
    return
  }

  const preflight = await ensureRequiredCredentialsPresent()
  if (!preflight.ok) {
    executionNotice.value = {
      type: 'error',
      message: preflight.message || 'Missing required credentials.',
      showCredentialsAction: true,
    }
    return
  }

  try {
    const result = await runWorkflow(workflowId)
    if (result?.status === 'success') {
      executionNotice.value = {
        type: 'success',
        message: 'Workflow executed successfully.',
      }
    } else if (result?.status === 'waiting') {
      executionNotice.value = {
        type: 'success',
        message: 'Workflow is waiting for external resume input.',
      }
    } else {
      const message = extractNodeError(result)
      executionNotice.value = {
        type: 'error',
        message,
        showCredentialsAction: isCredentialErrorMessage(message),
      }
    }
  } catch (err: any) {
    nodes.value.forEach((n) => {
      n.data.status = 'error'
    })

    const message = err?.response?.data || err?.message || 'Execution failed.'
    executionNotice.value = {
      type: 'error',
      message,
      showCredentialsAction: isCredentialErrorMessage(String(message)),
    }
  }
}

async function handleTestNode(node: any) {
  if (!node) return

  executionNotice.value = null
  nodeTestState.value = {
    nodeId: node.id,
    status: 'running',
    message: `Testing '${node.data.label}'...`,
  }
  setNodeRunData(
    String(node.id),
    buildRunDataPayload({
      source: 'nodeTest',
      status: 'running',
      preview: `Testing '${node.data.label}'...`,
      payload: null,
    }),
  )

  let workflowId = getCurrentWorkflowId()
  if (!workflowId) {
    const saved = await handleSave()
    workflowId = saved?.id || getCurrentWorkflowId()
  }

  if (!workflowId) {
    nodeTestState.value = {
      nodeId: node.id,
      status: 'error',
      message: 'Save workflow first before testing this step.',
    }
    setNodeRunData(
      String(node.id),
      buildRunDataPayload({
        source: 'nodeTest',
        status: 'error',
        preview: 'Save workflow first before testing this step.',
        payload: { error: 'missing_workflow_id' },
      }),
    )
    return
  }

  const scopedNodes = collectExecutionScopeForTarget(String(node.id))
  if (!scopedNodes.has(String(node.id))) {
    scopedNodes.add(String(node.id))
  }
  const requiredParams = validateRequiredParameters(scopedNodes)
  if (!requiredParams.ok) {
    const message = requiredParams.message || 'Missing required parameters.'
    nodeTestState.value = {
      nodeId: node.id,
      status: 'error',
      message,
    }
    setNodeRunData(
      String(node.id),
      buildRunDataPayload({
        source: 'nodeTest',
        status: 'error',
        preview: message,
        payload: { error: message },
      }),
    )
    executionNotice.value = {
      type: 'error',
      message,
    }
    return
  }

  const preflight = await ensureRequiredCredentialsPresent(scopedNodes)
  if (!preflight.ok) {
    const message = preflight.message || 'Missing required credentials.'
    nodeTestState.value = {
      nodeId: node.id,
      status: 'error',
      message,
    }
    setNodeRunData(
      String(node.id),
      buildRunDataPayload({
        source: 'nodeTest',
        status: 'error',
        preview: message,
        payload: { error: message },
      }),
    )
    executionNotice.value = {
      type: 'error',
      message,
      showCredentialsAction: true,
    }
    return
  }

  try {
    const result = await runWorkflow(
      workflowId,
      { manual: true },
      String(node.id),
    )
    if (result?.status === 'waiting') {
      nodeTestState.value = {
        nodeId: node.id,
        status: 'success',
        message: 'Test reached a Wait state and is awaiting resume input.',
      }
      setNodeRunData(
        String(node.id),
        buildRunDataPayload({
          source: 'nodeTest',
          status: 'waiting',
          executionId: result?.id || null,
          preview: 'Test reached a Wait state and is awaiting resume input.',
          payload: result?.data || null,
        }),
      )
      return
    }

    const nodeResult: any = result?.data?.[node.data.label]

    if (nodeResult?.success) {
      const outputsCount = Array.isArray(nodeResult?.outputs)
        ? nodeResult.outputs.reduce(
            (count: number, branch: any) => count + (Array.isArray(branch) ? branch.length : 0),
            0,
          )
        : 0
      const preview = extractNodeOutputPreview(nodeResult)
      nodeTestState.value = {
        nodeId: node.id,
        status: 'success',
        message:
          preview || outputsCount > 0
            ? `Test passed. ${preview || `Outputs: ${outputsCount}`}`
            : 'Test passed. No output items returned.',
      }
      setNodeRunData(
        String(node.id),
        buildRunDataPayload({
          source: 'nodeTest',
          status: 'success',
          executionId: result?.id || null,
          preview:
            preview ||
            (outputsCount > 0 ? `Outputs: ${outputsCount}` : 'No output items returned.'),
          payload: nodeResult,
        }),
      )
    } else {
      const message = extractNodeError(result, node.data.label)
      nodeTestState.value = {
        nodeId: node.id,
        status: 'error',
        message,
      }
      setNodeRunData(
        String(node.id),
        buildRunDataPayload({
          source: 'nodeTest',
          status: 'error',
          executionId: result?.id || null,
          preview: message,
          payload: nodeResult || result?.data || null,
        }),
      )
      executionNotice.value = {
        type: 'error',
        message,
        showCredentialsAction: isCredentialErrorMessage(message),
      }
    }
  } catch (err: any) {
    const message = err?.response?.data || err?.message || 'Node test failed.'
    nodeTestState.value = {
      nodeId: node.id,
      status: 'error',
      message,
    }
    setNodeRunData(
      String(node.id),
      buildRunDataPayload({
        source: 'nodeTest',
        status: 'error',
        preview: String(message),
        payload: { error: String(message) },
      }),
    )
    executionNotice.value = {
      type: 'error',
      message,
      showCredentialsAction: isCredentialErrorMessage(String(message)),
    }
  }
}

function handleDeleteNode(nodeId: string) {
  nodes.value = nodes.value.filter((n) => String(n.id) !== nodeId)
  edges.value = edges.value.filter((e) => String(e.source) !== nodeId && String(e.target) !== nodeId)
  setNodes(nodes.value)
  setEdges(edges.value)
  if (selectedNode.value && String(selectedNode.value.id) === nodeId) {
    selectedNode.value = null
  }
}

onMounted(async () => {
  await Promise.all([nodeStore.fetchNodeTypes(), workflowStore.fetchWorkflowTags()])
  if (!(route.params.id && route.params.id !== 'new')) {
    syncWorkflowMetadataFromRecord(null)
    return
  }

  await workflowStore.fetchWorkflow(route.params.id as string)
  const activeWf = workflowStore.activeWorkflow
  if (!activeWf || !Array.isArray(activeWf.nodes)) return

  const loadedNodes = activeWf.nodes.map((n: any) => toCanvasNode(n))
  const loadedEdges = buildCanvasEdges(loadedNodes, activeWf.connections || {})

  setNodes(loadedNodes)
  setEdges(loadedEdges)
  nodes.value = loadedNodes
  edges.value = loadedEdges
})

onBeforeUnmount(() => {
  stopExecutionEventStream()
})

onConnect((params) => {
  addEdges([
    {
      ...params,
      animated: true,
      style: { stroke: '#0ea5e9', strokeWidth: 2 },
    },
  ])
})

function onDragStart(event: DragEvent, nodeTypeObj: any) {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', JSON.stringify(nodeTypeObj))
    event.dataTransfer.effectAllowed = 'move'
  }
}

function onDrop(event: DragEvent) {
  const nodeDataStr = event.dataTransfer?.getData('application/vueflow')
  if (!nodeDataStr) return

  const nodeSchema = JSON.parse(nodeDataStr)
  const position = screenToFlowCoordinate({ x: event.clientX, y: event.clientY })

  const typeName = nodeSchema.schema?.name || nodeSchema.type || ''
  const typeEntry = findTypeEntry(typeName)
  const schema = nodeSchema.schema || typeEntry?.schema || null

  const propertiesObj = buildDefaultProperties(schema)
  const label = makeUniqueNodeLabel(nodeSchema.name || schema?.displayName || schema?.display_name || typeName)

  const newNode = {
    id: createNodeId(),
    type: 'custom',
    position,
    data: {
      type: typeName,
      kind: typeEntry?.kind || nodeSchema.kind || (schema?.isTrigger || schema?.is_trigger ? 'trigger' : 'action'),
      isTrigger: !!(typeEntry?.isTrigger || nodeSchema.isTrigger || schema?.isTrigger || schema?.is_trigger),
      label,
      description: nodeSchema.description || schema?.description || '',
      typeVersion: Number(schema?.typeVersion ?? 1),
      disabled: false,
      status: null,
      schema,
      properties: propertiesObj,
      credentials: {},
      runData: null,
    },
  }

  nodes.value.push(newNode)
}
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-900">
    <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <section class="border-b border-slate-800 bg-slate-950/96 px-4 py-4 text-white shadow-[0_24px_60px_rgba(15,23,42,0.35)] backdrop-blur xl:px-6">
        <div class="flex flex-col gap-4">
          <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-3">
                <button
                  class="inline-flex items-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-xs font-semibold text-slate-200 transition hover:bg-white/10"
                  @click="router.push('/workflows')"
                >
                  <ChevronLeft class="h-4 w-4" />
                  All workflows
                </button>
                <span class="text-[11px] font-black uppercase tracking-[0.26em] text-slate-400">Workflow Studio</span>
              </div>

              <div class="mt-3 flex flex-col gap-3 xl:flex-row xl:items-center">
                <input
                  v-model="workflowDraftName"
                  type="text"
                  placeholder="Untitled Workflow"
                  class="w-full rounded-[1.25rem] border border-white/10 bg-white/6 px-4 py-3 text-2xl font-display font-black text-white outline-none transition focus:border-sky-400 focus:bg-white/10 xl:max-w-3xl"
                />
                <div class="flex flex-wrap gap-2">
                  <span class="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-bold text-slate-300">
                    {{ editorNodeCount }} nodes
                  </span>
                  <span class="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-bold text-slate-300">
                    {{ editorTriggerCount }} triggers
                  </span>
                  <span class="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-bold text-slate-300">
                    v{{ editorVersion }}
                  </span>
                </div>
              </div>
            </div>

            <div class="flex flex-wrap gap-2 xl:justify-end">
              <button
                @click="openHistoryInspector"
                class="inline-flex items-center gap-2 rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-bold text-slate-100 transition hover:bg-white/10"
              >
                <History class="h-4 w-4" />
                History
              </button>
              <button
                @click="handleSave"
                class="inline-flex items-center gap-2 rounded-2xl border border-white/10 bg-white px-4 py-3 text-sm font-bold text-slate-900 transition hover:bg-slate-100"
              >
                <Save class="h-4 w-4" />
                Save
              </button>
              <button
                @click="handleExecute"
                :disabled="workflowStore.loading || executionInProgress"
                class="inline-flex items-center gap-2 rounded-2xl bg-sky-500 px-4 py-3 text-sm font-black text-slate-950 transition hover:bg-sky-400 disabled:cursor-not-allowed disabled:opacity-70"
              >
                <Loader2 v-if="workflowStore.loading || executionInProgress" class="h-4 w-4 animate-spin" />
                <Play v-else class="h-4 w-4 fill-current" />
                {{ workflowStore.loading || executionInProgress ? 'Executing…' : 'Run Workflow' }}
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
            <div class="flex min-w-0 flex-1 flex-wrap gap-2">
              <span
                v-for="tagName in workflowTags"
                :key="tagName"
                class="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-bold text-slate-200"
              >
                <Tag class="h-3.5 w-3.5" />
                {{ tagName }}
                <button
                  type="button"
                  class="text-slate-400 transition hover:text-red-300"
                  @click="removeWorkflowTag(tagName)"
                >
                  <X class="h-3.5 w-3.5" />
                </button>
              </span>
              <span
                v-if="workflowTags.length === 0"
                class="rounded-full border border-dashed border-white/15 px-3 py-1.5 text-xs font-bold text-slate-500"
              >
                No tags assigned
              </span>
            </div>

            <div class="flex w-full flex-col gap-2 xl:w-auto xl:min-w-[30rem]">
              <div class="flex gap-2">
                <input
                  v-model="workflowTagInput"
                  type="text"
                  placeholder="Add workflow tag"
                  class="w-full rounded-2xl border border-white/10 bg-white/6 px-4 py-2.5 text-sm font-medium text-white outline-none transition focus:border-sky-400 focus:bg-white/10"
                  @keydown.enter.prevent="addWorkflowTag()"
                />
                <button
                  @click="addWorkflowTag()"
                  class="rounded-2xl border border-white/10 bg-white/5 px-4 py-2.5 text-sm font-bold text-slate-100 transition hover:bg-white/10"
                >
                  Add Tag
                </button>
              </div>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="workspaceTag in availableWorkspaceTags"
                  :key="workspaceTag.id"
                  type="button"
                  class="rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-bold text-slate-400 transition hover:border-sky-400 hover:text-white"
                  @click="addWorkflowTag(workspaceTag.name)"
                >
                  {{ workspaceTag.name }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <div class="min-h-0 flex-1 bg-slate-950">
        <div class="relative h-full overflow-hidden border-t border-slate-800 bg-white shadow-panel">
          <div
            v-if="executionNotice"
            :class="[
              'absolute left-4 right-4 top-4 z-20 rounded-2xl border px-4 py-3 text-sm font-medium shadow-sm md:left-6 md:right-6',
              executionNotice.type === 'success'
                ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
                : 'border-red-200 bg-red-50 text-red-700',
            ]"
          >
            {{ executionNotice.message }}
            <button
              v-if="executionNotice.showCredentialsAction"
              @click="openCredentialsPage"
              class="ml-3 inline-flex items-center rounded-xl border border-current/20 px-3 py-1.5 text-xs font-semibold transition hover:bg-white/40"
            >
              Open Credentials
            </button>
          </div>

          <div
            v-if="liveExecutionId || recentLiveExecutionEvents.length > 0"
            class="pointer-events-auto absolute right-4 top-20 z-20 hidden w-full max-w-sm overflow-hidden rounded-[1.6rem] border border-slate-200 bg-white shadow-panel lg:block"
          >
            <div class="border-b border-slate-100 px-4 py-3">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">
                    Execution Activity
                  </p>
                  <p class="mt-1 font-mono text-xs text-slate-500">
                    {{ liveExecutionId || 'pending' }}
                  </p>
                </div>
                <ExecutionStatusBadge :status="liveExecutionStatus || 'running'" />
              </div>
            </div>
            <div class="max-h-72 overflow-auto p-4">
              <ExecutionTimeline
                :events="recentLiveExecutionEvents"
                :compact="true"
                :limit="6"
                empty-message="Execution events will appear here while the workflow runs."
              />
            </div>
          </div>

          <div class="absolute bottom-5 right-5 z-20 pointer-events-auto">
            <button
              @click="showNodeCreator = true"
              class="inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white shadow-lg transition hover:bg-slate-800"
            >
              <Plus class="h-5 w-5" />
              Add Step
            </button>
          </div>

          <div class="h-full w-full bg-slate-50" @drop="onDrop" @dragover.prevent>
            <VueFlow
              v-model:nodes="nodes"
              v-model:edges="edges"
              @node-click="onNodeClick"
              :node-types="{ custom: CustomNode }"
              class="n8n-canvas"
              :default-viewport="{ zoom: 1, x: 0, y: 0 }"
              :min-zoom="0.2"
              :max-zoom="2"
            >
              <Background pattern-color="#d3dbe4" :gap="22" />
              <Controls position="bottom-left" class="!mb-6 !ml-6 overflow-hidden !rounded-2xl !border !border-slate-200 !bg-white !shadow-sm" />
              <MiniMap class="!mr-6 !mb-6 !rounded-2xl !border !border-slate-200 !bg-white !shadow-sm" />
            </VueFlow>
          </div>

          <div
            v-if="liveExecutionId || recentLiveExecutionEvents.length > 0"
            class="pointer-events-auto absolute inset-x-4 bottom-4 z-20 overflow-hidden rounded-[1.6rem] border border-slate-200 bg-white shadow-panel lg:hidden"
          >
            <div class="border-b border-slate-100 px-4 py-3">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">
                    Execution Activity
                  </p>
                  <p class="mt-1 font-mono text-xs text-slate-500">
                    {{ liveExecutionId || 'pending' }}
                  </p>
                </div>
                <ExecutionStatusBadge :status="liveExecutionStatus || 'running'" />
              </div>
            </div>
            <div class="max-h-56 overflow-auto p-4">
              <ExecutionTimeline
                :events="recentLiveExecutionEvents"
                :compact="true"
                :limit="6"
                empty-message="Execution events will appear here while the workflow runs."
              />
            </div>
          </div>
        </div>
      </div>

      <WorkflowHistoryPanel
        :show="showHistoryPanel"
        :loading="historyLoading"
        :diff-loading="historyDiffLoading"
        :history="currentWorkflowHistory"
        :diff="activeHistoryDiff"
        :from-version="selectedHistoryFromVersion"
        :to-version="selectedHistoryToVersion"
        @close="showHistoryPanel = false"
        @update:from-version="selectedHistoryFromVersion = $event"
        @update:to-version="selectedHistoryToVersion = $event"
        @load-diff="loadWorkflowHistoryDiff"
      />
    </div>

    <NodeCreator :show="showNodeCreator" @close="showNodeCreator = false" @dragstart="onDragStart" />

    <NodePanel
      :node="selectedNode"
      :test-state="nodeTestState"
      @close="selectedNode = null"
      @test-node="handleTestNode"
      @delete-node="handleDeleteNode"
    />
  </div>
</template>

<style>
.n8n-canvas {
  background-color: #f8fbfd;
  background-image:
    linear-gradient(rgba(148, 163, 184, 0.12) 1px, transparent 1px),
    linear-gradient(90deg, rgba(148, 163, 184, 0.12) 1px, transparent 1px);
  background-size: 24px 24px;
}

.vue-flow__edge-path {
  stroke-dasharray: 5;
  animation: dash 1s linear infinite;
}

@keyframes dash {
  from {
    stroke-dashoffset: 10;
  }
  to {
    stroke-dashoffset: 0;
  }
}

.vue-flow__handle {
  width: 12px !important;
  height: 12px !important;
  border-radius: 4px !important;
}

.vue-flow__controls-button {
  border-bottom: 1px solid #f1f5f9 !important;
  fill: #64748b !important;
  width: 40px !important;
  height: 40px !important;
}

.vue-flow__controls-button:hover {
  background-color: #f8fafc !important;
}
</style>
