<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import {
  Activity,
  Clock,
  Copy,
  Loader2,
  RefreshCw,
  RotateCcw,
  Search,
  Square,
  Trash2,
  X,
} from 'lucide-vue-next'
import {
  createExecutionEventSource,
  deleteExecution as deleteExecutionRequest,
  getExecution,
  getExecutionEvents,
  listExecutions,
  retryExecution as retryExecutionRequest,
  stopExecution as stopExecutionRequest,
} from '../features/executions/api'
import type { ExecutionEvent, ExecutionRecord } from '../types/contracts'
import {
  extractExecutionEvents,
  extractExecutionMeta,
  extractExecutionNodeResults,
  extractExecutionWaitDetails,
  isTerminalExecutionEvent,
  mergeExecutionEvents,
  resolveExecutionStatusFromEvent,
} from '../features/executions/helpers'
import ExecutionStatusBadge from '../features/executions/components/ExecutionStatusBadge.vue'
import ExecutionTimeline from '../features/executions/components/ExecutionTimeline.vue'

const executions = ref<ExecutionRecord[]>([])
const selectedExecutionId = ref<string | null>(null)
const executionEvents = ref<Record<string, ExecutionEvent[]>>({})
const loading = ref(false)
const detailsLoading = ref(false)
const actionLoading = ref(false)
const error = ref<string | null>(null)
const query = ref('')
const statusFilter = ref('all')
const copiedResumeUrl = ref(false)

let eventSource: EventSource | null = null

const selectedExecution = computed(() => {
  if (!selectedExecutionId.value) return null
  return executions.value.find((execution) => execution.id === selectedExecutionId.value) || null
})

const filteredExecutions = computed(() => {
  const q = query.value.trim().toLowerCase()
  return executions.value.filter((execution) => {
    const matchesStatus =
      statusFilter.value === 'all' ||
      execution.status.toLowerCase() === statusFilter.value.toLowerCase()
    const matchesQuery =
      q.length === 0 ||
      execution.id.toLowerCase().includes(q) ||
      execution.workflowId.toLowerCase().includes(q)

    return matchesStatus && matchesQuery
  })
})

const selectedEvents = computed(() => {
  if (!selectedExecution.value) return []
  return mergeExecutionEvents(
    extractExecutionEvents(selectedExecution.value),
    executionEvents.value[selectedExecution.value.id] || [],
  )
})

const selectedMeta = computed(() => extractExecutionMeta(selectedExecution.value))
const nodeResults = computed(() => extractExecutionNodeResults(selectedExecution.value))
const waitDetails = computed(() => extractExecutionWaitDetails(selectedExecution.value))

const executionJson = computed(() => {
  if (!selectedExecution.value) return ''
  return JSON.stringify(selectedExecution.value.data, null, 2)
})

function stopExecutionStream() {
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }
}

function upsertExecution(updated: ExecutionRecord) {
  const existingIndex = executions.value.findIndex((execution) => execution.id === updated.id)
  if (existingIndex === -1) {
    executions.value = [updated, ...executions.value]
    return
  }

  executions.value.splice(existingIndex, 1, updated)
}

async function syncExecution(id: string) {
  const response = await getExecution(id)
  upsertExecution(response.data)
  return response.data
}

async function loadExecutionEvents(id: string) {
  const response = await getExecutionEvents(id)
  executionEvents.value[id] = mergeExecutionEvents(executionEvents.value[id] || [], response.data)
}

function applyExecutionEvent(executionId: string, event: ExecutionEvent) {
  executionEvents.value[executionId] = mergeExecutionEvents(
    executionEvents.value[executionId] || [],
    [event],
  )

  const match = executions.value.find((execution) => execution.id === executionId)
  if (!match) return

  match.status = resolveExecutionStatusFromEvent(event)
}

function startExecutionStream(executionId: string) {
  stopExecutionStream()
  const source = createExecutionEventSource(executionId)

  source.addEventListener('execution', async (rawEvent) => {
    const parsed = JSON.parse((rawEvent as MessageEvent<string>).data) as ExecutionEvent
    applyExecutionEvent(executionId, parsed)

    if (isTerminalExecutionEvent(parsed)) {
      stopExecutionStream()
      try {
        await syncExecution(executionId)
      } catch {
        // Keep the streamed timeline even if the terminal refresh fails.
      }
    }
  })

  source.onerror = async () => {
    stopExecutionStream()
    try {
      await syncExecution(executionId)
    } catch {
      // Preserve the last known state if refresh fails.
    }
  }

  eventSource = source
}

