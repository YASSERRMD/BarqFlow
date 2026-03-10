<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  AlertTriangle,
  ArrowRightLeft,
  Building2,
  CheckCircle2,
  Clock3,
  KeyRound,
  LockKeyhole,
  RefreshCw,
  Server,
  ShieldCheck,
  UserPlus,
  Users2,
  Workflow,
} from 'lucide-vue-next'
import { useAuthStore } from '../stores/auth'
import {
  addWorkspaceMember,
  changePassword,
  createApiKey,
  createWorkspace,
  getOperationsOverview,
  getRuntimeSettings,
  listApiKeys,
  listWorkspaceMembers,
  listWorkspaces,
  pruneExecutions,
  revokeApiKey,
  selectWorkspace,
} from '../features/settings/api'
import type {
  ApiKeyCreateResult,
  ApiKeyRecord,
  OperationsOverview,
  RuntimeSettings,
  WorkspaceMember,
  WorkspaceSummary,
} from '../types/contracts'

const authStore = useAuthStore()

const runtime = ref<RuntimeSettings | null>(null)
const operations = ref<OperationsOverview | null>(null)
const workspaces = ref<WorkspaceSummary[]>([])
const members = ref<WorkspaceMember[]>([])
const apiKeys = ref<ApiKeyRecord[]>([])

const loading = ref(false)
const pruningLoading = ref(false)
const error = ref<string | null>(null)
const successMessage = ref<string | null>(null)
const createdApiKey = ref<ApiKeyCreateResult | null>(null)
const switchingWorkspaceId = ref<string | null>(null)

const workspaceForm = ref({ name: '' })
const memberForm = ref({ email: '', role: 'member' })
const apiKeyForm = ref({ name: '', expiresAt: '' })
const passwordForm = ref({ currentPassword: '', newPassword: '', confirmPassword: '' })

const activeWorkspace = computed(() => authStore.activeWorkspace)
const canWriteWorkspace = computed(() => ['owner', 'admin', 'member'].includes(authStore.user?.workspaceRole || ''))
const canManageMembers = computed(() => ['owner', 'admin'].includes(authStore.user?.workspaceRole || ''))
const activeApiKeysCount = computed(() => apiKeys.value.filter((key) => !key.revokedAt).length)
const workspaceMemberCount = computed(() => members.value.length)
const canRunPrune = computed(() => ['owner', 'admin'].includes(authStore.user?.workspaceRole || '') && !!operations.value?.pruning.enabled)
const formattedServerTime = computed(() => {
  if (!runtime.value?.serverTime) return '-'
  return new Date(runtime.value.serverTime).toLocaleString()
})

function setFeedback(message: string) {
  successMessage.value = message
  error.value = null
}

async function loadSettingsSurface() {
  loading.value = true
  error.value = null
  try {
    const [runtimeResponse, operationsResponse, workspacesResponse, membersResponse, apiKeysResponse] = await Promise.all([
      getRuntimeSettings(),
      getOperationsOverview(),
      listWorkspaces(),
      listWorkspaceMembers(),
      listApiKeys(),
    ])

    runtime.value = runtimeResponse.data
    operations.value = operationsResponse.data
    workspaces.value = workspacesResponse.data
    members.value = membersResponse.data
    apiKeys.value = apiKeysResponse.data
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to load settings'
  } finally {
    loading.value = false
  }
}

async function refreshSurface() {
  await authStore.fetchMe()
  await loadSettingsSurface()
}

