<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { X, Play, Trash2, Info, ExternalLink, Settings2 } from 'lucide-vue-next'
import { useNodeStore } from '../stores/nodes'
import { listCredentials } from '../features/credentials/api'

const nodeStore = useNodeStore()

interface NodeTestState {
  nodeId: string
  status: 'running' | 'success' | 'error'
  message: string
}

const props = defineProps<{
  node?: any
  testState?: NodeTestState | null
}>()

const emit = defineEmits<{
  (event: 'close'): void
  (event: 'test-node', node: any): void
  (event: 'delete-node', nodeId: string): void
}>()

const localNotice = ref<string | null>(null)
const credentialOptions = ref<Record<string, any[]>>({})
const credentialsLoading = ref(false)
const credentialsError = ref<string | null>(null)
const collectionDrafts = ref<Record<string, string>>({})
const collectionErrors = ref<Record<string, string>>({})
const activeCollectionNodeId = ref<string | null>(null)

const FALLBACK_NODE_CREDENTIALS: Record<
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

const nodeSchema = computed(() => {
  if (!props.node) return null

  const matchedType = nodeStore.nodeTypes.find(
    (n: any) => n.schema?.name === props.node?.data?.type,
  )
  if (matchedType) return matchedType.schema

  return props.node.data.schema || null
})

const documentationUrl = computed(() => {
  const schema = nodeSchema.value as any
  return schema?.documentationUrl || schema?.documentation_url || null
})

const nodeCredentialRefs = computed(() => {
  const schemaRefs = Array.isArray((nodeSchema.value as any)?.credentials)
    ? (nodeSchema.value as any).credentials
    : []

  if (schemaRefs.length > 0) {
    return schemaRefs.map((ref: any) => ({
      credentialType: String(ref?.credentialType || ref?.credential_type || ''),
      displayName: String(ref?.displayName || ref?.display_name || ref?.credentialType || ''),
      required: ref?.required !== false,
    }))
  }

  const nodeType = props.node?.data?.schema?.name || props.node?.data?.type
  return FALLBACK_NODE_CREDENTIALS[nodeType] || []
})

function ensureNodeCredentialMap() {
  if (!props.node) return
  if (!props.node.data.credentials || typeof props.node.data.credentials !== 'object') {
    props.node.data.credentials = {}
  }
}

function ensureNodePropertyMap() {
  if (!props.node) return
  if (!props.node.data.properties || typeof props.node.data.properties !== 'object') {
    props.node.data.properties = {}
  }
}

function propertyType(prop: any): string {
  const rawType = prop?.type
  if (typeof rawType === 'string') return rawType

  if (rawType && typeof rawType === 'object') {
    if (typeof rawType.type === 'string') return rawType.type
    if (rawType.fixedCollection) return 'fixedCollection'
  }

  return ''
}

function valueMatches(left: any, right: any): boolean {
  return JSON.stringify(left) === JSON.stringify(right)
}

function isPropertyVisible(prop: any): boolean {
  if (!props.node) return true
  const show = prop?.displayOptions?.show
  if (!show) return true

  const sourceProperty = String(show?.property || '')
  if (!sourceProperty) return true

  const actualValue = props.node.data.properties?.[sourceProperty]
  const expectedValues = Array.isArray(show?.values) ? show.values : []
  if (expectedValues.length === 0) return true

  return expectedValues.some((expected: any) => valueMatches(actualValue, expected))
}

function toCollectionDraft(value: any): string {
  if (value === undefined || value === null || value === '') {
    return '[]'
  }

  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (!trimmed) return '[]'

    try {
      return JSON.stringify(JSON.parse(trimmed), null, 2)
    } catch {
      return value
    }
  }

  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return '[]'
  }
}

