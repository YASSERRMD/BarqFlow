<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { X, Play, Trash2, Info, Settings2 } from 'lucide-vue-next'
import { useNodeStore } from '../stores/nodes'
import { listCredentials } from '../features/credentials/api'
import NdvTabBar from '../features/ndv/components/NdvTabBar.vue'
import NdvParametersTab from '../features/ndv/components/NdvParametersTab.vue'
import NdvCredentialsTab from '../features/ndv/components/NdvCredentialsTab.vue'
import NdvSettingsTab from '../features/ndv/components/NdvSettingsTab.vue'
import NdvRunDataTab from '../features/ndv/components/NdvRunDataTab.vue'

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
const activeTab = ref('parameters')

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
  () => props.node?.id,
  () => {
    activeTab.value = 'parameters'
    localNotice.value = null
    ensureNodeCredentialMap()
    ensureNodePropertyMap()
  },
)

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

const currentRunData = computed(() => props.node?.data?.runData || null)

const tabs = computed(() => [
  {
    id: 'parameters',
    label: 'Parameters',
    badge:
      missingRequiredParameters.value.length > 0
        ? missingRequiredParameters.value.length
        : null,
  },
  {
    id: 'credentials',
    label: 'Credentials',
    badge:
      missingRequiredCredentials.value.length > 0
        ? missingRequiredCredentials.value.length
        : nodeCredentialRefs.value.length > 0
          ? nodeCredentialRefs.value.length
          : null,
  },
  {
    id: 'settings',
    label: 'Settings',
  },
  {
    id: 'runData',
    label: 'Run Data',
    badge:
      currentRunData.value?.status === 'error'
        ? '!'
        : currentRunData.value
          ? '1'
          : null,
  },
])

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
  activeTab.value = 'parameters'
}

function onDeleteNode() {
  if (!props.node?.id) return
  emit('delete-node', String(props.node.id))
}

function openCredentialsPage(credentialType?: string, displayName?: string) {
  const params = new URLSearchParams()
  if (credentialType) {
    params.set('credentialType', credentialType)
  }

  const currentPath = `${window.location.pathname}${window.location.search}`
  params.set('returnTo', currentPath)
  params.set(
    'nodeName',
    String(displayName || props.node?.data?.label || props.node?.data?.name || props.node?.id || 'node'),
  )

  window.location.assign(`/credentials?${params.toString()}`)
}
</script>

<template>
  <div
    v-if="node"
    class="fixed inset-0 z-40 bg-slate-900/20 transition-opacity"
    @click="emit('close')"
  ></div>

  <aside
    class="fixed inset-y-0 right-0 z-50 flex w-[450px] flex-col border-l border-slate-200 bg-white shadow-2xl transition-transform duration-300 ease-in-out"
    :class="node ? 'translate-x-0' : 'translate-x-full pointer-events-none'"
  >
    <div v-if="node" class="flex h-full flex-1 flex-col bg-slate-50">
      <div
        class="flex items-center justify-between border-b border-slate-200 bg-white px-6 py-4"
      >
        <div class="flex items-center gap-3">
          <div
            :class="[
              'flex h-8 w-8 items-center justify-center rounded',
              getCategoryColor(node.data.kind || node.data.type),
            ]"
          >
            <Settings2 class="h-5 w-5" />
          </div>
          <div>
            <h2 class="leading-tight text-lg font-bold text-slate-800">{{ node.data.label }}</h2>
            <div class="mt-0.5 text-[10px] font-semibold uppercase tracking-wider text-slate-500">
              {{ node.data.kind || node.data.type }} Node
            </div>
          </div>
        </div>
        <button
          class="rounded-lg p-1.5 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600"
          @click="emit('close')"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <NdvTabBar v-model="activeTab" :tabs="tabs" />

      <div class="flex-1 space-y-6 overflow-y-auto px-6 py-6">
        <div
          class="flex gap-3 rounded-lg border border-blue-100 bg-blue-50/50 p-3 text-sm text-blue-800"
        >
          <Info class="mt-0.5 h-4 w-4 shrink-0 text-blue-500" />
          <p>
            {{
              node.data.description ||
              'Configure this node to handle your workflow data processing requirements.'
            }}
          </p>
        </div>

        <NdvParametersTab
          v-if="activeTab === 'parameters'"
          :node="node"
          :node-schema="nodeSchema"
          :missing-required-parameters="missingRequiredParameters"
          :local-notice="localNotice"
          :property-type="propertyType"
          :is-property-visible="isPropertyVisible"
          :collection-drafts="collectionDrafts"
          :collection-errors="collectionErrors"
          :on-collection-input="onCollectionInput"
          :format-collection-draft="formatCollectionDraft"
        />

        <NdvCredentialsTab
          v-else-if="activeTab === 'credentials'"
          :node="node"
          :node-credential-refs="nodeCredentialRefs"
          :credential-options="credentialOptions"
          :credentials-loading="credentialsLoading"
          :credentials-error="credentialsError"
          :missing-required-credentials="missingRequiredCredentials"
          :credential-types-without-saved-options="credentialTypesWithoutSavedOptions"
          :open-credentials-page="openCredentialsPage"
        />

        <NdvSettingsTab
          v-else-if="activeTab === 'settings'"
          :node="node"
          :node-schema="nodeSchema"
          :documentation-url="documentationUrl"
        />

        <NdvRunDataTab
          v-else
          :test-state="panelTestState"
          :run-data="currentRunData"
        />
      </div>

      <div
        class="flex items-center justify-between gap-3 border-t border-slate-200 bg-white px-6 py-4"
      >
        <button
          class="rounded-lg border border-transparent p-2 text-slate-400 transition-colors hover:border-red-100 hover:bg-red-50 hover:text-red-600"
          title="Delete Node"
          @click="onDeleteNode"
        >
          <Trash2 class="h-5 w-5" />
        </button>
        <div class="flex gap-2">
          <button
            class="rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50"
            @click="applyMockData"
          >
            Mock Data
          </button>
          <button
            :disabled="isTesting || !canTestNode"
            class="flex items-center gap-2 rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white shadow-sm disabled:opacity-60 hover:bg-brand-700"
            @click="emit('test-node', node)"
          >
            <Play class="h-4 w-4 fill-current" />
            {{ isTesting ? 'Testing...' : 'Test Step' }}
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
