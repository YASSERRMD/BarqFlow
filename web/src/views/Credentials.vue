<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowRight,
  CheckCircle2,
  ExternalLink,
  FlaskConical,
  KeyRound,
  Loader2,
  Pencil,
  PlugZap,
  Plus,
  RefreshCcw,
  Search,
  Shield,
  Shuffle,
  Trash2,
  X,
  XCircle,
} from 'lucide-vue-next'
import {
  createCredential,
  deleteCredentialById,
  listCredentials,
  listCredentialTypes,
  rotateCredential,
  startCredentialOAuthConnect,
  testCredentialType,
  testSavedCredentialById,
  updateCredential,
} from '../features/credentials/api'
import {
  CREDENTIAL_QUICK_STARTS,
  credentialAuthKind,
  credentialStatusPresentation,
  credentialSupportsOAuthConnect,
  formatDateTime,
  formatRelativeTime,
  isSecretCredentialField,
} from '../features/credentials/helpers'
import type {
  CredentialSummary,
  CredentialTypeContract,
  CredentialValidationResult,
  NodeProperty,
} from '../types/contracts'

const route = useRoute()
const router = useRouter()

const credentials = ref<CredentialSummary[]>([])
const credentialTypes = ref<CredentialTypeContract[]>([])
const credentialsLoading = ref(false)
const credentialTypesLoading = ref(false)
const pageError = ref<string | null>(null)
const pageNotice = ref<string | null>(null)

const searchTerm = ref('')
const authFilter = ref<'all' | 'oauth' | 'token' | 'database' | 'custom'>('all')
const statusFilter = ref<'all' | 'validated' | 'attention' | 'untested'>('all')

const isModalOpen = ref(false)
const modalMode = ref<'create' | 'edit' | 'rotate'>('create')
const editingCredentialId = ref<string | null>(null)
const selectedTypeName = ref('')
const draftCredentialName = ref('')
const draftCredentialData = ref<Record<string, unknown>>({})
const modalError = ref<string | null>(null)
const modalSuccess = ref<string | null>(null)
const draftValidation = ref<CredentialValidationResult | null>(null)
const saveLoading = ref(false)
const testLoading = ref(false)
const rowLoading = ref<Record<string, string>>({})
const routedIntentHandled = ref('')

const credentialTypeMap = computed(() => {
  return new Map(credentialTypes.value.map((type) => [type.name, type]))
})

const selectedType = computed(() => {
  return credentialTypeMap.value.get(selectedTypeName.value) || null
})

const quickStarts = computed(() => {
  return CREDENTIAL_QUICK_STARTS.map((entry) => ({
    ...entry,
    type: credentialTypeMap.value.get(entry.credentialType) || null,
  })).filter((entry) => entry.type)
})

const requestedCredentialType = computed(() => {
  const raw = route.query.credentialType ?? route.query.type
  return typeof raw === 'string' ? raw.trim() : ''
})

const requestedReturnTo = computed(() => {
  const raw = route.query.returnTo
  return typeof raw === 'string' && raw.trim() ? raw.trim() : null
})

const requestedNodeName = computed(() => {
  const raw = route.query.nodeName ?? route.query.nodeId
  return typeof raw === 'string' && raw.trim() ? raw.trim() : null
})

const selectedTypeAuthKind = computed(() => credentialAuthKind(selectedType.value))
const canConnectSelectedType = computed(() => credentialSupportsOAuthConnect(selectedType.value))
const isCreateMode = computed(() => modalMode.value === 'create')
const isEditMode = computed(() => modalMode.value === 'edit')
const isRotateMode = computed(() => modalMode.value === 'rotate')

const filteredCredentials = computed(() => {
  const query = searchTerm.value.trim().toLowerCase()

  return credentials.value.filter((credential) => {
    const credentialType = credentialTypeMap.value.get(credential.credentialType) || null
    const authKind = credentialAuthKind(credentialType)
    const status = credentialStatusPresentation(credential, credentialType)
    const matchesQuery =
      query.length === 0 ||
      credential.name.toLowerCase().includes(query) ||
      credential.credentialType.toLowerCase().includes(query) ||
      status.detail.toLowerCase().includes(query)
    const matchesAuth = authFilter.value === 'all' || authKind === authFilter.value
    const matchesStatus =
      statusFilter.value === 'all' ||
      (statusFilter.value === 'validated' && status.label === 'Validated') ||
      (statusFilter.value === 'validated' && status.label === 'Connected') ||
      (statusFilter.value === 'attention' && ['Needs Fix', 'Test Error', 'Retest Required'].includes(status.label)) ||
      (statusFilter.value === 'untested' && status.label === 'Saved')

    return matchesQuery && matchesAuth && matchesStatus
  })
})