function syncCollectionDrafts(force = false) {
  if (!props.node || !nodeSchema.value) return
  ensureNodePropertyMap()

  const nodeId = String(props.node.id || '')
  if (force || activeCollectionNodeId.value !== nodeId) {
    activeCollectionNodeId.value = nodeId
    collectionDrafts.value = {}
    collectionErrors.value = {}
  }

  const schemaProperties = Array.isArray((nodeSchema.value as any)?.properties)
    ? (nodeSchema.value as any).properties
    : []

  schemaProperties.forEach((prop: any) => {
    const type = propertyType(prop)
    if (type !== 'collection' && type !== 'fixedCollection') return

    const propName = String(prop?.name || '')
    if (!propName) return

    if (force || collectionDrafts.value[propName] === undefined) {
      const currentValue =
        props.node?.data?.properties?.[propName] !== undefined
          ? props.node.data.properties[propName]
          : prop?.default
      collectionDrafts.value[propName] = toCollectionDraft(currentValue)
    }
  })
}

function onCollectionInput(propName: string, event: Event) {
  if (!props.node) return
  const target = event.target as HTMLTextAreaElement | null
  const raw = target?.value ?? ''
  ensureNodePropertyMap()

  collectionDrafts.value[propName] = raw
  if (!raw.trim()) {
    props.node.data.properties[propName] = []
    delete collectionErrors.value[propName]
    return
  }

  try {
    props.node.data.properties[propName] = JSON.parse(raw)
    delete collectionErrors.value[propName]
  } catch {
    collectionErrors.value[propName] = 'Invalid JSON'
  }
}

function formatCollectionDraft(propName: string) {
  if (!props.node) return
  ensureNodePropertyMap()
  const raw = String(collectionDrafts.value[propName] || '').trim()

  if (!raw) {
    collectionDrafts.value[propName] = '[]'
    props.node.data.properties[propName] = []
    delete collectionErrors.value[propName]
    return
  }

  try {
    const parsed = JSON.parse(raw)
    props.node.data.properties[propName] = parsed
    collectionDrafts.value[propName] = JSON.stringify(parsed, null, 2)
    delete collectionErrors.value[propName]
  } catch {
    collectionErrors.value[propName] = 'Invalid JSON. Expected array or object.'
  }
}

async function loadCredentialOptions() {
  if (!props.node) return
  ensureNodeCredentialMap()
  credentialOptions.value = {}
  credentialsError.value = null

  if (nodeCredentialRefs.value.length === 0) return

  credentialsLoading.value = true
  try {
    await Promise.all(
      nodeCredentialRefs.value.map(async (refInfo: any) => {
        const response = await listCredentials({ type: refInfo.credentialType })
        credentialOptions.value[refInfo.credentialType] = response.data || []
      }),
    )
  } catch (err: any) {
    credentialsError.value = err?.response?.data || err?.message || 'Failed to load credentials.'
  } finally {
    credentialsLoading.value = false
  }
}

watch(
  () =>
    `${String(props.node?.id || '')}:${nodeCredentialRefs.value
      .map((r: any) => r.credentialType)
      .join(',')}`,
  () => {
    loadCredentialOptions()
  },
  { immediate: true },
)

watch(
  () => `${String(props.node?.id || '')}:${String((nodeSchema.value as any)?.name || '')}`,
  () => {
    syncCollectionDrafts(true)
  },
  { immediate: true },
)

function getCategoryColor(type: string) {
  switch (type) {
    case 'trigger':
      return 'bg-purple-100 text-purple-700'
    case 'logic':
      return 'bg-amber-100 text-amber-700'
    case 'manipulation':
      return 'bg-blue-100 text-blue-700'
    default:
      return 'bg-brand-100 text-brand-700'
  }
}

const panelTestState = computed(() => {
  if (!props.node || !props.testState) return null
  if (props.testState.nodeId !== props.node.id) return null
  return props.testState
})

const isTesting = computed(() => panelTestState.value?.status === 'running')