async function handleWorkspaceSwitch(workspaceId: string) {
  if (!workspaceId || workspaceId === activeWorkspace.value?.id) return
  switchingWorkspaceId.value = workspaceId
  error.value = null
  successMessage.value = null
  try {
    await selectWorkspace(workspaceId)
    await authStore.fetchMe()
    await loadSettingsSurface()
    setFeedback('Active workspace updated.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to switch workspace'
  } finally {
    switchingWorkspaceId.value = null
  }
}

async function handleWorkspaceCreate() {
  const name = workspaceForm.value.name.trim()
  if (!name) {
    error.value = 'Workspace name is required.'
    return
  }

  try {
    await createWorkspace({ name })
    workspaceForm.value.name = ''
    await authStore.fetchMe()
    await loadSettingsSurface()
    setFeedback('Workspace created.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to create workspace'
  }
}

async function handleMemberInvite() {
  if (!canManageMembers.value) return

  const email = memberForm.value.email.trim()
  if (!email) {
    error.value = 'Member email is required.'
    return
  }

  try {
    await addWorkspaceMember({ email, role: memberForm.value.role })
    memberForm.value.email = ''
    memberForm.value.role = 'member'
    await loadSettingsSurface()
    setFeedback('Workspace membership updated.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to update workspace membership'
  }
}

async function handleApiKeyCreate() {
  if (!canWriteWorkspace.value) return

  const name = apiKeyForm.value.name.trim()
  if (!name) {
    error.value = 'API key name is required.'
    return
  }

  try {
    const response = await createApiKey({
      name,
      expiresAt: apiKeyForm.value.expiresAt || null,
    })
    createdApiKey.value = response.data
    apiKeyForm.value.name = ''
    apiKeyForm.value.expiresAt = ''
    await loadSettingsSurface()
    setFeedback('API key created. Copy it now; the raw value is only shown once.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to create API key'
  }
}

async function handleApiKeyRevoke(apiKeyId: string) {
  if (!canWriteWorkspace.value) return

  try {
    await revokeApiKey(apiKeyId)
    await loadSettingsSurface()
    setFeedback('API key revoked.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to revoke API key'
  }
}

async function copyCreatedApiKey() {
  if (!createdApiKey.value?.apiKey) return
  await navigator.clipboard.writeText(createdApiKey.value.apiKey)
  setFeedback('API key copied to clipboard.')
}

async function handlePasswordChange() {
  if (passwordForm.value.newPassword.trim().length < 8) {
    error.value = 'New password must be at least 8 characters.'
    return
  }

  if (passwordForm.value.newPassword !== passwordForm.value.confirmPassword) {
    error.value = 'Password confirmation does not match.'
    return
  }

  try {
    const response = await changePassword({
      currentPassword: passwordForm.value.currentPassword,
      newPassword: passwordForm.value.newPassword,
    })
    authStore.user = response.data
    passwordForm.value.currentPassword = ''
    passwordForm.value.newPassword = ''
    passwordForm.value.confirmPassword = ''
    setFeedback('Password updated.')
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to update password'
  }
}

async function handlePruneExecutions() {
  if (!canRunPrune.value) return

  pruningLoading.value = true
  error.value = null
  successMessage.value = null

  try {
    const response = await pruneExecutions()
    await loadSettingsSurface()
    setFeedback(
      `Execution retention run completed. Deleted ${response.data.executionsDeleted} executions, ${response.data.logsDeleted} log records, and ${response.data.waitResumesDeleted} wait tokens.`,
    )
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to prune execution data'
  } finally {
    pruningLoading.value = false
  }
}

onMounted(refreshSurface)
</script>

<template>
  <div class="h-full overflow-auto bg-slate-50 px-4 py-6 md:px-8 md:py-8">
    <div class="mx-auto max-w-7xl space-y-6">
      <section class="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm md:p-8">
        <div class="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div class="space-y-3">
            <p class="text-[11px] font-bold uppercase tracking-[0.24em] text-slate-500">Control Plane Settings</p>
            <div>
              <h1 class="text-3xl font-display font-bold tracking-tight text-slate-950">Workspace, access, and runtime operations</h1>
              <p class="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
                Manage workspace boundaries, operator access, service credentials, and runtime posture from one administrative surface.
              </p>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Active workspace</p>
              <p class="mt-1 text-sm font-semibold text-slate-950">{{ activeWorkspace?.name || 'No workspace' }}</p>
            </div>
            <button
              class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:opacity-60"
              :disabled="loading"
              @click="refreshSurface"
            >
              <RefreshCw class="h-4 w-4" />
              Refresh
            </button>
          </div>
        </div>
      </section>

      <div v-if="error" class="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
        {{ error }}
      </div>
      <div v-if="successMessage" class="rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
        {{ successMessage }}
      </div>
      <div v-if="createdApiKey" class="rounded-[2rem] border border-sky-200 bg-sky-50 p-5 shadow-sm">
        <div class="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
          <div>
            <p class="text-sm font-semibold text-sky-900">New API key ready</p>
            <p class="mt-1 text-sm text-sky-800">This raw key will not be shown again after you leave this screen.</p>
            <code class="mt-4 block overflow-x-auto rounded-2xl bg-slate-950 px-4 py-3 text-xs text-slate-100">{{ createdApiKey.apiKey }}</code>
          </div>
          <button
            class="inline-flex items-center justify-center rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800"
            @click="copyCreatedApiKey"
          >
            Copy key
          </button>
        </div>
      </div>

      <div v-if="loading" class="rounded-[2rem] border border-slate-200 bg-white p-6 text-sm text-slate-500 shadow-sm">
        Loading workspace settings...
      </div>

      <template v-else>
        <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <div class="rounded-[1.75rem] border border-slate-200 bg-white p-5 shadow-sm">
            <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
              <Building2 class="h-4 w-4" />
              Active workspace
            </div>
            <p class="mt-3 text-2xl font-semibold text-slate-950">{{ activeWorkspace?.name || '-' }}</p>
            <p class="mt-2 text-xs uppercase tracking-[0.18em] text-slate-500">{{ activeWorkspace?.role || authStore.user?.workspaceRole || 'viewer' }}</p>
          </div>

          <div class="rounded-[1.75rem] border border-slate-200 bg-white p-5 shadow-sm">
            <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
              <Users2 class="h-4 w-4" />
              Team members
            </div>
            <p class="mt-3 text-2xl font-semibold text-slate-950">{{ workspaceMemberCount }}</p>
            <p class="mt-2 text-sm text-slate-500">Operators in the current workspace boundary.</p>
          </div>

          <div class="rounded-[1.75rem] border border-slate-200 bg-white p-5 shadow-sm">
            <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
              <KeyRound class="h-4 w-4" />
              Active API keys
            </div>
            <p class="mt-3 text-2xl font-semibold text-slate-950">{{ activeApiKeysCount }}</p>
            <p class="mt-2 text-sm text-slate-500">Non-revoked machine access credentials.</p>
          </div>

          <div class="rounded-[1.75rem] border border-slate-200 bg-white p-5 shadow-sm">
            <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
              <Server class="h-4 w-4" />
              Runtime environment
            </div>
            <p class="mt-3 text-2xl font-semibold capitalize text-slate-950">{{ runtime?.environment || '-' }}</p>
            <p class="mt-2 text-sm text-slate-500">Server time {{ formattedServerTime }}</p>
          </div>
        </section>

        <section class="grid gap-6 xl:grid-cols-[1.2fr,0.8fr]">
          <div class="space-y-6">
            <div class="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-sm font-semibold text-slate-950">Workspace directory</p>
                  <p class="mt-1 text-sm text-slate-500">Switch operating context or establish a new workspace boundary for a separate team.</p>
                </div>
                <ArrowRightLeft class="mt-1 h-5 w-5 text-slate-400" />
              </div>

              <div class="mt-5 grid gap-3">
                <button
                  v-for="workspace in workspaces"
                  :key="workspace.id"
                  class="flex items-center justify-between rounded-2xl border px-4 py-4 text-left transition"
                  :class="workspace.id === activeWorkspace?.id ? 'border-slate-950 bg-slate-950 text-white' : 'border-slate-200 bg-slate-50 text-slate-900 hover:bg-white'"
                  :disabled="switchingWorkspaceId === workspace.id"
                  @click="handleWorkspaceSwitch(workspace.id)"
                >
                  <div>
                    <p class="text-sm font-semibold">{{ workspace.name }}</p>
                    <p :class="workspace.id === activeWorkspace?.id ? 'text-slate-300' : 'text-slate-500'" class="mt-1 text-xs uppercase tracking-[0.18em]">
                      {{ workspace.role }}
                    </p>
                  </div>
                  <span class="rounded-full px-3 py-1 text-xs font-semibold" :class="workspace.id === activeWorkspace?.id ? 'bg-white/10 text-white' : 'bg-white text-slate-600'">
                    {{ workspace.id === activeWorkspace?.id ? 'Active' : 'Switch' }}
                  </span>
                </button>
              </div>

              <div class="mt-6 rounded-[1.75rem] border border-slate-200 bg-slate-50 p-4">
                <div class="flex items-center gap-2 text-sm font-semibold text-slate-900">
                  <Building2 class="h-4 w-4 text-slate-500" />
                  Create workspace
                </div>
                <div class="mt-4 flex flex-col gap-3 md:flex-row">
                  <input
                    v-model="workspaceForm.name"
                    type="text"
                    placeholder="Platform Reliability"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400"
                  />
                  <button
                    class="inline-flex items-center justify-center rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800"
                    @click="handleWorkspaceCreate"
                  >
                    Create
                  </button>
                </div>
              </div>
            </div>

            <div class="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-sm font-semibold text-slate-950">Team access</p>
                  <p class="mt-1 text-sm text-slate-500">Assign operators to the current workspace with explicit ownership and execution privileges.</p>
                </div>
                <Users2 class="mt-1 h-5 w-5 text-slate-400" />
              </div>

              <div class="mt-5 space-y-3">
                <div
                  v-for="member in members"
                  :key="member.membershipId"
                  class="flex flex-col gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4 md:flex-row md:items-center md:justify-between"
                >
                  <div>
                    <p class="text-sm font-semibold text-slate-950">{{ member.firstName || member.email }}</p>
                    <p class="mt-1 text-sm text-slate-500">{{ member.email }}</p>
                  </div>
                  <div class="flex items-center gap-3 text-xs uppercase tracking-[0.16em] text-slate-500">
                    <span class="rounded-full bg-white px-3 py-1.5 font-semibold text-slate-700">{{ member.role }}</span>
                    <span>Joined {{ new Date(member.createdAt).toLocaleDateString() }}</span>
                  </div>
                </div>
              </div>

              <div class="mt-6 rounded-[1.75rem] border border-slate-200 bg-slate-50 p-4">
                <div class="flex items-center gap-2 text-sm font-semibold text-slate-900">
                  <UserPlus class="h-4 w-4 text-slate-500" />
                  Add or update member
                </div>
                <div class="mt-4 grid gap-3 md:grid-cols-[1.2fr,0.8fr,auto]">
                  <input
                    v-model="memberForm.email"
                    type="email"
                    placeholder="operator@company.com"
                    :disabled="!canManageMembers"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400 disabled:cursor-not-allowed disabled:bg-slate-100"
                  />
                  <select
                    v-model="memberForm.role"
                    :disabled="!canManageMembers"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400 disabled:cursor-not-allowed disabled:bg-slate-100"
                  >
                    <option value="viewer">Viewer</option>
                    <option value="member">Member</option>
                    <option value="admin">Admin</option>
                    <option value="owner">Owner</option>
                  </select>
                  <button
                    class="inline-flex items-center justify-center rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                    :disabled="!canManageMembers"
                    @click="handleMemberInvite"
                  >
                    Save
                  </button>
                </div>
                <p v-if="!canManageMembers" class="mt-3 text-xs text-slate-500">Only workspace admins and owners can change membership.</p>
              </div>
            </div>
          </div>

          <div class="space-y-6">
            <div class="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-sm font-semibold text-slate-950">Machine access</p>
                  <p class="mt-1 text-sm text-slate-500">Issue and revoke API keys for agents, CI jobs, and service integrations.</p>
                </div>
                <KeyRound class="mt-1 h-5 w-5 text-slate-400" />
              </div>

              <div class="mt-5 space-y-3">
                <div
                  v-for="key in apiKeys"
                  :key="key.id"
                  class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4"
                >
                  <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                    <div>
                      <p class="text-sm font-semibold text-slate-950">{{ key.name }}</p>
                      <p class="mt-1 text-sm text-slate-500">{{ key.keyPrefix }}</p>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="rounded-full px-3 py-1 text-xs font-semibold" :class="key.revokedAt ? 'bg-red-100 text-red-700' : 'bg-emerald-100 text-emerald-700'">
                        {{ key.revokedAt ? 'Revoked' : 'Active' }}
                      </span>
                      <button
                        v-if="!key.revokedAt"
                        class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-semibold text-slate-700 transition hover:bg-slate-100 disabled:opacity-50"
                        :disabled="!canWriteWorkspace"
                        @click="handleApiKeyRevoke(key.id)"
                      >
                        Revoke
                      </button>
                    </div>
                  </div>
                  <div class="mt-3 grid gap-2 text-xs text-slate-500">
                    <p>Created {{ new Date(key.createdAt).toLocaleString() }}</p>
                    <p v-if="key.lastUsedAt">Last used {{ new Date(key.lastUsedAt).toLocaleString() }}</p>
                    <p v-if="key.expiresAt">Expires {{ new Date(key.expiresAt).toLocaleString() }}</p>
                  </div>
                </div>
              </div>

              <div class="mt-6 rounded-[1.75rem] border border-slate-200 bg-slate-50 p-4">
                <div class="flex items-center gap-2 text-sm font-semibold text-slate-900">
                  <KeyRound class="h-4 w-4 text-slate-500" />
                  Create API key
                </div>
                <div class="mt-4 space-y-3">
                  <input
                    v-model="apiKeyForm.name"
                    type="text"
                    placeholder="CI deploy runner"
                    :disabled="!canWriteWorkspace"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400 disabled:cursor-not-allowed disabled:bg-slate-100"
                  />
                  <input
                    v-model="apiKeyForm.expiresAt"
                    type="datetime-local"
                    :disabled="!canWriteWorkspace"
                    class="w-full rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400 disabled:cursor-not-allowed disabled:bg-slate-100"
                  />
                  <button
                    class="inline-flex w-full items-center justify-center rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                    :disabled="!canWriteWorkspace"
                    @click="handleApiKeyCreate"
                  >
                    Create machine credential
                  </button>
                </div>
              </div>
            </div>

            <div class="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-sm font-semibold text-slate-950">Password and session security</p>
                  <p class="mt-1 text-sm text-slate-500">Rotate the operator password used for interactive access to this control plane.</p>
                </div>
                <LockKeyhole class="mt-1 h-5 w-5 text-slate-400" />
              </div>

              <div class="mt-5 space-y-3">
                <input
                  v-model="passwordForm.currentPassword"
                  type="password"
                  placeholder="Current password"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400"
                />
                <input
                  v-model="passwordForm.newPassword"
                  type="password"
                  placeholder="New password"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400"
                />
                <input
                  v-model="passwordForm.confirmPassword"
                  type="password"
                  placeholder="Confirm new password"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-slate-400"
                />
                <button
                  class="inline-flex w-full items-center justify-center rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800"
                  @click="handlePasswordChange"
                >
                  Update password
                </button>
              </div>
            </div>

            <div class="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-sm font-semibold text-slate-950">Runtime posture</p>
                  <p class="mt-1 text-sm text-slate-500">Runtime health, worker dispatch, telemetry hooks, and retention controls for this BarqFlow deployment.</p>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-2 text-xs font-semibold text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
                    :disabled="!canRunPrune || pruningLoading"
                    @click="handlePruneExecutions"
                  >
                    <RefreshCw :class="['h-3.5 w-3.5', pruningLoading ? 'animate-spin' : '']" />
                    {{ pruningLoading ? 'Pruning…' : 'Run prune' }}
                  </button>
                  <ShieldCheck class="mt-1 h-5 w-5 text-slate-400" />
                </div>
              </div>

              <div class="mt-5 grid gap-3">
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <Clock3 class="h-4 w-4" />
                    Server time
                  </div>
                  <p class="mt-2 text-base font-semibold text-slate-950">{{ formattedServerTime }}</p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <Server class="h-4 w-4" />
                    Registered node types
                  </div>
                  <p class="mt-2 text-base font-semibold text-slate-950">{{ runtime?.nodeTypesCount || 0 }}</p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <Workflow class="h-4 w-4" />
                    Registered credential types
                  </div>
                  <p class="mt-2 text-base font-semibold text-slate-950">{{ runtime?.credentialTypesCount || 0 }}</p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <AlertTriangle class="h-4 w-4" />
                    Encryption key status
                  </div>
                  <p class="mt-2 inline-flex items-center gap-2 text-base font-semibold" :class="runtime?.encryptionKeyConfigured ? 'text-emerald-700' : 'text-red-700'">
                    <CheckCircle2 v-if="runtime?.encryptionKeyConfigured" class="h-4 w-4" />
                    <AlertTriangle v-else class="h-4 w-4" />
                    {{ runtime?.encryptionKeyConfigured ? 'Configured' : 'Missing' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <Server class="h-4 w-4" />
                    Execution dispatch
                  </div>
                  <p class="mt-2 text-base font-semibold capitalize text-slate-950">
                    {{ operations?.dispatch.mode || runtime?.executionMode || '-' }}
                  </p>
                  <p class="mt-1 text-xs text-slate-500">
                    {{ operations?.dispatch.runningCount ?? 0 }} running, {{ operations?.dispatch.queuedCount ?? 0 }} open, {{ runtime?.runWorkerConcurrency || operations?.dispatch.runWorkerConcurrency || 0 }} run / {{ runtime?.triggerWorkerConcurrency || operations?.dispatch.triggerWorkerConcurrency || 0 }} trigger workers
                  </p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <Workflow class="h-4 w-4" />
                    Queue capacity
                  </div>
                  <p class="mt-2 text-base font-semibold text-slate-950">
                    {{ operations?.dispatch.queueCapacity ?? runtime?.queueCapacity ?? 0 }}
                  </p>
                  <p class="mt-1 text-xs text-slate-500">
                    {{ operations?.dispatch.runQueuedCount ?? 0 }} run-lane, {{ operations?.dispatch.triggerQueuedCount ?? 0 }} trigger-lane items
                  </p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <CheckCircle2 class="h-4 w-4" />
                    Telemetry and tracing
                  </div>
                  <p class="mt-2 text-base font-semibold text-slate-950">
                    {{ operations?.telemetry.enabled || runtime?.tracingEnabled ? 'Enabled' : 'Disabled' }}
                  </p>
                  <p class="mt-1 text-xs text-slate-500">
                    {{ operations?.telemetry.format || runtime?.traceFormat || 'pretty' }} format via {{ operations?.telemetry.requestIdHeader || 'x-request-id' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <div class="flex items-center gap-2 text-sm font-medium text-slate-500">
                    <AlertTriangle class="h-4 w-4" />
                    Execution retention
                  </div>
                  <p class="mt-2 text-base font-semibold text-slate-950">
                    {{ operations?.pruning.enabled || runtime?.pruningEnabled ? `${operations?.pruning.retentionDays || runtime?.executionRetentionDays || 0} days` : 'Disabled' }}
                  </p>
                  <p class="mt-1 text-xs text-slate-500">
                    Last run {{ operations?.pruning.lastRunAt ? new Date(operations.pruning.lastRunAt).toLocaleString() : 'not yet executed' }}
                  </p>
                </div>
              </div>

              <div class="mt-5 rounded-[1.75rem] border border-slate-200 bg-slate-50 p-4">
                <div class="grid gap-3 md:grid-cols-2">
                  <div>
                    <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">Dispatch totals</p>
                    <div class="mt-3 grid gap-3 sm:grid-cols-3">
                      <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-slate-400">Started</p>
                        <p class="mt-1 text-lg font-semibold text-slate-950">{{ operations?.dispatch.totalStarted ?? 0 }}</p>
                      </div>
                      <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-slate-400">Finished</p>
                        <p class="mt-1 text-lg font-semibold text-slate-950">{{ operations?.dispatch.totalFinished ?? 0 }}</p>
                      </div>
                      <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-slate-400">Rejected</p>
                        <p class="mt-1 text-lg font-semibold text-slate-950">{{ operations?.dispatch.totalFailedToDispatch ?? 0 }}</p>
                      </div>
                    </div>
                  </div>
                  <div>
                    <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">Trigger footprint</p>
                    <div class="mt-3 grid gap-3 sm:grid-cols-3">
                      <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-slate-400">Active runs</p>
                        <p class="mt-1 text-lg font-semibold text-slate-950">{{ operations?.activeExecutions ?? 0 }}</p>
                      </div>
                      <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-slate-400">Webhooks</p>
                        <p class="mt-1 text-lg font-semibold text-slate-950">{{ operations?.webhookEndpointCount ?? 0 }}</p>
                      </div>
                      <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-slate-400">Cron jobs</p>
                        <p class="mt-1 text-lg font-semibold text-slate-950">{{ operations?.cronJobCount ?? 0 }}</p>
                      </div>
                    </div>
                  </div>
                </div>

                <p v-if="!operations?.pruning.enabled && !runtime?.pruningEnabled" class="mt-4 text-xs text-slate-500">
                  Automatic execution retention is disabled for this deployment.
                </p>
              </div>
            </div>
          </div>
        </section>
      </template>
    </div>
  </div>
</template>
