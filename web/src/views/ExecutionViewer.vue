<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Clock, CheckCircle2, XCircle, Loader2, Search, RefreshCw, RotateCcw, Square, Trash2, X } from 'lucide-vue-next'
import api from '../api'

interface ExecutionEntity {
  id: string
  workflow_id: string
  status: string
  data: any
  started_at: string
  stopped_at?: string | null
}

const executions = ref<ExecutionEntity[]>([])
const selectedExecution = ref<ExecutionEntity | null>(null)
const loading = ref(false)
const actionLoading = ref(false)
const error = ref<string | null>(null)
const query = ref('')
const statusFilter = ref('all')

const filteredExecutions = computed(() => {
  const q = query.value.trim().toLowerCase()
  return executions.value.filter((exec) => {
    const matchesStatus =
      statusFilter.value === 'all' ||
      exec.status.toLowerCase() === statusFilter.value.toLowerCase()

    const matchesQuery =
      q.length === 0 ||
      exec.id.toLowerCase().includes(q) ||
      exec.workflow_id.toLowerCase().includes(q)

    return matchesStatus && matchesQuery
  })
})

const nodeResults = computed(() => {
  const data = selectedExecution.value?.data
  if (!data || typeof data !== 'object' || Array.isArray(data)) return []

  return Object.entries(data)
    .filter(([_, value]) => value && typeof value === 'object')
    .map(([nodeName, value]: [string, any]) => ({
      nodeName,
      success: value.success,
      error: value.error,
      outputsCount: Array.isArray(value.outputs) ? value.outputs.length : 0,
    }))
})

const executionJson = computed(() => {
  if (!selectedExecution.value) return ''
  return JSON.stringify(selectedExecution.value.data, null, 2)
})

function formatDuration(exec: ExecutionEntity): string {
  if (!exec.stopped_at) return 'Running'
  const start = new Date(exec.started_at).getTime()
  const end = new Date(exec.stopped_at).getTime()
  const ms = Math.max(0, end - start)

  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

function formatRelativeTime(iso: string): string {
  const date = new Date(iso).getTime()
  const diffSeconds = Math.floor((date - Date.now()) / 1000)
  const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })

  const abs = Math.abs(diffSeconds)
  if (abs < 60) return rtf.format(diffSeconds, 'second')

  const diffMinutes = Math.floor(diffSeconds / 60)
  if (Math.abs(diffMinutes) < 60) return rtf.format(diffMinutes, 'minute')

  const diffHours = Math.floor(diffMinutes / 60)
  if (Math.abs(diffHours) < 24) return rtf.format(diffHours, 'hour')

  const diffDays = Math.floor(diffHours / 24)
  return rtf.format(diffDays, 'day')
}

function openExecutionDetails(exec: ExecutionEntity) {
  selectedExecution.value = exec
}

function closeExecutionDetails() {
  selectedExecution.value = null
}

async function fetchExecutions() {
  loading.value = true
  error.value = null
  try {
    const res = await api.get('/executions', { params: { limit: 100 } })
    executions.value = res.data

    if (selectedExecution.value) {
      selectedExecution.value =
        res.data.find((e: ExecutionEntity) => e.id === selectedExecution.value?.id) || null
    }
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.message || 'Failed to fetch executions'
  } finally {
    loading.value = false
  }
}

async function retryExecution(id: string) {
  actionLoading.value = true
  error.value = null
  try {
    await api.post(`/executions/${id}/retry`)
    await fetchExecutions()
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
    await api.post(`/executions/${id}/stop`)
    await fetchExecutions()
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
    await api.delete(`/executions/${id}`)
    executions.value = executions.value.filter((exec) => exec.id !== id)
    if (selectedExecution.value?.id === id) {
      selectedExecution.value = null
    }
  } catch (err: any) {
    error.value = err?.response?.data || err?.message || 'Failed to delete execution'
  } finally {
    actionLoading.value = false
  }
}

onMounted(fetchExecutions)
</script>

