<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  CheckCircle2,
  FileClock,
  FolderGit2,
  KeyRound,
  Loader2,
  RefreshCw,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Waypoints,
} from 'lucide-vue-next'
import { useAuthStore } from '../stores/auth'
import {
  approvePromotionRequest,
  createPromotionRequest,
  createPromotionTarget,
  createSecretProvider,
  getWorkspacePolicy,
  listAuditLogs,
  listPromotionRequests,
  listPromotionTargets,
  listSecretProviders,
  updateWorkspacePolicy,
  validateSecretProvider,
} from '../features/governance/api'
import { listWorkflows } from '../features/workflows/api'
import type {
  AuditLogRecord,
  PromotionRequestRecord,
  PromotionTargetRecord,
  SecretProviderRecord,
  WorkflowRecord,
  WorkspacePolicyRecord,
} from '../types/contracts'

const authStore = useAuthStore()

const loading = ref(false)
const actionLoading = ref(false)
const validatingProviderId = ref<string | null>(null)
const error = ref<string | null>(null)
const success = ref<string | null>(null)

const policy = ref<WorkspacePolicyRecord | null>(null)
const providers = ref<SecretProviderRecord[]>([])
const targets = ref<PromotionTargetRecord[]>([])
const promotionRequests = ref<PromotionRequestRecord[]>([])
const auditLogs = ref<AuditLogRecord[]>([])
const workflows = ref<WorkflowRecord[]>([])

const policyForm = ref({
  blockedNodeTypes: '',
  blockedSupportTiers: '',
  approvalRequiredNodeTypes: '',
  maxWorkflowNodes: '',
})

const providerForm = ref({
  name: '',
  providerType: 'env',
  envPrefix: '',
  vaultAddress: '',
  vaultMountPath: 'secret',
  vaultTokenEnvVar: '',
})

const targetForm = ref({
  name: '',
  environment: 'staging',
  gitRepoUrl: '',
  gitBranch: 'main',
  requiresApproval: true,
})

const requestForm = ref({
  workflowId: '',
  targetId: '',
  sourceControlRef: '',
  notes: '',
})

const approvalNotes = ref<Record<string, string>>({})

const activeRole = computed(() => authStore.user?.workspaceRole || 'viewer')
const canOperate = computed(() => ['owner', 'admin', 'member'].includes(activeRole.value))
const canAdminister = computed(() => ['owner', 'admin'].includes(activeRole.value))
const providerDraftHint = computed(() => {
  if (providerForm.value.providerType === 'vault') {
    return 'HashiCorp Vault provider. BarqFlow will read KV v2 values using the configured address, mount path, and token env var.'
  }

  return 'Environment provider. BarqFlow resolves secret refs from environment variables using an optional uppercase prefix.'
})

const pendingApprovals = computed(() =>
  promotionRequests.value.filter((request) => request.status.toLowerCase() === 'pendingapproval'),
)
const providerAlerts = computed(() => providers.value.filter((provider) => provider.status !== 'validated'))
const approvalGates = computed(() => policy.value?.approvalRequiredNodeTypes.length || 0)
const blockedNodesCount = computed(() => policy.value?.blockedNodeTypes.length || 0)
const supportedTierBlocks = computed(() => policy.value?.blockedSupportTiers.length || 0)
const activeTargets = computed(() => targets.value.length)

function clearFeedback() {
  error.value = null
  success.value = null
}

function setSuccess(message: string) {
  success.value = message
  error.value = null
}

function parseLines(value: string) {
  return Array.from(
    new Set(
      value
        .split(/[\n,]/)
        .map((entry) => entry.trim())
        .filter(Boolean),
    ),
  )
}

function joinLines(values?: string[] | null) {
  return Array.isArray(values) ? values.join('\n') : ''
}

function buildProviderConfig() {
  if (providerForm.value.providerType === 'vault') {
    return {
      address: providerForm.value.vaultAddress.trim(),
      mountPath: providerForm.value.vaultMountPath.trim() || 'secret',
      tokenEnvVar: providerForm.value.vaultTokenEnvVar.trim(),
    }
  }

  return {
    prefix: providerForm.value.envPrefix.trim(),
  }
}