const totalCredentialCount = computed(() => credentials.value.length)
const validatedCredentialCount = computed(
  () =>
    credentials.value.filter((credential) => {
      const status = credentialStatusPresentation(
        credential,
        credentialTypeMap.value.get(credential.credentialType) || null,
      ).label
      return status === 'Validated' || status === 'Connected'
    }).length,
)
const attentionCredentialCount = computed(
  () =>
    credentials.value.filter((credential) =>
      ['Needs Fix', 'Test Error', 'Retest Required'].includes(
        credentialStatusPresentation(
          credential,
          credentialTypeMap.value.get(credential.credentialType) || null,
        ).label,
      ),
    ).length,
)
const totalUsageCount = computed(() =>
  credentials.value.reduce((sum, credential) => sum + credential.usageCount, 0),
)

function oauthRedirectUri(): string {
  return `${window.location.origin}/rest/oauth2-credential/callback`
}

function cloneDefaultValue<T>(value: T): T {
  if (value === undefined || value === null) return value
  return JSON.parse(JSON.stringify(value)) as T
}

function typeLabel(typeName: string): string {
  return credentialTypeMap.value.get(typeName)?.displayName || typeName
}

function authKindLabel(type: CredentialTypeContract | null): string {
  switch (credentialAuthKind(type)) {
    case 'oauth':
      return 'OAuth2'
    case 'token':
      return 'Token'
    case 'database':
      return 'Database'
    default:
      return 'Custom'
  }
}

function resetModalState() {
  modalError.value = null
  modalSuccess.value = null
  draftValidation.value = null
}

function buildDraftForType(type: CredentialTypeContract, mode: 'create' | 'edit' | 'rotate') {
  const draft: Record<string, unknown> = {}

  if (mode === 'create') {
    for (const property of type.properties || []) {
      if (property.default !== undefined) {
        draft[property.name] = cloneDefaultValue(property.default)
      }
    }
  }

  if (credentialSupportsOAuthConnect(type) && draft.redirectUri === undefined) {
    draft.redirectUri = oauthRedirectUri()
  }

  return draft
}

function compactCredentialData(raw: Record<string, unknown>) {
  const compacted: Record<string, unknown> = {}

  Object.entries(raw || {}).forEach(([key, value]) => {
    if (value === undefined) return
    if (typeof value === 'string' && value.trim().length === 0) return
    compacted[key] = value
  })

  return compacted
}

function rowActionState(id: string, action: string) {
  return rowLoading.value[id] === action
}

function openCreateModal(typeName?: string) {
  modalMode.value = 'create'
  editingCredentialId.value = null
  selectedTypeName.value = typeName || ''
  draftCredentialName.value = ''
  draftCredentialData.value = selectedTypeName.value && selectedType.value
    ? buildDraftForType(selectedType.value, 'create')
    : {}
  resetModalState()
  isModalOpen.value = true
}

function openEditModal(credential: CredentialSummary, mode: 'edit' | 'rotate' = 'edit') {
  const matchedType = credentialTypeMap.value.get(credential.credentialType)
  modalMode.value = mode
  editingCredentialId.value = credential.id
  selectedTypeName.value = credential.credentialType
  draftCredentialName.value = credential.name
  draftCredentialData.value = matchedType ? buildDraftForType(matchedType, mode) : {}
  resetModalState()
  isModalOpen.value = true
}

function closeModal() {
  isModalOpen.value = false
  editingCredentialId.value = null
  selectedTypeName.value = ''
  draftCredentialName.value = ''
  draftCredentialData.value = {}
  resetModalState()
}

function chooseCredentialType(typeName: string) {
  selectedTypeName.value = typeName
  const matchedType = credentialTypeMap.value.get(typeName)
  draftCredentialData.value = matchedType ? buildDraftForType(matchedType, modalMode.value) : {}
  draftValidation.value = null
  modalError.value = null
  modalSuccess.value = null
}

async function fetchCredentials() {
  credentialsLoading.value = true
  pageError.value = null

  try {
    const response = await listCredentials()
    credentials.value = response.data
  } catch (error: any) {
    pageError.value = error?.response?.data || error?.message || 'Failed to load credentials.'
  } finally {
    credentialsLoading.value = false
  }
}

async function fetchCredentialTypes() {
  credentialTypesLoading.value = true

  try {
    const response = await listCredentialTypes()
    credentialTypes.value = response.data
  } catch (error: any) {
    pageError.value = error?.response?.data || error?.message || 'Failed to load credential types.'
  } finally {
    credentialTypesLoading.value = false
  }
}

async function testDraftCredential() {
  if (!selectedType.value) return

  testLoading.value = true
  modalError.value = null
  modalSuccess.value = null

  try {
    const response = await testCredentialType({
      credentialType: selectedType.value.name,
      data: compactCredentialData(draftCredentialData.value),
    })

    draftValidation.value = response.data
    if (response.data.valid) {
      modalSuccess.value = response.data.message
    } else {
      modalError.value = response.data.message
    }
  } catch (error: any) {
    modalError.value = error?.response?.data || error?.message || 'Credential validation failed.'
    draftValidation.value = {
      valid: false,
      status: 'error',
      message: modalError.value || 'Credential validation failed.',
    }
  } finally {
    testLoading.value = false
  }
}

