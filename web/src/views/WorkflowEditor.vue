<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import { Plus, Play, Save, Loader2 } from 'lucide-vue-next'

import CustomNode from '../components/CustomNode.vue'
import NodeCreator from '../components/NodeCreator.vue'
import NodePanel from '../components/NodePanel.vue'
import { useWorkflowStore } from '../stores/workflows'
import { useNodeStore } from '../stores/nodes'
import { useRoute, useRouter } from 'vue-router'
import api from '../api'

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
      kind: typeEntry?.kind || (schema?.is_trigger ? 'trigger' : 'action'),
      isTrigger: !!(typeEntry?.isTrigger || schema?.is_trigger),
      label: inode.name,
      description: typeEntry?.description || schema?.description || '',
      status: null,
      schema,
      properties,
      credentials: normalizeNodeCredentials(inode.credentials, String(inode.id)),
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
    typeVersion: 1.0,
    position: [
      Number(flowNode?.position?.x ?? 0),
      Number(flowNode?.position?.y ?? 0),
    ],
    parameters: flowNode?.data?.properties || {},
    credentials: credentialBindings,
    disabled: false,
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

function getCurrentWorkflowName() {
  return workflowStore.activeWorkflow?.name || 'My New Workflow'
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
    const response = await api.get('/credentials')
    const availableByType = new Map<string, Set<string>>()
    ;(response.data || []).forEach((cred: any) => {
      const type = String(cred?.cred_type || cred?.credential_type || '')
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
    } else if (!targetNodeId) {
      n.data.status = result?.status === 'success' ? 'success' : 'error'
    } else if (String(n.id) === targetNodeId) {
      n.data.status = result?.status === 'success' ? 'success' : 'error'
    }
  })
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

async function handleSave() {
  const flow = toObject()
  const payloadNodes = flow.nodes.map((n: any) => toWorkflowNode(n))
  const payloadConnections = buildWorkflowConnections(flow.nodes, flow.edges)

  const payload = {
    id: route.params.id !== 'new' ? route.params.id : undefined,
    name: getCurrentWorkflowName(),
    nodes: payloadNodes,
    connections: payloadConnections,
    settings: workflowStore.activeWorkflow?.settings || {},
  }

  const saved = await workflowStore.saveWorkflow(payload)

  if (route.params.id === 'new' && saved?.id) {
    await router.replace(`/workflow/${saved.id}`)
  }

  return saved
}

async function runWorkflow(
  workflowId: string,
  payload: Record<string, any> = {},
  targetNodeId?: string,
) {
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

  const isTerminal = (status: string) =>
    ['success', 'failed', 'error', 'stopped', 'cancelled', 'crashed', 'waiting'].includes(
      status.toLowerCase(),
    )

  executionInProgress.value = true
  try {
    let latest = execution
    const startedAt = Date.now()
    const timeoutMs = 120_000

    while (!isTerminal(String(latest?.status || ''))) {
      if (Date.now() - startedAt > timeoutMs) {
        latest = {
          ...latest,
          status: 'failed',
          data: { error: 'Execution polling timed out after 120 seconds.' },
        }
        break
      }

      await new Promise((resolve) => setTimeout(resolve, 500))
      const response = await api.get(`/executions/${executionId}`)
      latest = response.data
    }

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
      return
    }

    const nodeResult = result?.data?.[node.data.label]

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
    } else {
      const message = extractNodeError(result, node.data.label)
      nodeTestState.value = {
        nodeId: node.id,
        status: 'error',
        message,
      }
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
  await nodeStore.fetchNodeTypes()
  if (!(route.params.id && route.params.id !== 'new')) return

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
  const label = makeUniqueNodeLabel(nodeSchema.name || schema?.display_name || typeName)

  const newNode = {
    id: createNodeId(),
    type: 'custom',
    position,
    data: {
      type: typeName,
      kind: typeEntry?.kind || nodeSchema.kind || (schema?.is_trigger ? 'trigger' : 'action'),
      isTrigger: !!(typeEntry?.isTrigger || nodeSchema.isTrigger || schema?.is_trigger),
      label,
      description: nodeSchema.description || schema?.description || '',
      status: null,
      schema,
      properties: propertiesObj,
      credentials: {},
    },
  }

  nodes.value.push(newNode)
}
</script>

<template>
  <div class="h-full w-full flex overflow-hidden bg-transparent">
    <div class="flex-1 relative overflow-hidden">
      <div class="absolute top-4 left-4 right-4 flex justify-between items-center z-10 pointer-events-none">
        <div class="bg-white rounded-lg shadow-sm border border-slate-200 px-4 py-2 flex items-center gap-3 pointer-events-auto">
          <div>
            <h1 class="font-bold text-slate-800 text-base leading-tight">{{ getCurrentWorkflowName() }}</h1>
            <p class="text-xs text-slate-500">Builder mode</p>
          </div>
        </div>

        <div class="flex gap-2 pointer-events-auto">
          <button
            @click="handleSave"
            class="bg-white hover:bg-slate-50 border border-slate-200 text-slate-700 px-4 py-2 rounded-lg flex items-center gap-2 transition-colors font-semibold text-sm shadow-sm"
          >
            <Save class="w-4 h-4" /> Save
          </button>
          <button
            @click="handleExecute"
            :disabled="workflowStore.loading || executionInProgress"
            class="bg-brand-500 hover:bg-brand-600 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors font-semibold text-sm disabled:opacity-70 shadow-sm"
          >
            <Loader2 v-if="workflowStore.loading || executionInProgress" class="w-4 h-4 animate-spin" />
            <Play v-else class="w-4 h-4 fill-current" />
            {{ workflowStore.loading || executionInProgress ? 'Executing...' : 'Execute Workflow' }}
          </button>
        </div>
      </div>

      <div
        v-if="executionNotice"
        :class="[
          'absolute top-20 left-4 right-4 z-20 p-3 rounded-lg border text-sm font-medium pointer-events-auto',
          executionNotice.type === 'success'
            ? 'bg-green-50 border-green-200 text-green-700'
            : 'bg-red-50 border-red-200 text-red-700',
        ]"
      >
        {{ executionNotice.message }}
        <button
          v-if="executionNotice.showCredentialsAction"
          @click="openCredentialsPage"
          class="ml-3 inline-flex items-center px-2.5 py-1 rounded-md text-xs font-semibold border border-current/30 hover:bg-white/30"
        >
          Open Credentials
        </button>
      </div>

      <div class="absolute bottom-6 right-6 z-10 pointer-events-auto">
        <button
          @click="showNodeCreator = true"
          class="w-12 h-12 bg-brand-500 shadow-lg text-white rounded-full flex items-center justify-center hover:bg-brand-600 hover:scale-105 transition-all"
        >
          <Plus class="w-6 h-6" />
        </button>
      </div>

      <div class="h-full w-full bg-[#f8f9fa]" @drop="onDrop" @dragover.prevent>
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
          <Background pattern-color="#ccc" :gap="20" />
          <Controls position="bottom-left" class="!bg-white !border-slate-200 !shadow-sm !rounded-md overflow-hidden mb-6 ml-6" />
          <MiniMap class="!bg-white !border-slate-200 !shadow-sm !rounded-md mr-20 mb-6" />
        </VueFlow>
      </div>
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
  background-image: radial-gradient(#e5e7eb 1px, transparent 1px);
  background-size: 20px 20px;
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
}

.vue-flow__controls-button:hover {
  background-color: #f8fafc !important;
}
</style>