function formatDuration(execution: ExecutionRecord): string {
  if (!execution.stoppedAt) {
    return execution.status === 'running' ? 'Live' : 'Open'
  }

  const startedAt = new Date(execution.startedAt).getTime()
  const stoppedAt = new Date(execution.stoppedAt).getTime()
  const durationMs = Math.max(0, stoppedAt - startedAt)

  if (durationMs < 1000) return `${durationMs}ms`
  return `${(durationMs / 1000).toFixed(2)}s`
}

function formatRelativeTime(value: string) {
  const date = new Date(value).getTime()
  const diffSeconds = Math.floor((date - Date.now()) / 1000)
  const formatter = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })

  const abs = Math.abs(diffSeconds)
  if (abs < 60) return formatter.format(diffSeconds, 'second')

  const minutes = Math.floor(diffSeconds / 60)
  if (Math.abs(minutes) < 60) return formatter.format(minutes, 'minute')

  const hours = Math.floor(minutes / 60)
  if (Math.abs(hours) < 24) return formatter.format(hours, 'hour')

  const days = Math.floor(hours / 24)
  return formatter.format(days, 'day')
}

function formatTimestamp(value?: string | null) {
  if (!value) return 'Unknown'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

async function fetchExecutions() {
  loading.value = true
  error.value = null

  try {
    const response = await listExecutions({ limit: 100 })
    executions.value = response.data

    if (selectedExecutionId.value) {
      const stillExists = response.data.some((execution) => execution.id === selectedExecutionId.value)
      if (!stillExists) {
        selectedExecutionId.value = null
        stopExecutionStream()
      }
    }
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.message || 'Failed to fetch executions'
  } finally {
    loading.value = false
  }
}

async function openExecutionDetails(execution: ExecutionRecord) {
  selectedExecutionId.value = execution.id
  detailsLoading.value = true
  copiedResumeUrl.value = false
  error.value = null

  try {
    const latest = await syncExecution(execution.id)
    await loadExecutionEvents(execution.id)

    if (latest.status === 'running' || latest.status === 'queued') {
      startExecutionStream(execution.id)
    } else {
      stopExecutionStream()
    }
  } catch (err: any) {
    error.value = err?.response?.data || err?.message || 'Failed to load execution details'
  } finally {
    detailsLoading.value = false
  }
}

function closeExecutionDetails() {
  selectedExecutionId.value = null
  copiedResumeUrl.value = false
  stopExecutionStream()
}

async function retryExecution(id: string) {
  actionLoading.value = true
  error.value = null

  try {
    const response = await retryExecutionRequest(id)
    await fetchExecutions()
    const nextExecution =
      executions.value.find((execution) => execution.id === response.data.id) || response.data
    await openExecutionDetails(nextExecution)
  } catch (err: any) {
    error.value = err?.response?.data || err?.message || 'Failed to retry execution'
  } finally {
    actionLoading.value = false
  }
}

async function stopExecution(id: string) {
  actionLoading.value = true
  error.value = null

  try {
    await stopExecutionRequest(id)
    await syncExecution(id)
    await loadExecutionEvents(id)
  } catch (err: any) {
    error.value = err?.response?.data || err?.message || 'Failed to stop execution'
  } finally {
    actionLoading.value = false
  }
}

async function deleteExecution(id: string) {
  const confirmed = window.confirm('Delete this execution record?')
  if (!confirmed) return

  actionLoading.value = true
  error.value = null

  try {
    await deleteExecutionRequest(id)
    executions.value = executions.value.filter((execution) => execution.id !== id)
    delete executionEvents.value[id]
    if (selectedExecutionId.value === id) {
      closeExecutionDetails()
    }
  } catch (err: any) {
    error.value = err?.response?.data || err?.message || 'Failed to delete execution'
  } finally {
    actionLoading.value = false
  }
}

async function copyResumeUrl() {
  if (!waitDetails.value?.resumeUrl) return
  await navigator.clipboard.writeText(waitDetails.value.resumeUrl)
  copiedResumeUrl.value = true
  window.setTimeout(() => {
    copiedResumeUrl.value = false
  }, 2000)
}

onMounted(fetchExecutions)
onBeforeUnmount(stopExecutionStream)
</script>

<template>
  <div class="h-full overflow-auto bg-slate-50 px-4 py-4 md:px-8 md:py-8">
    <div class="mx-auto max-w-7xl space-y-6">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.22em] text-brand-600">Runtime</p>
          <h1 class="mt-2 text-3xl font-bold text-slate-900">Execution History</h1>
          <p class="mt-2 max-w-2xl text-sm text-slate-500">
            Inspect workflow runs, follow live execution events, and debug node failures without
            dropping back to raw logs.
          </p>
        </div>

        <div class="flex flex-col gap-3 sm:flex-row">
          <button
            class="inline-flex items-center justify-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-50 disabled:opacity-60"
            :disabled="loading || actionLoading"
            @click="fetchExecutions"
          >
            <RefreshCw class="h-4 w-4" />
            Refresh
          </button>
          <div class="relative">
            <Search class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
            <input
              v-model="query"
              type="text"
              placeholder="Search execution or workflow id"
              class="w-full rounded-2xl border border-slate-200 bg-white py-2.5 pl-9 pr-4 text-sm text-slate-700 shadow-sm focus:border-brand-500 focus:outline-none focus:ring-2 focus:ring-brand-500/20 sm:w-72"
            />
          </div>
          <select
            v-model="statusFilter"
            class="rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-medium text-slate-700 shadow-sm focus:border-brand-500 focus:outline-none focus:ring-2 focus:ring-brand-500/20"
          >
            <option value="all">All statuses</option>
            <option value="success">Success</option>
            <option value="failed">Failed</option>
            <option value="running">Running</option>
            <option value="waiting">Waiting</option>
            <option value="stopped">Stopped</option>
          </select>
        </div>
      </div>

      <div
        v-if="error"
        class="rounded-3xl border border-red-200 bg-red-50 px-5 py-4 text-sm font-medium text-red-700"
      >
        {{ error }}
      </div>

      <div class="grid gap-6 lg:grid-cols-[360px,minmax(0,1fr)]">
        <section class="overflow-hidden rounded-[28px] border border-slate-200 bg-white shadow-sm">
          <div class="border-b border-slate-100 px-5 py-4">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-sm font-semibold text-slate-800">Runs</h2>
                <p class="mt-1 text-xs text-slate-500">{{ filteredExecutions.length }} visible</p>
              </div>
              <Activity class="h-4 w-4 text-slate-400" />
            </div>
          </div>

          <div
            v-if="loading"
            class="flex items-center gap-3 px-5 py-6 text-sm text-slate-500"
          >
            <Loader2 class="h-4 w-4 animate-spin" />
            Loading executions...
          </div>

          <div
            v-else-if="filteredExecutions.length === 0"
            class="px-5 py-8 text-sm text-slate-500"
          >
            No executions found.
          </div>

          <ul v-else class="max-h-[70vh] divide-y divide-slate-100 overflow-auto">
            <li
              v-for="execution in filteredExecutions"
              :key="execution.id"
              class="cursor-pointer px-5 py-4 transition-colors hover:bg-slate-50"
              :class="selectedExecutionId === execution.id ? 'bg-brand-50/60' : ''"
              @click="openExecutionDetails(execution)"
            >
              <div class="flex items-start justify-between gap-4">
                <div class="min-w-0 space-y-2">
                  <div class="flex flex-wrap items-center gap-2">
                    <p class="text-sm font-semibold text-slate-800">
                      Workflow {{ execution.workflowId.slice(0, 8) }}
                    </p>
                    <ExecutionStatusBadge :status="execution.status" />
                  </div>
                  <p class="font-mono text-[11px] text-slate-400">#{{ execution.id }}</p>
                  <div class="flex flex-wrap items-center gap-3 text-xs text-slate-500">
                    <span class="inline-flex items-center gap-1">
                      <Clock class="h-3.5 w-3.5" />
                      {{ formatDuration(execution) }}
                    </span>
                    <span>{{ formatRelativeTime(execution.startedAt) }}</span>
                  </div>
                </div>

                <div class="flex items-center gap-1">
                  <button
                    class="rounded-xl border border-slate-200 p-2 text-slate-500 hover:border-brand-200 hover:text-brand-600 disabled:opacity-60"
                    :disabled="loading || actionLoading"
                    title="Retry execution"
                    @click.stop="retryExecution(execution.id)"
                  >
                    <RotateCcw class="h-4 w-4" />
                  </button>
                  <button
                    v-if="execution.status === 'running'"
                    class="rounded-xl border border-slate-200 p-2 text-slate-500 hover:border-amber-200 hover:text-amber-700 disabled:opacity-60"
                    :disabled="loading || actionLoading"
                    title="Stop execution"
                    @click.stop="stopExecution(execution.id)"
                  >
                    <Square class="h-4 w-4" />
                  </button>
                  <button
                    class="rounded-xl border border-slate-200 p-2 text-slate-500 hover:border-red-200 hover:text-red-600 disabled:opacity-60"
                    :disabled="loading || actionLoading"
                    title="Delete execution"
                    @click.stop="deleteExecution(execution.id)"
                  >
                    <Trash2 class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </li>
          </ul>
        </section>

        <section class="overflow-hidden rounded-[28px] border border-slate-200 bg-white shadow-sm">
          <div
            v-if="!selectedExecution"
            class="flex h-full min-h-[420px] items-center justify-center px-8 py-10 text-center"
          >
            <div class="max-w-md">
              <p class="text-xs font-semibold uppercase tracking-[0.22em] text-slate-400">
                Execution Inspector
              </p>
              <h2 class="mt-3 text-2xl font-semibold text-slate-900">Choose a run to inspect</h2>
              <p class="mt-2 text-sm text-slate-500">
                Select an execution from the left to load node results, lifecycle events, wait
                state metadata, and the persisted payload.
              </p>
            </div>
          </div>

          <div v-else class="space-y-6 p-5 md:p-6">
            <div class="flex flex-wrap items-start justify-between gap-4 border-b border-slate-100 pb-5">
              <div>
                <div class="flex flex-wrap items-center gap-3">
                  <h2 class="text-xl font-semibold text-slate-900">
                    Execution #{{ selectedExecution.id.slice(0, 8) }}
                  </h2>
                  <ExecutionStatusBadge :status="selectedExecution.status" />
                </div>
                <p class="mt-2 font-mono text-xs text-slate-400">{{ selectedExecution.id }}</p>
                <p class="mt-1 text-sm text-slate-500">
                  Workflow {{ selectedExecution.workflowId }}
                </p>
              </div>

              <div class="flex items-center gap-2">
                <button
                  class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 disabled:opacity-60"
                  :disabled="detailsLoading || actionLoading"
                  @click="openExecutionDetails(selectedExecution)"
                >
                  <RefreshCw class="h-4 w-4" />
                  Refresh detail
                </button>
                <button
                  class="rounded-2xl border border-slate-200 p-2 text-slate-500 hover:bg-slate-50 hover:text-slate-700"
                  @click="closeExecutionDetails"
                >
                  <X class="h-4 w-4" />
                </button>
              </div>
            </div>

            <div
              v-if="detailsLoading"
              class="flex items-center gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4 text-sm text-slate-500"
            >
              <Loader2 class="h-4 w-4 animate-spin" />
              Loading execution detail...
            </div>

            <template v-else>
              <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <div class="rounded-3xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">
                    Status
                  </p>
                  <div class="mt-3">
                    <ExecutionStatusBadge :status="selectedExecution.status" />
                  </div>
                </div>

                <div class="rounded-3xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">
                    Duration
                  </p>
                  <p class="mt-3 text-xl font-semibold text-slate-800">
                    {{ formatDuration(selectedExecution) }}
                  </p>
                </div>

                <div class="rounded-3xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">
                    Started
                  </p>
                  <p class="mt-3 text-sm font-medium text-slate-700">
                    {{ formatTimestamp(selectedExecution.startedAt) }}
                  </p>
                </div>

                <div class="rounded-3xl border border-slate-200 bg-slate-50 px-4 py-4">
                  <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">
                    Events
                  </p>
                  <p class="mt-3 text-xl font-semibold text-slate-800">
                    {{ selectedMeta?.eventCount || selectedEvents.length }}
                  </p>
                </div>
              </div>

              <div
                v-if="waitDetails"
                class="rounded-[28px] border border-amber-200 bg-amber-50/70 px-5 py-5"
              >
                <div class="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <p class="text-xs font-semibold uppercase tracking-[0.2em] text-amber-700">
                      Wait State
                    </p>
                    <h3 class="mt-2 text-lg font-semibold text-amber-950">
                      Waiting at {{ waitDetails.nodeName || 'unknown node' }}
                    </h3>
                  </div>
                  <ExecutionStatusBadge :status="'waiting'" />
                </div>

                <div class="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                  <div class="rounded-2xl border border-amber-200 bg-white/80 px-4 py-3">
                    <p class="text-[11px] uppercase tracking-[0.16em] text-amber-700">Type</p>
                    <p class="mt-1 text-sm font-medium text-slate-800">
                      {{ waitDetails.waitType || 'unknown' }}
                    </p>
                  </div>
                  <div class="rounded-2xl border border-amber-200 bg-white/80 px-4 py-3">
                    <p class="text-[11px] uppercase tracking-[0.16em] text-amber-700">Duration</p>
                    <p class="mt-1 text-sm font-medium text-slate-800">
                      {{ waitDetails.durationMs ? `${waitDetails.durationMs}ms` : 'n/a' }}
                    </p>
                  </div>
                  <div class="rounded-2xl border border-amber-200 bg-white/80 px-4 py-3 md:col-span-2">
                    <div class="flex items-start justify-between gap-3">
                      <div class="min-w-0">
                        <p class="text-[11px] uppercase tracking-[0.16em] text-amber-700">
                          Resume URL
                        </p>
                        <p class="mt-1 break-all text-sm font-medium text-slate-800">
                          {{ waitDetails.resumeUrl || 'n/a' }}
                        </p>
                        <p v-if="waitDetails.expiresAt" class="mt-2 text-xs text-slate-500">
                          Expires {{ formatTimestamp(waitDetails.expiresAt) }}
                        </p>
                      </div>
                      <button
                        v-if="waitDetails.resumeUrl"
                        class="inline-flex items-center gap-1 rounded-xl border border-amber-200 bg-white px-3 py-2 text-xs font-semibold text-amber-800 hover:bg-amber-100"
                        @click="copyResumeUrl"
                      >
                        <Copy class="h-3.5 w-3.5" />
                        {{ copiedResumeUrl ? 'Copied' : 'Copy' }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div v-if="nodeResults.length > 0" class="space-y-3">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-semibold text-slate-800">Node Results</h3>
                  <p class="text-xs text-slate-400">{{ nodeResults.length }} nodes</p>
                </div>
                <div class="grid gap-3 md:grid-cols-2">
                  <div
                    v-for="nodeResult in nodeResults"
                    :key="nodeResult.nodeName"
                    class="rounded-[24px] border px-4 py-4"
                    :class="
                      nodeResult.success
                        ? 'border-green-200 bg-green-50/60'
                        : 'border-red-200 bg-red-50/60'
                    "
                  >
                    <div class="flex items-center justify-between gap-3">
                      <p class="text-sm font-semibold text-slate-900">{{ nodeResult.nodeName }}</p>
                      <ExecutionStatusBadge :status="nodeResult.success ? 'success' : 'failed'" />
                    </div>
                    <p class="mt-2 text-xs text-slate-600">
                      {{
                        nodeResult.success
                          ? `Outputs: ${nodeResult.outputsCount}`
                          : nodeResult.error || 'Node execution failed'
                      }}
                    </p>
                  </div>
                </div>
              </div>

              <div class="space-y-3">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-semibold text-slate-800">Event Timeline</h3>
                  <p class="text-xs text-slate-400">
                    {{ selectedEvents.length }} event{{ selectedEvents.length === 1 ? '' : 's' }}
                  </p>
                </div>
                <ExecutionTimeline
                  :events="selectedEvents"
                  empty-message="This execution has no lifecycle timeline yet."
                />
              </div>

              <div class="space-y-3">
                <h3 class="text-sm font-semibold text-slate-800">Raw Execution Data</h3>
                <pre class="max-h-[380px] overflow-auto rounded-[28px] bg-slate-950 p-4 text-xs text-slate-100">{{ executionJson }}</pre>
              </div>
            </template>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
