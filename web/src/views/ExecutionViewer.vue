<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Clock, CheckCircle2, XCircle, Loader2, Search } from 'lucide-vue-next'
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
const loading = ref(false)
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

async function fetchExecutions() {
  loading.value = true
  error.value = null
  try {
    const res = await api.get('/executions', { params: { limit: 100 } })
    executions.value = res.data
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.message || 'Failed to fetch executions'
  } finally {
    loading.value = false
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
            class="p-5 hover:bg-slate-50 transition-colors"
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
              </div>
            </div>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