function resetProviderForm() {
  providerForm.value = {
    name: '',
    providerType: 'env',
    envPrefix: '',
    vaultAddress: '',
    vaultMountPath: 'secret',
    vaultTokenEnvVar: '',
  }
}

function resetTargetForm() {
  targetForm.value = {
    name: '',
    environment: 'staging',
    gitRepoUrl: '',
    gitBranch: 'main',
    requiresApproval: true,
  }
}

function resetRequestForm() {
  requestForm.value = {
    workflowId: '',
    targetId: '',
    sourceControlRef: '',
    notes: '',
  }
}

function loadPolicyForm(nextPolicy: WorkspacePolicyRecord) {
  policyForm.value = {
    blockedNodeTypes: joinLines(nextPolicy.blockedNodeTypes),
    blockedSupportTiers: joinLines(nextPolicy.blockedSupportTiers),
    approvalRequiredNodeTypes: joinLines(nextPolicy.approvalRequiredNodeTypes),
    maxWorkflowNodes:
      nextPolicy.maxWorkflowNodes !== undefined && nextPolicy.maxWorkflowNodes !== null
        ? String(nextPolicy.maxWorkflowNodes)
        : '',
  }
}

function statusBadgeClasses(status: string) {
  const normalized = status.toLowerCase()
  if (normalized === 'validated' || normalized === 'approved' || normalized === 'healthy') {
    return 'bg-emerald-50 text-emerald-700 ring-emerald-200'
  }
  if (normalized === 'draft' || normalized === 'pendingapproval' || normalized === 'warning') {
    return 'bg-amber-50 text-amber-700 ring-amber-200'
  }
  if (normalized === 'error' || normalized === 'failed' || normalized === 'needsattention') {
    return 'bg-rose-50 text-rose-700 ring-rose-200'
  }
  return 'bg-slate-100 text-slate-700 ring-slate-200'
}