function isMissingRequiredPropertyValue(prop: any, value: any): boolean {
  if (value === undefined || value === null) return true

  const type = propertyType(prop)
  if (typeof value === 'string') {
    return value.trim().length === 0
  }

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

function hasSelectedRequiredCredentialForNode(): boolean {
  if (!props.node) return false
  const selected = props.node?.data?.credentials || {}
  return nodeCredentialRefs.value
    .filter((ref: any) => ref?.required)
    .some((ref: any) => String(selected?.[ref.credentialType] || '').trim().length > 0)
}

function shouldBypassTokenPropertyRequirement(prop: any): boolean {
  const propName = String(prop?.name || '')
  if (propName !== 'authToken' && propName !== 'botToken') {
    return false
  }

  return hasSelectedRequiredCredentialForNode()
}

const missingRequiredParameters = computed(() => {
  if (!props.node || !nodeSchema.value) return []

  const schemaProperties = Array.isArray((nodeSchema.value as any)?.properties)
    ? (nodeSchema.value as any).properties
    : []
  const properties = props.node?.data?.properties || {}

  return schemaProperties
    .filter((prop: any) => prop?.required === true)
    .filter((prop: any) => isPropertyVisible(prop))
    .filter((prop: any) => !shouldBypassTokenPropertyRequirement(prop))
    .filter((prop: any) => isMissingRequiredPropertyValue(prop, properties[prop.name]))
    .map((prop: any) => String(prop?.displayName || prop?.name || 'unknown'))
})

const missingRequiredCredentials = computed(() => {
  if (!props.node) return []
  const credentials = props.node?.data?.credentials || {}

  return nodeCredentialRefs.value
    .filter((ref: any) => ref?.required)
    .filter((ref: any) => !String(credentials?.[ref.credentialType] || '').trim())
    .map((ref: any) => String(ref.displayName || ref.credentialType || 'credential'))
})

const credentialTypesWithoutSavedOptions = computed(() => {
  return nodeCredentialRefs.value
    .filter((ref: any) => ref?.required)
    .filter((ref: any) => (credentialOptions.value[ref.credentialType] || []).length === 0)
    .map((ref: any) => String(ref.displayName || ref.credentialType || 'credential'))
})

const canTestNode = computed(
  () =>
    missingRequiredParameters.value.length === 0 &&
    missingRequiredCredentials.value.length === 0,
)

function mockValueForProperty(prop: any) {
  const type = propertyType(prop)
  const name = String(prop?.name || '').toLowerCase()

  if (type === 'options') {
    return prop?.options?.[0]?.value ?? null
  }

  if (type === 'boolean') {
    return true
  }

  if (type === 'string' || type === 'text') {
    if (name.includes('url')) return 'http://localhost:11434'
    if (name.includes('model')) return 'llama3.2'
    if (name.includes('prompt')) return 'Write one sentence about workflow automation.'
    if (name.includes('api') && name.includes('key')) return 'test-key'
    return 'sample-value'
  }

  if (type === 'collection' || type === 'fixedCollection') {
    return []
  }

  return null
}

function applyMockData() {
  if (!props.node || !nodeSchema.value) return

  const schema: any = nodeSchema.value
  const target = props.node.data.properties || {}
  let updated = 0

  for (const prop of schema.properties || []) {
    const current = target[prop.name]
    if (current !== undefined && current !== null && String(current).length > 0) {
      continue
    }

    const next = mockValueForProperty(prop)
    if (next !== null) {
      target[prop.name] = next
      updated += 1
    }
  }

  props.node.data.properties = { ...target }
  syncCollectionDrafts()
  localNotice.value =
    updated > 0
      ? `Injected mock values for ${updated} parameter(s).`
      : 'No empty parameters found for mock injection.'
}

function onDeleteNode() {
  if (!props.node?.id) return
  emit('delete-node', String(props.node.id))
}

function openCredentialsPage() {
  window.location.assign('/credentials')
}
</script>

<template>
  <div v-if="node" class="fixed inset-0 bg-slate-900/20 z-40 transition-opacity" @click="emit('close')"></div>

  <aside
    class="fixed inset-y-0 right-0 w-[450px] bg-white shadow-2xl flex flex-col transition-transform duration-300 ease-in-out z-50 border-l border-slate-200"
    :class="node ? 'translate-x-0' : 'translate-x-full pointer-events-none'"
  >
    <div v-if="node" class="flex-1 flex flex-col h-full bg-slate-50">
      <div class="px-6 py-4 border-b border-slate-200 bg-white flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div :class="['w-8 h-8 rounded flex items-center justify-center', getCategoryColor(node.data.kind || node.data.type)]">
            <Settings2 class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-lg font-bold text-slate-800 leading-tight">{{ node.data.label }}</h2>
            <div class="text-[10px] font-semibold uppercase tracking-wider text-slate-500 mt-0.5">
              {{ node.data.kind || node.data.type }} Node
            </div>
          </div>
        </div>
        <button @click="emit('close')" class="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-6 py-6 space-y-6">
        <div class="bg-blue-50/50 border border-blue-100 rounded-lg p-3 flex gap-3 text-sm text-blue-800">
          <Info class="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
          <p>{{ node.data.description || 'Configure this node to handle your workflow data processing requirements.' }}</p>
        </div>

        <div class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
          <h3 class="font-semibold text-slate-800 mb-4">Parameters</h3>

          <div v-if="nodeSchema && nodeSchema.properties" class="space-y-4">
            <template v-for="(prop, pIdx) in nodeSchema.properties" :key="pIdx">
              <div v-if="isPropertyVisible(prop)">
                <label class="block text-sm font-medium text-slate-700 mb-1.5">{{ prop.displayName }}</label>

                <div v-if="propertyType(prop) === 'string'" class="relative">
                  <input
                    v-model="node.data.properties[prop.name]"
                    type="text"
                    :placeholder="prop.placeholder || ''"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm"
                  />
                </div>

                <div v-else-if="propertyType(prop) === 'text'" class="relative">
                  <textarea
                    v-model="node.data.properties[prop.name]"
                    rows="5"
                    :placeholder="prop.placeholder || ''"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm font-mono"
                  />
                </div>

                <div v-else-if="propertyType(prop) === 'number'" class="relative">
                  <input
                    v-model.number="node.data.properties[prop.name]"
                    type="number"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm"
                  />
                </div>

                <div v-else-if="propertyType(prop) === 'options'">
                  <select
                    v-model="node.data.properties[prop.name]"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm"
                  >
                    <option v-for="opt in prop.options || []" :key="opt.value" :value="opt.value">
                      {{ opt.name }}
                    </option>
                  </select>
                </div>

                <div v-else-if="propertyType(prop) === 'boolean'" class="flex items-center mt-2">
                  <input
                    v-model="node.data.properties[prop.name]"
                    type="checkbox"
                    class="w-4 h-4 text-brand-600 bg-white border-slate-300 rounded focus:ring-brand-500"
                  />
                  <label class="ml-2 text-sm text-slate-700">{{ prop.description || prop.displayName }}</label>
                </div>

                <div v-else-if="propertyType(prop) === 'collection' || propertyType(prop) === 'fixedCollection'" class="space-y-2">
                  <textarea
                    :value="collectionDrafts[prop.name] || '[]'"
                    rows="6"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm font-mono"
                    @input="onCollectionInput(prop.name, $event)"
                    @blur="formatCollectionDraft(prop.name)"
                  />
                  <p class="text-xs text-slate-500">Provide valid JSON (array or object).</p>
                  <p v-if="collectionErrors[prop.name]" class="text-xs text-red-600">
                    {{ collectionErrors[prop.name] }}
                  </p>
                </div>

                <p v-if="prop.description && propertyType(prop) !== 'boolean'" class="mt-1 text-xs text-slate-500">
                  {{ prop.description }}
                </p>
                <p v-if="prop.hint" class="mt-1 text-xs text-slate-400">
                  Hint: {{ prop.hint }}
                </p>
              </div>
            </template>
          </div>

          <div v-else-if="(node.data.kind || node.data.type) === 'action'" class="space-y-4">
            <p class="text-xs text-amber-600 bg-amber-50 p-2 rounded border border-amber-200">No dynamic schema available from backend. Using fallback rendering.</p>
          </div>
        </div>

        <div v-if="nodeCredentialRefs.length > 0" class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
          <h3 class="font-semibold text-slate-800 mb-4">Credentials</h3>
          <div v-if="credentialsError" class="text-xs text-red-600 bg-red-50 border border-red-200 rounded-md px-3 py-2">
            {{ credentialsError }}
          </div>
          <div class="space-y-4">
            <div v-for="ref in nodeCredentialRefs" :key="ref.credentialType">
              <label class="block text-sm font-medium text-slate-700 mb-1.5">
                {{ ref.displayName }}
              </label>
              <select
                v-model="node.data.credentials[ref.credentialType]"
                :disabled="credentialsLoading"
                class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm disabled:opacity-70"
              >
                <option value="">Select credential</option>
                <option
                  v-for="cred in credentialOptions[ref.credentialType] || []"
                  :key="cred.id"
                  :value="cred.id"
                >
                  {{ cred.name }}
                </option>
              </select>
              <p
                v-if="ref.required && !node.data.credentials[ref.credentialType]"
                class="mt-1 text-xs text-amber-600"
              >
                This credential is required.
              </p>
              <p
                v-if="!credentialsLoading && (credentialOptions[ref.credentialType] || []).length === 0"
                class="mt-1 text-xs text-red-600"
              >
                No saved {{ ref.displayName }} credential found.
                <a href="/credentials" class="underline font-semibold">Add one in Credentials</a>.
              </p>
            </div>
          </div>
        </div>

        <div v-if="documentationUrl">
          <a :href="documentationUrl" target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1.5 text-sm font-medium text-brand-600 hover:text-brand-700">
            <ExternalLink class="w-4 h-4" /> View Documentation
          </a>
        </div>

        <div
          v-if="missingRequiredParameters.length > 0 || missingRequiredCredentials.length > 0"
          class="bg-amber-50 border border-amber-200 rounded-lg px-3 py-2 text-xs text-amber-700"
        >
          <p v-if="missingRequiredParameters.length > 0">
            Missing required parameters: {{ missingRequiredParameters.join(', ') }}
          </p>
          <p v-if="missingRequiredCredentials.length > 0">
            Missing required credentials: {{ missingRequiredCredentials.join(', ') }}
          </p>
          <p v-if="credentialTypesWithoutSavedOptions.length > 0">
            No saved credentials available for:
            {{ credentialTypesWithoutSavedOptions.join(', ') }}.
            <button type="button" class="underline font-semibold" @click="openCredentialsPage">
              Open Credentials
            </button>
          </p>
        </div>
      </div>

      <div
        v-if="panelTestState"
        :class="[
          'mx-6 mb-3 px-3 py-2 rounded-lg text-xs font-medium',
          panelTestState.status === 'success' && 'bg-green-50 text-green-700 border border-green-200',
          panelTestState.status === 'error' && 'bg-red-50 text-red-700 border border-red-200',
          panelTestState.status === 'running' && 'bg-blue-50 text-blue-700 border border-blue-200',
        ]"
      >
        {{ panelTestState.message }}
      </div>

      <div v-if="localNotice" class="mx-6 mb-3 px-3 py-2 rounded-lg text-xs font-medium bg-amber-50 text-amber-700 border border-amber-200">
        {{ localNotice }}
      </div>

      <div class="px-6 py-4 border-t border-slate-200 bg-white flex items-center justify-between gap-3">
        <button
          @click="onDeleteNode"
          class="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors border border-transparent hover:border-red-100"
          title="Delete Node"
        >
          <Trash2 class="w-5 h-5" />
        </button>
        <div class="flex gap-2">
          <button
            @click="applyMockData"
            class="px-4 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-lg hover:bg-slate-50 shadow-sm"
          >
            Mock Data
          </button>
          <button
            @click="emit('test-node', node)"
            :disabled="isTesting || !canTestNode"
            class="px-4 py-2 text-sm font-medium text-white bg-brand-600 rounded-lg hover:bg-brand-700 shadow-sm flex items-center gap-2 disabled:opacity-60"
          >
            <Play class="w-4 h-4 fill-current" /> {{ isTesting ? 'Testing...' : 'Test Step' }}
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