function setRowLoading(id: string, action: string | null) {
  rowLoading.value = {
    ...rowLoading.value,
    [id]: action || '',
  }
}

async function openOAuthPopup(connectUrl: string) {
  return new Promise<{ success: boolean; credentialId?: string; message: string }>((resolve, reject) => {
    const popup = window.open(connectUrl, 'barqflow-oauth-connect', 'width=640,height=760,noopener=no,noreferrer=no')
    if (!popup) {
      reject(new Error('Popup blocked. Allow popups and try again.'))
      return
    }

    let settled = false
    let closeTimer = 0

    const cleanup = () => {
      window.removeEventListener('message', onMessage)
      if (closeTimer) {
        window.clearInterval(closeTimer)
      }
    }

    const onMessage = (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return
      if (!event.data || event.data.source !== 'barqflow-oauth2') return

      settled = true
      cleanup()
      resolve({
        success: !!event.data.success,
        credentialId: event.data.credentialId || undefined,
        message: String(event.data.message || ''),
      })
    }

    window.addEventListener('message', onMessage)
    closeTimer = window.setInterval(() => {
      if (!popup || popup.closed) {
        cleanup()
        if (!settled) {
          reject(new Error('OAuth popup closed before the connection completed.'))
        }
      }
    }, 400)
  })
}

async function connectCredential(credential: CredentialSummary) {
  const credentialType = credentialTypeMap.value.get(credential.credentialType) || null
  if (!credentialSupportsOAuthConnect(credentialType)) {
    pageNotice.value = `${credential.name} uses a manual credential flow. Save the values directly instead.`
    return
  }

  setRowLoading(credential.id, 'connect')
  pageError.value = null
  pageNotice.value = null

  try {
    const response = await startCredentialOAuthConnect(credential.id)
    const result = await openOAuthPopup(response.data.connectUrl)
    await fetchCredentials()

    if (!result.success) {
      throw new Error(result.message || 'OAuth connection failed.')
    }

    pageNotice.value = result.message || `${credential.name} connected successfully.`
  } catch (error: any) {
    pageError.value = error?.message || 'OAuth connection failed.'
  } finally {
    setRowLoading(credential.id, null)
  }
}

async function saveCredential(action: 'save' | 'connect') {
  if (!selectedType.value || !draftCredentialName.value.trim()) {
    modalError.value = 'Credential name is required.'
    return
  }

  saveLoading.value = true
  modalError.value = null
  modalSuccess.value = null

  try {
    const payload = {
      name: draftCredentialName.value.trim(),
      data: compactCredentialData(draftCredentialData.value),
    }

    let saved: CredentialSummary

    if (modalMode.value === 'edit') {
      if (!editingCredentialId.value) throw new Error('Missing credential id for update.')
      const response = await updateCredential(editingCredentialId.value, payload)
      saved = response.data
    } else if (modalMode.value === 'rotate') {
      if (!editingCredentialId.value) throw new Error('Missing credential id for rotation.')
      const response = await rotateCredential(editingCredentialId.value, payload)
      saved = response.data
    } else {
      const response = await createCredential({
        name: payload.name,
        credentialType: selectedType.value.name,
        data: payload.data,
      })
      saved = response.data
    }

    await fetchCredentials()
    const successMessage =
      modalMode.value === 'rotate'
        ? 'Credential rotated. Re-test or reconnect it before production use.'
        : modalMode.value === 'edit'
          ? 'Credential updated.'
          : 'Credential saved.'
    modalSuccess.value = successMessage

    const shouldConnect = action === 'connect' && credentialSupportsOAuthConnect(selectedType.value)
    closeModal()

    if (shouldConnect) {
      await connectCredential(saved)
      return
    }

    pageNotice.value = successMessage
  } catch (error: any) {
    modalError.value = error?.response?.data || error?.message || 'Failed to save credential.'
  } finally {
    saveLoading.value = false
  }
}

async function testSavedCredential(credential: CredentialSummary) {
  setRowLoading(credential.id, 'test')
  pageError.value = null
  pageNotice.value = null

  try {
    const response = await testSavedCredentialById(credential.id)
    await fetchCredentials()
    pageNotice.value = response.data.message
  } catch (error: any) {
    pageError.value = error?.response?.data || error?.message || 'Credential validation failed.'
  } finally {
    setRowLoading(credential.id, null)
  }
}

async function deleteCredential(credential: CredentialSummary) {
  const confirmed = window.confirm(`Delete credential \"${credential.name}\"?`)
  if (!confirmed) return

  setRowLoading(credential.id, 'delete')
  pageError.value = null
  pageNotice.value = null

  try {
    await deleteCredentialById(credential.id)
    credentials.value = credentials.value.filter((entry) => entry.id !== credential.id)
    pageNotice.value = `${credential.name} deleted.`
  } catch (error: any) {
    pageError.value = error?.response?.data || error?.message || 'Failed to delete credential.'
  } finally {
    setRowLoading(credential.id, null)
  }
}