function formatDateTime(value?: string | null) {
  if (!value) return 'Not available'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

function providerSummary(provider: SecretProviderRecord) {
  if (provider.providerType === 'vault') {
    const address = String(provider.config.address || 'Vault')
    const mountPath = String(provider.config.mountPath || 'secret')
    return `${address} / ${mountPath}`
  }

  const prefix = String(provider.config.prefix || 'No prefix')
  return prefix ? `Prefix: ${prefix}` : 'Uses environment variables directly'
}

function workflowNameForRequest(request: PromotionRequestRecord) {
  const snapshotName = request.workflowSnapshot?.name
  return typeof snapshotName === 'string' && snapshotName.trim() ? snapshotName : request.workflowId
}

function nodeCountForRequest(request: PromotionRequestRecord) {
  const policySnapshot = request.workflowSnapshot?.policy
  const policyNodeCount =
    policySnapshot &&
    typeof policySnapshot === 'object' &&
    typeof (policySnapshot as Record<string, unknown>).nodeCount === 'number'
      ? Number((policySnapshot as Record<string, unknown>).nodeCount)
      : null
  if (policyNodeCount !== null) return policyNodeCount

  const nodes = request.workflowSnapshot?.nodes
  return Array.isArray(nodes) ? nodes.length : 0
}

function approvalReasonsForRequest(request: PromotionRequestRecord) {
  const policySnapshot = request.workflowSnapshot?.policy
  if (!policySnapshot || typeof policySnapshot !== 'object') return []

  const approvalReasons = (policySnapshot as Record<string, unknown>).approvalReasons
  return Array.isArray(approvalReasons)
    ? approvalReasons.map((entry) => String(entry)).filter(Boolean)
    : []
}

function workflowTagsForRequest(request: PromotionRequestRecord) {
  const tags = request.workflowSnapshot?.tags
  return Array.isArray(tags) ? tags.map((entry) => String(entry)).filter(Boolean) : []
}

function targetName(targetId: string) {
  return targets.value.find((target) => target.id === targetId)?.name || targetId
}

async function loadSurface() {
  loading.value = true
  error.value = null

  try {
    const [
      policyResponse,
      providersResponse,
      targetsResponse,
      requestsResponse,
      auditLogsResponse,
      workflowsResponse,
    ] = await Promise.all([
      getWorkspacePolicy(),
      listSecretProviders(),
      listPromotionTargets(),
      listPromotionRequests(50),
      listAuditLogs(80),
      listWorkflows({ limit: 100, sortBy: 'updatedAt', sortDirection: 'desc' }),
    ])

    policy.value = policyResponse.data
    loadPolicyForm(policyResponse.data)
    providers.value = providersResponse.data
    targets.value = targetsResponse.data
    promotionRequests.value = requestsResponse.data
    auditLogs.value = auditLogsResponse.data
    workflows.value = workflowsResponse.data

    if (!requestForm.value.workflowId && workflows.value.length > 0) {
      requestForm.value.workflowId = workflows.value[0].id
    }

    if (!requestForm.value.targetId && targets.value.length > 0) {
      requestForm.value.targetId = targets.value[0].id
    }
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to load governance controls.'
  } finally {
    loading.value = false
  }
}

async function handlePolicySave() {
  if (!canAdminister.value) return

  actionLoading.value = true
  clearFeedback()

  try {
    const response = await updateWorkspacePolicy({
      blockedNodeTypes: parseLines(policyForm.value.blockedNodeTypes),
      blockedSupportTiers: parseLines(policyForm.value.blockedSupportTiers),
      approvalRequiredNodeTypes: parseLines(policyForm.value.approvalRequiredNodeTypes),
      maxWorkflowNodes: policyForm.value.maxWorkflowNodes.trim()
        ? Number(policyForm.value.maxWorkflowNodes.trim())
        : null,
    })
    policy.value = response.data
    loadPolicyForm(response.data)
    setSuccess('Workspace governance policy updated.')
    await loadSurface()
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to update workspace policy.'
  } finally {
    actionLoading.value = false
  }
}

async function handleProviderCreate() {
  if (!canAdminister.value) return

  const name = providerForm.value.name.trim()
  if (!name) {
    error.value = 'Secret provider name is required.'
    return
  }

  const config = buildProviderConfig()
  if (providerForm.value.providerType === 'vault') {
    if (!String(config.address || '').trim() || !String(config.tokenEnvVar || '').trim()) {
      error.value = 'Vault providers require an address and a token environment variable.'
      return
    }
  }

  actionLoading.value = true
  clearFeedback()

  try {
    await createSecretProvider({
      name,
      providerType: providerForm.value.providerType,
      config,
    })
    resetProviderForm()
    await loadSurface()
    setSuccess('Secret provider created and validated.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to create secret provider.'
  } finally {
    actionLoading.value = false
  }
}

async function handleProviderValidate(providerId: string) {
  if (!canAdminister.value) return

  validatingProviderId.value = providerId
  clearFeedback()

  try {
    await validateSecretProvider(providerId)
    await loadSurface()
    setSuccess('Secret provider validation completed.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to validate secret provider.'
  } finally {
    validatingProviderId.value = null
  }
}

async function handleTargetCreate() {
  if (!canAdminister.value) return

  const name = targetForm.value.name.trim()
  const environment = targetForm.value.environment.trim()
  if (!name || !environment) {
    error.value = 'Promotion target name and environment are required.'
    return
  }

  actionLoading.value = true
  clearFeedback()

  try {
    await createPromotionTarget({
      name,
      environment,
      gitRepoUrl: targetForm.value.gitRepoUrl.trim() || null,
      gitBranch: targetForm.value.gitBranch.trim() || null,
      requiresApproval: targetForm.value.requiresApproval,
    })
    resetTargetForm()
    await loadSurface()
    setSuccess('Promotion target created.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to create promotion target.'
  } finally {
    actionLoading.value = false
  }
}

async function handlePromotionRequestCreate() {
  if (!canOperate.value) return

  if (!requestForm.value.workflowId || !requestForm.value.targetId) {
    error.value = 'Select both a workflow and a promotion target.'
    return
  }

  actionLoading.value = true
  clearFeedback()

  try {
    await createPromotionRequest({
      workflowId: requestForm.value.workflowId,
      targetId: requestForm.value.targetId,
      sourceControlRef: requestForm.value.sourceControlRef.trim() || null,
      notes: requestForm.value.notes.trim() || null,
    })
    resetRequestForm()
    await loadSurface()
    setSuccess('Promotion request submitted.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to create promotion request.'
  } finally {
    actionLoading.value = false
  }
}

async function handlePromotionApproval(requestId: string) {
  if (!canAdminister.value) return

  actionLoading.value = true
  clearFeedback()

  try {
    await approvePromotionRequest(requestId, {
      notes: approvalNotes.value[requestId]?.trim() || null,
    })
    approvalNotes.value = {
      ...approvalNotes.value,
      [requestId]: '',
    }
    await loadSurface()
    setSuccess('Promotion request approved.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to approve promotion request.'
  } finally {
    actionLoading.value = false
  }
}

onMounted(loadSurface)
</script>

<template>
  <div class="h-full overflow-y-auto bg-slate-50">
    <div class="mx-auto flex max-w-[1600px] flex-col gap-6 px-4 py-6 md:px-6 lg:px-8">
      <section class="rounded-[28px] border border-slate-200 bg-white p-5 shadow-sm sm:p-6">
        <div class="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
          <div class="max-w-3xl">
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Enterprise governance</p>
            <h2 class="mt-2 text-3xl font-display font-bold tracking-tight text-slate-950">Governance Control Center</h2>
            <p class="mt-3 text-sm leading-6 text-slate-600 sm:text-base">
              Enforce node policy, manage secret-provider posture, route change approvals, and keep a durable audit trail for the active workspace.
            </p>
          </div>

          <div class="flex flex-wrap gap-3">
            <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Workspace role</p>
              <p class="mt-1 font-semibold capitalize text-slate-900">{{ activeRole }}</p>
            </div>
            <button
              type="button"
              class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
              @click="loadSurface"
            >
              <RefreshCw class="h-4 w-4" />
              Refresh
            </button>
          </div>
        </div>
      </section>

      <div v-if="error" class="rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
        {{ error }}
      </div>
      <div v-else-if="success" class="rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
        {{ success }}
      </div>

      <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-6">
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <div class="flex items-center justify-between">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Providers</p>
            <KeyRound class="h-4 w-4 text-slate-400" />
          </div>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ providers.length }}</p>
          <p class="mt-1 text-sm text-slate-500">{{ providerAlerts.length }} need attention.</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <div class="flex items-center justify-between">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Blocked Nodes</p>
            <ShieldAlert class="h-4 w-4 text-slate-400" />
          </div>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ blockedNodesCount }}</p>
          <p class="mt-1 text-sm text-slate-500">Explicitly denied node types.</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <div class="flex items-center justify-between">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Tier Blocks</p>
            <Shield class="h-4 w-4 text-slate-400" />
          </div>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ supportedTierBlocks }}</p>
          <p class="mt-1 text-sm text-slate-500">Support tiers blocked at save time.</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <div class="flex items-center justify-between">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Approval Gates</p>
            <ShieldCheck class="h-4 w-4 text-slate-400" />
          </div>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ approvalGates }}</p>
          <p class="mt-1 text-sm text-slate-500">Node types that require approval.</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <div class="flex items-center justify-between">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Targets</p>
            <FolderGit2 class="h-4 w-4 text-slate-400" />
          </div>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ activeTargets }}</p>
          <p class="mt-1 text-sm text-slate-500">Promotion environments on record.</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <div class="flex items-center justify-between">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Pending</p>
            <FileClock class="h-4 w-4 text-slate-400" />
          </div>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ pendingApprovals.length }}</p>
          <p class="mt-1 text-sm text-slate-500">Approval requests awaiting review.</p>
        </article>
      </section>

      <section class="grid gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)]">
        <div class="space-y-6">
          <article class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
            <div class="flex items-center justify-between gap-4">
              <div>
                <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Policy controls</p>
                <h3 class="mt-2 text-xl font-black text-slate-950">Workspace workflow policy</h3>
                <p class="mt-2 text-sm leading-6 text-slate-500">
                  These controls apply when workflows are created, updated, duplicated, imported, or instantiated from templates.
                </p>
              </div>
              <span class="rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">
                Save-time enforcement
              </span>
            </div>

            <div class="mt-6 grid gap-4 lg:grid-cols-2">
              <label class="block">
                <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Blocked node types</span>
                <textarea
                  v-model="policyForm.blockedNodeTypes"
                  rows="5"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                  placeholder="openAi, github, slack"
                  :disabled="!canAdminister"
                ></textarea>
              </label>

              <label class="block">
                <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Blocked support tiers</span>
                <textarea
                  v-model="policyForm.blockedSupportTiers"
                  rows="5"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                  placeholder="beta, hidden"
                  :disabled="!canAdminister"
                ></textarea>
              </label>

              <label class="block">
                <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Approval-required node types</span>
                <textarea
                  v-model="policyForm.approvalRequiredNodeTypes"
                  rows="5"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                  placeholder="openAi, executeWorkflow"
                  :disabled="!canAdminister"
                ></textarea>
              </label>

              <label class="block">
                <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Max workflow nodes</span>
                <input
                  v-model="policyForm.maxWorkflowNodes"
                  type="number"
                  min="1"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                  placeholder="Leave blank for no explicit cap"
                  :disabled="!canAdminister"
                />
                <p class="mt-2 text-xs text-slate-500">Use an empty value to avoid a hard cap on workflow size.</p>
              </label>
            </div>

            <div class="mt-5 flex justify-end">
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                :disabled="!canAdminister || actionLoading"
                @click="handlePolicySave"
              >
                <Loader2 v-if="actionLoading" class="h-4 w-4 animate-spin" />
                <ShieldCheck v-else class="h-4 w-4" />
                Save Policy
              </button>
            </div>
          </article>

          <article class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
            <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
              <div class="max-w-2xl">
                <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Secret providers</p>
                <h3 class="mt-2 text-xl font-black text-slate-950">External secret provider posture</h3>
                <p class="mt-2 text-sm leading-6 text-slate-500">
                  Add environment or Vault-backed providers so credentials can reference external secret material instead of storing plaintext values.
                </p>
              </div>
              <span class="rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">
                Runtime secret resolution
              </span>
            </div>

            <div class="mt-6 grid gap-4 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
              <div class="rounded-[24px] border border-slate-200 bg-slate-50 p-4">
                <div class="grid gap-4">
                  <label class="block">
                    <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Provider name</span>
                    <input
                      v-model="providerForm.name"
                      type="text"
                      class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                      placeholder="Production Vault"
                      :disabled="!canAdminister"
                    />
                  </label>

                  <label class="block">
                    <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Provider type</span>
                    <select
                      v-model="providerForm.providerType"
                      class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-900 outline-none transition focus:border-sky-400"
                      :disabled="!canAdminister"
                    >
                      <option value="env">Environment</option>
                      <option value="vault">Vault</option>
                    </select>
                  </label>

                  <template v-if="providerForm.providerType === 'vault'">
                    <label class="block">
                      <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Vault address</span>
                      <input
                        v-model="providerForm.vaultAddress"
                        type="url"
                        class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                        placeholder="https://vault.internal:8200"
                        :disabled="!canAdminister"
                      />
                    </label>

                    <label class="block">
                      <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">KV mount path</span>
                      <input
                        v-model="providerForm.vaultMountPath"
                        type="text"
                        class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                        placeholder="secret"
                        :disabled="!canAdminister"
                      />
                    </label>

                    <label class="block">
                      <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Token env var</span>
                      <input
                        v-model="providerForm.vaultTokenEnvVar"
                        type="text"
                        class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                        placeholder="BARQFLOW_VAULT_TOKEN"
                        :disabled="!canAdminister"
                      />
                    </label>
                  </template>

                  <label v-else class="block">
                    <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Variable prefix</span>
                    <input
                      v-model="providerForm.envPrefix"
                      type="text"
                      class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                      placeholder="PROD_AUTOMATION"
                      :disabled="!canAdminister"
                    />
                  </label>

                  <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-600">
                    {{ providerDraftHint }}
                  </div>

                  <button
                    type="button"
                    class="inline-flex items-center justify-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                    :disabled="!canAdminister || actionLoading"
                    @click="handleProviderCreate"
                  >
                    <Loader2 v-if="actionLoading" class="h-4 w-4 animate-spin" />
                    <KeyRound v-else class="h-4 w-4" />
                    Add Provider
                  </button>
                </div>
              </div>

              <div class="space-y-3">
                <div
                  v-if="loading && providers.length === 0"
                  class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-6 text-sm text-slate-500"
                >
                  Loading providers...
                </div>
                <article
                  v-for="provider in providers"
                  :key="provider.id"
                  class="rounded-[24px] border border-slate-200 bg-white p-4"
                >
                  <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                    <div>
                      <div class="flex flex-wrap items-center gap-2">
                        <h4 class="text-base font-black text-slate-950">{{ provider.name }}</h4>
                        <span class="rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] ring-1" :class="statusBadgeClasses(provider.status)">
                          {{ provider.status }}
                        </span>
                        <span class="rounded-full bg-slate-100 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] text-slate-500">
                          {{ provider.providerType }}
                        </span>
                      </div>
                      <p class="mt-2 text-sm text-slate-600">{{ providerSummary(provider) }}</p>
                      <p class="mt-1 text-xs text-slate-500">Last validated: {{ formatDateTime(provider.lastValidatedAt) }}</p>
                      <p v-if="provider.lastError" class="mt-2 text-sm text-rose-600">{{ provider.lastError }}</p>
                    </div>

                    <button
                      type="button"
                      class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-70"
                      :disabled="!canAdminister || validatingProviderId === provider.id"
                      @click="handleProviderValidate(provider.id)"
                    >
                      <Loader2 v-if="validatingProviderId === provider.id" class="h-4 w-4 animate-spin" />
                      <CheckCircle2 v-else class="h-4 w-4" />
                      Validate
                    </button>
                  </div>
                </article>
                <div
                  v-if="!loading && providers.length === 0"
                  class="rounded-2xl border border-dashed border-slate-300 bg-slate-50 px-4 py-8 text-center text-sm text-slate-500"
                >
                  No secret providers registered yet.
                </div>
              </div>
            </div>
          </article>
        </div>

        <div class="space-y-6">
          <article class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
            <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Promotion targets</p>
                <h3 class="mt-2 text-xl font-black text-slate-950">Environment and source-control targets</h3>
                <p class="mt-2 text-sm leading-6 text-slate-500">
                  Define which environments a workflow can move through and whether approval is required before that promotion is accepted.
                </p>
              </div>
              <span class="rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">
                Deployment gates
              </span>
            </div>

            <div class="mt-6 grid gap-4">
              <div class="grid gap-4 md:grid-cols-2">
                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Target name</span>
                  <input
                    v-model="targetForm.name"
                    type="text"
                    class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                    placeholder="Production"
                    :disabled="!canAdminister"
                  />
                </label>
                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Environment</span>
                  <input
                    v-model="targetForm.environment"
                    type="text"
                    class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                    placeholder="production"
                    :disabled="!canAdminister"
                  />
                </label>
                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Git repo URL</span>
                  <input
                    v-model="targetForm.gitRepoUrl"
                    type="url"
                    class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                    placeholder="https://github.com/acme/automation-config"
                    :disabled="!canAdminister"
                  />
                </label>
                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Git branch</span>
                  <input
                    v-model="targetForm.gitBranch"
                    type="text"
                    class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                    placeholder="main"
                    :disabled="!canAdminister"
                  />
                </label>
              </div>

              <label class="flex items-center gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-700">
                <input v-model="targetForm.requiresApproval" type="checkbox" class="h-4 w-4 rounded border-slate-300" :disabled="!canAdminister" />
                Require approval before promotion is marked approved.
              </label>

              <button
                type="button"
                class="inline-flex items-center justify-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                :disabled="!canAdminister || actionLoading"
                @click="handleTargetCreate"
              >
                <Loader2 v-if="actionLoading" class="h-4 w-4 animate-spin" />
                <Waypoints v-else class="h-4 w-4" />
                Add Target
              </button>
            </div>

            <div class="mt-6 space-y-3">
              <article
                v-for="target in targets"
                :key="target.id"
                class="rounded-[24px] border border-slate-200 bg-slate-50 p-4"
              >
                <div class="flex flex-col gap-2">
                  <div class="flex flex-wrap items-center gap-2">
                    <h4 class="text-base font-black text-slate-950">{{ target.name }}</h4>
                    <span class="rounded-full bg-slate-900 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] text-white">
                      {{ target.environment }}
                    </span>
                    <span class="rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] ring-1" :class="target.requiresApproval ? 'bg-amber-50 text-amber-700 ring-amber-200' : 'bg-emerald-50 text-emerald-700 ring-emerald-200'">
                      {{ target.requiresApproval ? 'Approval required' : 'Auto approve' }}
                    </span>
                  </div>
                  <p class="text-sm text-slate-600">
                    {{ target.gitRepoUrl || 'No repo URL configured' }}
                    <span v-if="target.gitBranch" class="text-slate-400"> / {{ target.gitBranch }}</span>
                  </p>
                </div>
              </article>
              <div v-if="!loading && targets.length === 0" class="rounded-2xl border border-dashed border-slate-300 bg-slate-50 px-4 py-8 text-center text-sm text-slate-500">
                No promotion targets registered yet.
              </div>
            </div>
          </article>

          <article class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
            <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Promotion requests</p>
                <h3 class="mt-2 text-xl font-black text-slate-950">Submission and approval queue</h3>
                <p class="mt-2 text-sm leading-6 text-slate-500">
                  Submit a workflow snapshot to a target with an optional source-control ref, then review the approval queue with policy context attached.
                </p>
              </div>
            </div>

            <div class="mt-6 grid gap-4 rounded-[24px] border border-slate-200 bg-slate-50 p-4">
              <div class="grid gap-4 md:grid-cols-2">
                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Workflow</span>
                  <select
                    v-model="requestForm.workflowId"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-900 outline-none transition focus:border-sky-400"
                    :disabled="!canOperate"
                  >
                    <option value="">Select workflow</option>
                    <option v-for="workflow in workflows" :key="workflow.id" :value="workflow.id">
                      {{ workflow.name }}
                    </option>
                  </select>
                </label>

                <label class="block">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Target</span>
                  <select
                    v-model="requestForm.targetId"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-900 outline-none transition focus:border-sky-400"
                    :disabled="!canOperate"
                  >
                    <option value="">Select target</option>
                    <option v-for="target in targets" :key="target.id" :value="target.id">
                      {{ target.name }}
                    </option>
                  </select>
                </label>

                <label class="block md:col-span-2">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Source control ref</span>
                  <input
                    v-model="requestForm.sourceControlRef"
                    type="text"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                    placeholder="refs/heads/release/automation-2026-03"
                    :disabled="!canOperate"
                  />
                </label>

                <label class="block md:col-span-2">
                  <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Notes</span>
                  <textarea
                    v-model="requestForm.notes"
                    rows="4"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                    placeholder="Promotion rationale, rollback notes, or reviewer context"
                    :disabled="!canOperate"
                  ></textarea>
                </label>
              </div>

              <button
                type="button"
                class="inline-flex items-center justify-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                :disabled="!canOperate || actionLoading || workflows.length === 0 || targets.length === 0"
                @click="handlePromotionRequestCreate"
              >
                <Loader2 v-if="actionLoading" class="h-4 w-4 animate-spin" />
                <FolderGit2 v-else class="h-4 w-4" />
                Submit Promotion Request
              </button>
            </div>

            <div class="mt-6 space-y-4">
              <article
                v-for="request in promotionRequests"
                :key="request.id"
                class="rounded-[24px] border border-slate-200 bg-white p-4"
              >
                <div class="flex flex-col gap-4">
                  <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                    <div>
                      <div class="flex flex-wrap items-center gap-2">
                        <h4 class="text-base font-black text-slate-950">{{ workflowNameForRequest(request) }}</h4>
                        <span class="rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.18em] ring-1" :class="statusBadgeClasses(request.status)">
                          {{ request.status }}
                        </span>
                      </div>
                      <p class="mt-2 text-sm text-slate-600">
                        Target: {{ targetName(request.targetId) }}
                        <span v-if="request.sourceControlRef" class="text-slate-400"> / {{ request.sourceControlRef }}</span>
                      </p>
                      <p class="mt-1 text-xs text-slate-500">Requested: {{ formatDateTime(request.requestedAt) }}</p>
                    </div>

                    <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                      <p class="font-semibold text-slate-900">{{ nodeCountForRequest(request) }} nodes in snapshot</p>
                      <p class="mt-1 text-xs text-slate-500">{{ workflowTagsForRequest(request).join(', ') || 'No tags attached' }}</p>
                    </div>
                  </div>

                  <div v-if="approvalReasonsForRequest(request).length > 0" class="rounded-2xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
                    <p class="font-semibold">Approval reasons</p>
                    <p class="mt-2">{{ approvalReasonsForRequest(request).join(' • ') }}</p>
                  </div>

                  <p v-if="request.notes" class="text-sm leading-6 text-slate-600">{{ request.notes }}</p>

                  <div v-if="request.status.toLowerCase() === 'pendingapproval' && canAdminister" class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
                    <label class="block">
                      <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Approval notes</span>
                      <textarea
                        v-model="approvalNotes[request.id]"
                        rows="3"
                        class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-sky-400"
                        placeholder="Review decision, rollout conditions, or follow-up checks"
                      ></textarea>
                    </label>

                    <button
                      type="button"
                      class="inline-flex items-center justify-center gap-2 rounded-2xl bg-emerald-600 px-4 py-3 text-sm font-semibold text-white transition hover:bg-emerald-500 disabled:cursor-not-allowed disabled:bg-emerald-300"
                      :disabled="actionLoading"
                      @click="handlePromotionApproval(request.id)"
                    >
                      <Loader2 v-if="actionLoading" class="h-4 w-4 animate-spin" />
                      <CheckCircle2 v-else class="h-4 w-4" />
                      Approve
                    </button>
                  </div>
                </div>
              </article>

              <div v-if="!loading && promotionRequests.length === 0" class="rounded-2xl border border-dashed border-slate-300 bg-slate-50 px-4 py-8 text-center text-sm text-slate-500">
                No promotion requests recorded yet.
              </div>
            </div>
          </article>
        </div>
      </section>

      <section class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
        <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div class="max-w-3xl">
            <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Audit log</p>
            <h3 class="mt-2 text-xl font-black text-slate-950">Governance activity trail</h3>
            <p class="mt-2 text-sm leading-6 text-slate-500">
              Every provider change, policy edit, promotion request, and approval event is captured here with actor and resource context.
            </p>
          </div>
        </div>

        <div class="mt-6 overflow-hidden rounded-[24px] border border-slate-200">
          <div class="grid grid-cols-[minmax(0,1.4fr)_minmax(0,0.8fr)_minmax(0,0.8fr)_minmax(0,0.9fr)] gap-4 border-b border-slate-200 bg-slate-50 px-5 py-3 text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">
            <p>Action</p>
            <p>Resource</p>
            <p>Actor</p>
            <p>Created</p>
          </div>

          <div v-if="loading && auditLogs.length === 0" class="px-5 py-6 text-sm text-slate-500">
            Loading audit activity...
          </div>

          <div v-else-if="auditLogs.length === 0" class="px-5 py-10 text-center text-sm text-slate-500">
            No governance audit events are available yet.
          </div>

          <div v-else class="divide-y divide-slate-200">
            <article
              v-for="log in auditLogs"
              :key="log.id"
              class="grid grid-cols-1 gap-4 px-5 py-4 text-sm text-slate-600 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,0.8fr)_minmax(0,0.8fr)_minmax(0,0.9fr)]"
            >
              <div>
                <p class="font-semibold text-slate-900">{{ log.summary }}</p>
                <p class="mt-1 text-xs text-slate-500">{{ log.action }}</p>
              </div>
              <div>
                <p class="font-semibold text-slate-900">{{ log.resourceType }}</p>
                <p class="mt-1 text-xs text-slate-500">{{ log.resourceId || 'Workspace scope' }}</p>
              </div>
              <div>
                <p class="font-semibold text-slate-900">{{ log.actorEmail || 'System' }}</p>
                <p class="mt-1 text-xs text-slate-500">{{ log.actorUserId || 'No actor id' }}</p>
              </div>
              <div>
                <p class="font-semibold text-slate-900">{{ formatDateTime(log.createdAt) }}</p>
              </div>
            </article>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