<template>
  <div class="h-full bg-slate-50 overflow-auto p-4 md:p-8">
    <div class="max-w-6xl mx-auto">
      <div class="flex flex-col md:flex-row md:items-center justify-between mb-8 gap-4">
        <div>
          <h1 class="text-2xl font-bold text-slate-900">Execution History</h1>
          <p class="text-slate-500 text-sm mt-1">
            Review recent workflow runs and troubleshoot failures.
          </p>
        </div>

        <div class="flex items-center gap-3">
          <button
            @click="fetchExecutions"
            :disabled="loading || actionLoading"
            class="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg border border-slate-200 bg-white text-slate-600 text-sm font-medium hover:bg-slate-50 disabled:opacity-60"
          >
            <RefreshCw class="w-4 h-4" />
            Refresh
          </button>
          <div class="relative">
            <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
            <input
              v-model="query"
              type="text"
              placeholder="Search by execution or workflow id..."
              class="pl-9 pr-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-transparent w-full md:w-72"
            />
          </div>
          <select
            v-model="statusFilter"
            class="bg-white border border-slate-200 text-slate-600 px-3 py-2 rounded-lg text-sm font-medium"
          >
            <option value="all">All</option>
            <option value="success">Success</option>
            <option value="failed">Failed</option>
            <option value="running">Running</option>
          </select>
        </div>
      </div>

      <div
        v-if="loading"
        class="bg-white rounded-xl shadow-sm border border-slate-200 p-8 flex items-center gap-3 text-slate-500"
      >
        <Loader2 class="w-4 h-4 animate-spin" />
        Loading executions...
      </div>

      <div
        v-else-if="error"
        class="bg-red-50 border border-red-200 text-red-700 rounded-xl p-4 text-sm font-medium"
      >
        {{ error }}
      </div>

      <div
        v-else-if="filteredExecutions.length === 0"
        class="bg-white rounded-xl shadow-sm border border-slate-200 p-8 text-slate-500 text-sm"
      >
        No executions found.
      </div>

      <div v-else class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
        <ul class="divide-y divide-slate-100">
          <li
            v-for="exec in filteredExecutions"
            :key="exec.id"
            class="p-5 hover:bg-slate-50 transition-colors cursor-pointer"
            @click="openExecutionDetails(exec)"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-4">
                <div
                  :class="[
                    'w-10 h-10 rounded-full flex items-center justify-center shrink-0',
                    exec.status === 'success'
                      ? 'bg-green-100 text-green-600'
                      : exec.status === 'running'
                        ? 'bg-amber-100 text-amber-700'
                        : 'bg-red-100 text-red-600',
                  ]"
                >
                  <Clock v-if="exec.status === 'running'" class="w-5 h-5" />
                  <CheckCircle2 v-else-if="exec.status === 'success'" class="w-5 h-5" />
                  <XCircle v-else class="w-5 h-5" />
                </div>

                <div>
                  <h3 class="font-semibold text-slate-800 text-base">Workflow {{ exec.workflow_id.slice(0, 8) }}</h3>
                  <div class="flex items-center text-xs text-slate-500 mt-1 gap-3">
                    <span class="font-mono text-slate-400">#{{ exec.id.slice(0, 8) }}</span>
                    <span class="flex items-center gap-1"><Clock class="w-3 h-3" /> {{ formatDuration(exec) }}</span>
                  </div>
                </div>
              </div>

              <div class="text-right">
                <span class="text-sm text-slate-500 block">{{ formatRelativeTime(exec.started_at) }}</span>
                <span
                  :class="[
                    'inline-block mt-1 text-xs font-semibold px-2 py-0.5 rounded-full',
                    exec.status === 'success'
                      ? 'bg-green-50 text-green-700'
                      : exec.status === 'running'
                        ? 'bg-amber-50 text-amber-700'
                        : 'bg-red-50 text-red-700',
                  ]"
                >
                  {{ exec.status }}
                </span>
                <div class="mt-2 flex items-center justify-end gap-1">
                  <button
                    @click.stop="retryExecution(exec.id)"
                    :disabled="loading || actionLoading"
                    class="p-1.5 rounded-md border border-slate-200 text-slate-500 hover:text-brand-600 hover:border-brand-200 disabled:opacity-60"
                    title="Retry execution"
                  >
                    <RotateCcw class="w-4 h-4" />
                  </button>
                  <button
                    v-if="exec.status === 'running'"
                    @click.stop="stopExecution(exec.id)"
                    :disabled="loading || actionLoading"
                    class="p-1.5 rounded-md border border-slate-200 text-slate-500 hover:text-amber-700 hover:border-amber-200 disabled:opacity-60"
                    title="Stop execution"
                  >
                    <Square class="w-4 h-4" />
                  </button>
                  <button
                    @click.stop="deleteExecution(exec.id)"
                    :disabled="loading || actionLoading"
                    class="p-1.5 rounded-md border border-slate-200 text-slate-500 hover:text-red-600 hover:border-red-200 disabled:opacity-60"
                    title="Delete execution"
                  >
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </li>
        </ul>
      </div>

      <div
        v-if="selectedExecution"
        class="mt-6 bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden"
      >
        <div class="px-5 py-4 border-b border-slate-100 flex items-center justify-between">
          <div>
            <h2 class="font-semibold text-slate-900">Execution #{{ selectedExecution.id.slice(0, 8) }}</h2>
            <p class="text-xs text-slate-500 mt-1">Workflow {{ selectedExecution.workflow_id }}</p>
          </div>
          <button
            @click="closeExecutionDetails"
            class="p-2 rounded-lg border border-slate-200 text-slate-500 hover:text-slate-700 hover:bg-slate-50"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="p-5 space-y-4">
          <div v-if="nodeResults.length > 0" class="space-y-2">
            <h3 class="text-sm font-semibold text-slate-700">Node results</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div
                v-for="result in nodeResults"
                :key="result.nodeName"
                class="rounded-lg border p-3"
                :class="result.success ? 'border-green-200 bg-green-50/50' : 'border-red-200 bg-red-50/50'"
              >
                <p class="font-medium text-sm text-slate-900">{{ result.nodeName }}</p>
                <p class="text-xs mt-1" :class="result.success ? 'text-green-700' : 'text-red-700'">
                  {{ result.success ? `Success (outputs: ${result.outputsCount})` : (result.error || 'Failed') }}
                </p>
              </div>
            </div>
          </div>

          <div>
            <h3 class="text-sm font-semibold text-slate-700 mb-2">Raw execution data</h3>
            <pre class="bg-slate-900 text-slate-100 text-xs rounded-lg p-4 overflow-auto max-h-80">{{ executionJson }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