function updateDraftField(property: NodeProperty, value: unknown) {
  draftCredentialData.value = {
    ...draftCredentialData.value,
    [property.name]: value,
  }
  draftValidation.value = null
  modalSuccess.value = null
}

function credentialCardTitle(credential: CredentialSummary) {
  const type = credentialTypeMap.value.get(credential.credentialType) || null
  return `${credential.name}`
}

function returnToWorkflow() {
  if (!requestedReturnTo.value) return
  router.push(requestedReturnTo.value)
}

watch(
  () => route.fullPath,
  () => {
    const intentKey = `${requestedCredentialType.value}:${requestedReturnTo.value || ''}:${requestedNodeName.value || ''}`
    if (!requestedCredentialType.value || credentialTypes.value.length === 0) return
    if (routedIntentHandled.value === intentKey) return

    routedIntentHandled.value = intentKey
    openCreateModal(requestedCredentialType.value)
    pageNotice.value = requestedNodeName.value
      ? `Create a ${typeLabel(requestedCredentialType.value)} credential for ${requestedNodeName.value}, then return to keep binding the node.`
      : `Create a ${typeLabel(requestedCredentialType.value)} credential, then return to continue.`
  },
)

onMounted(async () => {
  await Promise.all([fetchCredentials(), fetchCredentialTypes()])
})
</script>

<template>
  <div class="min-h-full bg-slate-100/80 p-6 text-slate-900 md:p-10">
    <div class="mx-auto max-w-7xl space-y-6">
      <section class="rounded-3xl border border-slate-200 bg-white px-6 py-6 shadow-sm md:px-8">
        <div class="flex flex-col gap-5 lg:flex-row lg:items-center lg:justify-between">
          <div class="max-w-3xl">
            <div class="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500">
              <Shield class="h-3.5 w-3.5" />
              Credential Management
            </div>
            <h1 class="mt-3 text-3xl font-black tracking-tight text-slate-950 md:text-4xl">
              Credential Inventory and Access
            </h1>
            <p class="mt-2 text-sm leading-6 text-slate-600 md:text-base">
              Manage tokens, database connections, and OAuth credentials with clear validation state,
              usage telemetry, and reusable workflow bindings.
            </p>
          </div>

          <div class="flex flex-wrap gap-3">
            <button
              type="button"
              class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
              @click="fetchCredentials"
            >
              <RefreshCcw class="h-4 w-4" />
              Refresh
            </button>
            <button
              type="button"
              class="inline-flex items-center gap-2 rounded-xl bg-slate-950 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-slate-800"
              @click="openCreateModal()"
            >
              <Plus class="h-4 w-4" />
              New Credential
            </button>
          </div>
        </div>
      </section>

      <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Total</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ totalCredentialCount }}</p>
          <p class="mt-1 text-sm text-slate-500">Credentials stored in the workspace.</p>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Validated</p>
          <p class="mt-3 text-3xl font-black text-emerald-600">{{ validatedCredentialCount }}</p>
          <p class="mt-1 text-sm text-slate-500">Credentials ready for production use.</p>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Attention</p>
          <p class="mt-3 text-3xl font-black text-amber-600">{{ attentionCredentialCount }}</p>
          <p class="mt-1 text-sm text-slate-500">Credentials that should be re-tested or fixed.</p>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Usage Events</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ totalUsageCount }}</p>
          <p class="mt-1 text-sm text-slate-500">Runtime resolutions recorded across executions.</p>
        </div>
      </section>

      <section class="grid gap-6 xl:grid-cols-[320px_minmax(0,1fr)]">
        <aside class="space-y-6">
          <div
            v-if="requestedReturnTo"
            class="rounded-2xl border border-brand-200 bg-brand-50 px-5 py-4 text-sm text-brand-900 shadow-sm"
          >
            <p class="font-semibold">Workflow setup handoff</p>
            <p class="mt-2 leading-6 text-brand-900/80">
              {{ pageNotice || 'Create the credential, then return to finish binding it in the editor.' }}
            </p>
            <button
              type="button"
              class="mt-4 inline-flex items-center gap-2 rounded-xl bg-white px-4 py-2.5 font-semibold text-brand-900 transition hover:bg-brand-100"
              @click="returnToWorkflow"
            >
              Back To Workflow
              <ArrowRight class="h-4 w-4" />
            </button>
          </div>

          <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
            <div class="mb-4">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Filters</p>
              <h2 class="mt-2 text-lg font-black text-slate-950">Search and segment credentials</h2>
            </div>

            <div class="space-y-4">
              <label class="flex items-center gap-3 rounded-xl border border-slate-200 bg-slate-50 px-4 py-3">
                <Search class="h-4 w-4 text-slate-400" />
                <input
                  v-model="searchTerm"
                  type="text"
                  placeholder="Search credentials or validation notes"
                  class="w-full bg-transparent text-sm text-slate-900 outline-none placeholder:text-slate-400"
                />
              </label>

              <div class="grid gap-4">
                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Auth Mode</span>
                  <select
                    v-model="authFilter"
                    class="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-700 outline-none transition focus:border-brand-500"
                  >
                    <option value="all">All auth modes</option>
                    <option value="oauth">OAuth2</option>
                    <option value="token">Token</option>
                    <option value="database">Database</option>
                    <option value="custom">Custom</option>
                  </select>
                </label>

                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Status</span>
                  <select
                    v-model="statusFilter"
                    class="w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-700 outline-none transition focus:border-brand-500"
                  >
                    <option value="all">All statuses</option>
                    <option value="validated">Validated / Connected</option>
                    <option value="attention">Needs attention</option>
                    <option value="untested">Untested</option>
                  </select>
                </label>
              </div>
            </div>
          </div>

          <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
            <div class="mb-4">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Quick Start</p>
              <h2 class="mt-2 text-lg font-black text-slate-950">Common integrations</h2>
            </div>

            <div class="space-y-3">
              <button
                v-for="quickStart in quickStarts"
                :key="quickStart.credentialType"
                type="button"
                class="w-full rounded-xl border border-slate-200 bg-slate-50 px-4 py-3 text-left transition hover:border-slate-300 hover:bg-white"
                @click="openCreateModal(quickStart.credentialType)"
              >
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <p class="text-sm font-semibold text-slate-900">{{ typeLabel(quickStart.credentialType) }}</p>
                    <p class="mt-1 text-xs leading-5 text-slate-500">{{ quickStart.summary }}</p>
                  </div>
                  <span class="rounded-full bg-white px-2 py-1 text-[10px] font-bold uppercase tracking-[0.18em] text-slate-500">
                    {{ authKindLabel(quickStart.type) }}
                  </span>
                </div>
              </button>
            </div>
          </div>
        </aside>

        <section class="space-y-4">
          <div
            v-if="pageError"
            class="flex items-start gap-3 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 shadow-sm"
          >
            <XCircle class="mt-0.5 h-4 w-4 shrink-0" />
            <span>{{ pageError }}</span>
          </div>
          <div
            v-else-if="pageNotice && !requestedReturnTo"
            class="flex items-start gap-3 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 shadow-sm"
          >
            <CheckCircle2 class="mt-0.5 h-4 w-4 shrink-0" />
            <span>{{ pageNotice }}</span>
          </div>

          <div class="rounded-2xl border border-slate-200 bg-white shadow-sm">
            <div class="flex flex-col gap-3 border-b border-slate-200 px-5 py-4 md:flex-row md:items-center md:justify-between">
              <div>
                <h2 class="text-lg font-black text-slate-950">Saved Credentials</h2>
                <p class="mt-1 text-sm text-slate-500">
                  {{ filteredCredentials.length }} of {{ credentials.length }} credential{{ credentials.length === 1 ? '' : 's' }} visible.
                </p>
              </div>
            </div>

            <div v-if="credentialsLoading" class="flex items-center gap-3 px-5 py-6 text-sm text-slate-500">
              <Loader2 class="h-4 w-4 animate-spin" />
              Loading credentials...
            </div>

            <div
              v-else-if="filteredCredentials.length === 0"
              class="px-6 py-16 text-center"
            >
              <KeyRound class="mx-auto h-8 w-8 text-slate-300" />
              <h3 class="mt-4 text-lg font-bold text-slate-900">No credentials match this view.</h3>
              <p class="mt-2 text-sm text-slate-500">
                Adjust the filters or create a new credential to start binding integrations.
              </p>
            </div>

            <div v-else class="divide-y divide-slate-200">
              <article
                v-for="credential in filteredCredentials"
                :key="credential.id"
                class="px-5 py-5"
              >
                <div class="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
                  <div class="min-w-0 space-y-3">
                    <div class="flex flex-wrap items-center gap-2">
                      <h3 class="text-lg font-black text-slate-950">{{ credentialCardTitle(credential) }}</h3>
                      <span
                        class="rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em]"
                        :class="credentialStatusPresentation(credential, credentialTypeMap.get(credential.credentialType) || null).badgeClass"
                      >
                        {{ credentialStatusPresentation(credential, credentialTypeMap.get(credential.credentialType) || null).label }}
                      </span>
                      <span class="rounded-full bg-slate-100 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] text-slate-500">
                        {{ authKindLabel(credentialTypeMap.get(credential.credentialType) || null) }}
                      </span>
                    </div>
                    <div>
                      <p class="text-sm font-semibold text-slate-700">{{ typeLabel(credential.credentialType) }}</p>
                      <p class="mt-1 max-w-2xl text-sm text-slate-500">
                        {{ credentialStatusPresentation(credential, credentialTypeMap.get(credential.credentialType) || null).detail }}
                      </p>
                    </div>

                    <dl class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
                      <div class="rounded-xl bg-slate-50 px-4 py-3">
                        <dt class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Last Tested</dt>
                        <dd class="mt-2 text-sm font-semibold text-slate-900">
                          {{ formatRelativeTime(credential.lastTestedAt, 'Not tested') }}
                        </dd>
                        <p class="mt-1 text-xs text-slate-500">{{ formatDateTime(credential.lastTestedAt, 'Not tested') }}</p>
                      </div>
                      <div class="rounded-xl bg-slate-50 px-4 py-3">
                        <dt class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Last Used</dt>
                        <dd class="mt-2 text-sm font-semibold text-slate-900">
                          {{ formatRelativeTime(credential.lastUsedAt, 'Never used') }}
                        </dd>
                        <p class="mt-1 text-xs text-slate-500">{{ formatDateTime(credential.lastUsedAt, 'Never used') }}</p>
                      </div>
                      <div class="rounded-xl bg-slate-50 px-4 py-3">
                        <dt class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Usage Count</dt>
                        <dd class="mt-2 text-sm font-semibold text-slate-900">{{ credential.usageCount }}</dd>
                        <p class="mt-1 text-xs text-slate-500">Incremented when the runtime resolves the credential.</p>
                      </div>
                      <div class="rounded-xl bg-slate-50 px-4 py-3">
                        <dt class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Rotated</dt>
                        <dd class="mt-2 text-sm font-semibold text-slate-900">
                          {{ formatRelativeTime(credential.rotatedAt, 'Not rotated') }}
                        </dd>
                        <p class="mt-1 text-xs text-slate-500">{{ formatDateTime(credential.rotatedAt, 'Not rotated') }}</p>
                      </div>
                    </dl>
                  </div>

                  <div class="flex flex-wrap gap-2 xl:max-w-[280px] xl:justify-end">
                    <button
                      type="button"
                      class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
                      :disabled="rowActionState(credential.id, 'test')"
                      @click="testSavedCredential(credential)"
                    >
                      <Loader2 v-if="rowActionState(credential.id, 'test')" class="h-4 w-4 animate-spin" />
                      <FlaskConical v-else class="h-4 w-4" />
                      Test
                    </button>
                    <button
                      v-if="credentialSupportsOAuthConnect(credentialTypeMap.get(credential.credentialType) || null)"
                      type="button"
                      class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
                      :disabled="rowActionState(credential.id, 'connect')"
                      @click="connectCredential(credential)"
                    >
                      <Loader2 v-if="rowActionState(credential.id, 'connect')" class="h-4 w-4 animate-spin" />
                      <PlugZap v-else class="h-4 w-4" />
                      Connect
                    </button>
                    <button
                      type="button"
                      class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
                      @click="openEditModal(credential, 'edit')"
                    >
                      <Pencil class="h-4 w-4" />
                      Edit
                    </button>
                    <button
                      type="button"
                      class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
                      @click="openEditModal(credential, 'rotate')"
                    >
                      <Shuffle class="h-4 w-4" />
                      Rotate
                    </button>
                    <button
                      type="button"
                      class="inline-flex items-center gap-2 rounded-xl border border-red-200 bg-white px-3 py-2 text-sm font-semibold text-red-600 transition hover:border-red-300 hover:bg-red-50"
                      :disabled="rowActionState(credential.id, 'delete')"
                      @click="deleteCredential(credential)"
                    >
                      <Loader2 v-if="rowActionState(credential.id, 'delete')" class="h-4 w-4 animate-spin" />
                      <Trash2 v-else class="h-4 w-4" />
                      Delete
                    </button>
                  </div>
                </div>
              </article>
            </div>
          </div>
        </section>
      </section>
    </div>

    <div v-if="isModalOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 p-4 backdrop-blur-sm">
      <div class="max-h-[92vh] w-full max-w-4xl overflow-hidden rounded-[28px] border border-slate-200 bg-white shadow-2xl shadow-slate-950/20">
        <div class="flex items-start justify-between border-b border-slate-100 px-7 py-6">
          <div class="space-y-2">
            <div class="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">
              <Shield class="h-3.5 w-3.5" />
              {{ isRotateMode ? 'Rotate Credential' : isEditMode ? 'Edit Credential' : 'Create Credential' }}
            </div>
            <div>
              <h2 class="text-2xl font-black text-slate-950">
                {{ selectedType ? selectedType.displayName : 'Choose a credential type' }}
              </h2>
              <p class="mt-1 text-sm text-slate-500">
                <span v-if="isCreateMode">Create a reusable credential and bind it across workflows.</span>
                <span v-else-if="isRotateMode">Leave any field blank to keep the stored value. Rotation clears the last validation state.</span>
                <span v-else>Leave any field blank to preserve the stored value, then re-test after changes.</span>
              </p>
            </div>
          </div>

          <button
            type="button"
            class="rounded-full p-2 text-slate-400 transition hover:bg-slate-100 hover:text-slate-700"
            @click="closeModal"
          >
            <X class="h-5 w-5" />
          </button>
        </div>

        <div class="grid max-h-[calc(92vh-164px)] gap-0 overflow-auto lg:grid-cols-[280px_minmax(0,1fr)]">
          <aside class="border-b border-slate-100 bg-slate-50 p-5 lg:border-b-0 lg:border-r">
            <div class="mb-4 flex items-center justify-between">
              <h3 class="text-sm font-black uppercase tracking-[0.18em] text-slate-500">Credential Types</h3>
              <Loader2 v-if="credentialTypesLoading" class="h-4 w-4 animate-spin text-slate-400" />
            </div>
            <div class="space-y-2">
              <button
                v-for="type in credentialTypes"
                :key="type.name"
                type="button"
                class="w-full rounded-2xl border px-4 py-3 text-left transition"
                :class="selectedTypeName === type.name
                  ? 'border-slate-950 bg-slate-950 text-white shadow-lg shadow-slate-950/10'
                  : 'border-slate-200 bg-white text-slate-700 hover:border-slate-300 hover:bg-slate-50'"
                @click="chooseCredentialType(type.name)"
              >
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <p class="text-sm font-semibold">{{ type.displayName }}</p>
                    <p class="mt-1 text-xs" :class="selectedTypeName === type.name ? 'text-slate-300' : 'text-slate-500'">
                      {{ authKindLabel(type) }}
                    </p>
                  </div>
                  <span
                    class="rounded-full px-2 py-1 text-[10px] font-bold uppercase tracking-[0.18em]"
                    :class="selectedTypeName === type.name ? 'bg-white/10 text-white' : 'bg-slate-100 text-slate-500'"
                  >
                    {{ credentialSupportsOAuthConnect(type) ? 'Connect' : 'Manual' }}
                  </span>
                </div>
              </button>
            </div>
          </aside>

          <section class="space-y-6 p-7">
            <div v-if="!selectedType" class="rounded-[24px] border border-dashed border-slate-300 bg-slate-50 px-6 py-16 text-center text-sm text-slate-500">
              Choose a credential type to configure the required fields.
            </div>
            <template v-else>
              <div class="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
                <div>
                  <label class="mb-2 block text-sm font-bold text-slate-700">Credential Name</label>
                  <input
                    v-model="draftCredentialName"
                    type="text"
                    placeholder="e.g. Production OpenAI"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-medium text-slate-900 outline-none transition focus:border-brand-500"
                  />
                </div>

                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                  <p class="font-semibold text-slate-800">Auth Mode</p>
                  <p class="mt-1">{{ selectedTypeAuthKind === 'oauth' ? 'OAuth2 connect flow' : selectedTypeAuthKind === 'database' ? 'Database connection' : selectedTypeAuthKind === 'token' ? 'Token / API secret' : 'Custom credential' }}</p>
                </div>
              </div>

              <div v-if="selectedType.notice" class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                {{ selectedType.notice }}
              </div>

              <div v-if="canConnectSelectedType" class="rounded-2xl border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-800">
                <p class="font-semibold">OAuth redirect URI</p>
                <p class="mt-1 break-all font-mono text-xs">{{ oauthRedirectUri() }}</p>
                <p class="mt-2 text-blue-700/80">
                  Save the credential, then use <span class="font-semibold">Save and Connect</span> to complete the provider authorization flow.
                </p>
              </div>

              <div class="grid gap-4 md:grid-cols-2">
                <div
                  v-for="property in selectedType.properties"
                  :key="property.name"
                  class="rounded-2xl border border-slate-200 bg-white p-4"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div>
                      <label class="text-sm font-bold text-slate-800">{{ property.displayName }}</label>
                      <p v-if="property.required" class="mt-1 text-[11px] font-bold uppercase tracking-[0.16em] text-red-500">
                        Required
                      </p>
                    </div>
                    <span
                      v-if="isSecretCredentialField(property)"
                      class="rounded-full bg-slate-100 px-2 py-1 text-[10px] font-bold uppercase tracking-[0.18em] text-slate-500"
                    >
                      Secret
                    </span>
                  </div>

                  <input
                    v-if="property.type === 'string'"
                    :value="String(draftCredentialData[property.name] ?? '')"
                    :type="isSecretCredentialField(property) ? 'password' : 'text'"
                    class="mt-3 w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm text-slate-900 outline-none transition focus:border-brand-500"
                    :placeholder="!isCreateMode ? 'Leave blank to keep stored value' : ''"
                    @input="updateDraftField(property, ($event.target as HTMLInputElement).value)"
                  />

                  <textarea
                    v-else-if="property.type === 'text'"
                    :value="String(draftCredentialData[property.name] ?? '')"
                    rows="4"
                    class="mt-3 w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm text-slate-900 outline-none transition focus:border-brand-500"
                    :placeholder="!isCreateMode ? 'Leave blank to keep stored value' : ''"
                    @input="updateDraftField(property, ($event.target as HTMLTextAreaElement).value)"
                  />

                  <input
                    v-else-if="property.type === 'number'"
                    :value="String(draftCredentialData[property.name] ?? '')"
                    type="number"
                    class="mt-3 w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm text-slate-900 outline-none transition focus:border-brand-500"
                    :placeholder="!isCreateMode ? 'Leave blank to keep stored value' : ''"
                    @input="updateDraftField(property, ($event.target as HTMLInputElement).value === '' ? '' : Number(($event.target as HTMLInputElement).value))"
                  />

                  <label v-else-if="property.type === 'boolean'" class="mt-3 flex items-center gap-3 rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm text-slate-700">
                    <input
                      :checked="Boolean(draftCredentialData[property.name])"
                      type="checkbox"
                      class="h-4 w-4 rounded border-slate-300"
                      @change="updateDraftField(property, ($event.target as HTMLInputElement).checked)"
                    />
                    Enable
                  </label>

                  <select
                    v-else-if="property.type === 'options'"
                    :value="String(draftCredentialData[property.name] ?? '')"
                    class="mt-3 w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm font-medium text-slate-900 outline-none transition focus:border-brand-500"
                    @change="updateDraftField(property, ($event.target as HTMLSelectElement).value)"
                  >
                    <option value="">Select an option</option>
                    <option
                      v-for="option in property.options || []"
                      :key="String(option.value)"
                      :value="String(option.value)"
                    >
                      {{ option.name }}
                    </option>
                  </select>

                  <textarea
                    v-else
                    :value="String(draftCredentialData[property.name] ?? '')"
                    rows="3"
                    class="mt-3 w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2.5 text-sm text-slate-900 outline-none transition focus:border-brand-500"
                    :placeholder="!isCreateMode ? 'Leave blank to keep stored value' : ''"
                    @input="updateDraftField(property, ($event.target as HTMLTextAreaElement).value)"
                  />

                  <p v-if="property.description" class="mt-3 text-xs leading-5 text-slate-500">{{ property.description }}</p>
                </div>
              </div>

              <div v-if="selectedType.documentationUrl" class="flex items-center justify-between rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                <div>
                  <p class="font-semibold text-slate-800">Provider documentation</p>
                  <p class="mt-1">Open the provider instructions in a new tab while configuring the credential.</p>
                </div>
                <a
                  :href="selectedType.documentationUrl"
                  target="_blank"
                  rel="noreferrer"
                  class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
                >
                  Open Docs
                  <ExternalLink class="h-4 w-4" />
                </a>
              </div>

              <div v-if="modalError" class="flex items-start gap-3 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                <XCircle class="mt-0.5 h-4 w-4 shrink-0" />
                <span>{{ modalError }}</span>
              </div>
              <div v-else-if="modalSuccess" class="flex items-start gap-3 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                <CheckCircle2 class="mt-0.5 h-4 w-4 shrink-0" />
                <span>{{ modalSuccess }}</span>
              </div>
            </template>
          </section>
        </div>

        <div class="flex flex-col gap-3 border-t border-slate-100 bg-slate-50 px-7 py-5 md:flex-row md:items-center md:justify-between">
          <div class="text-sm text-slate-500">
            <span v-if="draftValidation">Last draft test: {{ draftValidation.message }}</span>
            <span v-else-if="canConnectSelectedType">OAuth credentials can be saved first, then connected in a popup flow.</span>
            <span v-else>Test saved credentials after edit or rotation so the validation state stays current.</span>
          </div>

          <div class="flex flex-wrap justify-end gap-3">
            <button
              type="button"
              class="rounded-2xl px-4 py-3 text-sm font-semibold text-slate-500 transition hover:bg-white hover:text-slate-700"
              @click="closeModal"
            >
              Cancel
            </button>
            <button
              v-if="selectedType && isCreateMode"
              type="button"
              class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="testLoading"
              @click="testDraftCredential"
            >
              <Loader2 v-if="testLoading" class="h-4 w-4 animate-spin" />
              <FlaskConical v-else class="h-4 w-4" />
              Test Draft
            </button>
            <button
              v-if="selectedType"
              type="button"
              class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="saveLoading || !draftCredentialName.trim()"
              @click="saveCredential('save')"
            >
              <Loader2 v-if="saveLoading" class="h-4 w-4 animate-spin" />
              <Pencil v-else-if="isEditMode" class="h-4 w-4" />
              <Shuffle v-else-if="isRotateMode" class="h-4 w-4" />
              <Plus v-else class="h-4 w-4" />
              {{ isRotateMode ? 'Rotate Credential' : isEditMode ? 'Update Credential' : 'Save Credential' }}
            </button>
            <button
              v-if="selectedType && canConnectSelectedType"
              type="button"
              class="inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-slate-950/10 transition hover:-translate-y-0.5 hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="saveLoading || !draftCredentialName.trim()"
              @click="saveCredential('connect')"
            >
              <Loader2 v-if="saveLoading" class="h-4 w-4 animate-spin" />
              <PlugZap v-else class="h-4 w-4" />
              {{ isEditMode || isRotateMode ? 'Save And Connect' : 'Save And Connect' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
