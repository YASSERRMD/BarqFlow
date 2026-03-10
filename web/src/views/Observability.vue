<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Clock3,
  Flame,
  KeyRound,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
  Workflow,
} from 'lucide-vue-next'
import { getObservabilityOverview } from '../features/observability/api'
import type {
  CredentialHealthRecord,
  ExecutionFlamegraph,
  ExecutionFlamegraphSpan,
  FailureCluster,
  NodeLatencyHistogram,
  ObservabilityOverview,
  WorkflowBottleneck,
} from '../types/contracts'

const overview = ref<ObservabilityOverview | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const selectedWindow = ref(72)
const selectedFlamegraphId = ref<string | null>(null)

const windowOptions = [
  { label: '24 hours', value: 24 },
  { label: '72 hours', value: 72 },
  { label: '7 days', value: 168 },
]

const selectedFlamegraph = computed(() => {
  if (!overview.value || !selectedFlamegraphId.value) return null
  return (
    overview.value.executionFlamegraphs.find((sample) => sample.executionId === selectedFlamegraphId.value) || null
  )
})

const criticalCredentials = computed(() => {
  return overview.value?.credentialHealth.filter((credential) => credential.health === 'critical').length || 0
})

const warningCredentials = computed(() => {
  return overview.value?.credentialHealth.filter((credential) => credential.health === 'warning').length || 0
})

const hasData = computed(() => !!overview.value)

watch(overview, (nextOverview) => {
  const samples = nextOverview?.executionFlamegraphs || []
  if (samples.length === 0) {
    selectedFlamegraphId.value = null
    return
  }

  const stillExists = samples.some((sample) => sample.executionId === selectedFlamegraphId.value)
  if (!stillExists) {
    selectedFlamegraphId.value = samples[0].executionId
  }
})

function metricCards() {
  if (!overview.value) return []

  return [
    {
      title: 'Windowed executions',
      value: overview.value.executionCount.toLocaleString(),
      detail: `${overview.value.workflowCount} workflows in scope`,
      icon: Activity,
      tone: 'sky',
    },
    {
      title: 'Success rate',
      value: `${overview.value.successRate.toFixed(1)}%`,
      detail: `${overview.value.successfulExecutionCount}/${overview.value.terminalExecutionCount} terminal runs`,
      icon: CheckCircle2,
      tone: 'emerald',
    },
    {
      title: 'Failure rate',
      value: `${overview.value.failureRate.toFixed(1)}%`,
      detail: `${overview.value.failedExecutionCount} failed runs`,
      icon: AlertTriangle,
      tone: 'rose',
    },
    {
      title: 'Average runtime',
      value: formatDuration(overview.value.averageExecutionDurationMs),
      detail: `${overview.value.runningExecutionCount} running, ${overview.value.waitingExecutionCount} waiting`,
      icon: Clock3,
      tone: 'amber',
    },
    {
      title: 'Credential posture',
      value: `${criticalCredentials.value + warningCredentials.value}`,
      detail: `${criticalCredentials.value} critical, ${warningCredentials.value} warning`,
      icon: KeyRound,
      tone: 'violet',
    },
  ]
}

function toneClasses(tone: string) {
  switch (tone) {
    case 'emerald':
      return 'bg-emerald-50 text-emerald-700 ring-emerald-200'
    case 'rose':
      return 'bg-rose-50 text-rose-700 ring-rose-200'
    case 'amber':
      return 'bg-amber-50 text-amber-700 ring-amber-200'
    case 'violet':
      return 'bg-violet-50 text-violet-700 ring-violet-200'
    default:
      return 'bg-sky-50 text-sky-700 ring-sky-200'
  }
}

function statusBadgeClasses(status: string) {
  const normalized = status.toLowerCase()
  if (normalized === 'success' || normalized === 'healthy') {
    return 'bg-emerald-50 text-emerald-700 ring-emerald-200'
  }
  if (normalized === 'failed' || normalized === 'critical' || normalized === 'error' || normalized === 'invalid') {
    return 'bg-rose-50 text-rose-700 ring-rose-200'
  }
  if (normalized === 'warning' || normalized === 'waiting') {
    return 'bg-amber-50 text-amber-700 ring-amber-200'
  }
  if (normalized === 'running' || normalized === 'queued') {
    return 'bg-sky-50 text-sky-700 ring-sky-200'
  }
  if (normalized === 'idle' || normalized === 'stopped' || normalized === 'cancelled') {
    return 'bg-slate-100 text-slate-700 ring-slate-200'
  }
  return 'bg-slate-100 text-slate-700 ring-slate-200'
}

function flamegraphBarClasses(status: string) {
  const normalized = status.toLowerCase()
  if (normalized === 'success') return 'bg-emerald-500'
  if (normalized === 'failed') return 'bg-rose-500'
  if (normalized === 'skipped') return 'bg-slate-400'
  return 'bg-sky-500'
}

function flamegraphBarStyle(span: ExecutionFlamegraphSpan, sample: ExecutionFlamegraph) {
  const total = Math.max(sample.totalDurationMs, 1)
  const left = (span.offsetMs / total) * 100
  const width = Math.max((span.durationMs / total) * 100, 2)
  return {
    left: `${Math.min(left, 98)}%`,
    width: `${Math.min(width, 100 - Math.min(left, 98))}%`,
  }
}

function histogramBarWidth(bucketCount: number, sampleCount: number) {
  if (!sampleCount) return '0%'
  return `${Math.max((bucketCount / sampleCount) * 100, bucketCount > 0 ? 6 : 0)}%`
}

function formatDuration(durationMs: number) {
  if (durationMs < 1000) return `${durationMs}ms`
  if (durationMs < 60_000) return `${(durationMs / 1000).toFixed(1)}s`
  return `${(durationMs / 60_000).toFixed(1)}m`
}

function formatTimestamp(value?: string | null) {
  if (!value) return 'Unknown'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

function formatRelative(value?: string | null) {
  if (!value) return 'No timestamp'
  const date = new Date(value).getTime()
  if (Number.isNaN(date)) return value

  const diffSeconds = Math.floor((date - Date.now()) / 1000)
  const formatter = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })
  const absSeconds = Math.abs(diffSeconds)

  if (absSeconds < 60) return formatter.format(diffSeconds, 'second')

  const minutes = Math.floor(diffSeconds / 60)
  if (Math.abs(minutes) < 60) return formatter.format(minutes, 'minute')

  const hours = Math.floor(minutes / 60)
  if (Math.abs(hours) < 24) return formatter.format(hours, 'hour')

  const days = Math.floor(hours / 24)
  return formatter.format(days, 'day')
}

function emptyIssues(credential: CredentialHealthRecord) {
  return credential.issues.length ? credential.issues : ['No active issues detected']
}

function asyncLabel(cluster: FailureCluster) {
  return [cluster.workflowName, cluster.nodeName].filter(Boolean).join(' / ') || 'Workflow-level failure'
}

function bottleneckLabel(item: WorkflowBottleneck) {
  return `${item.workflowName} / ${item.nodeName}`
}

function latencyLabel(item: NodeLatencyHistogram) {
  return `${item.workflowName} / ${item.nodeName}`
}

async function loadOverview() {
  loading.value = true
  error.value = null

  try {
    const response = await getObservabilityOverview(selectedWindow.value)
    overview.value = response.data
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to load observability overview'
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await loadOverview()
})
</script>

<template>
  <div class="h-full overflow-y-auto bg-slate-50">
    <div class="mx-auto flex max-w-[1600px] flex-col gap-6 px-4 py-6 md:px-6 lg:px-8">
      <section class="rounded-[28px] border border-slate-200 bg-white p-5 shadow-sm sm:p-6">
        <div class="flex flex-col gap-5 xl:flex-row xl:items-start xl:justify-between">
          <div class="max-w-3xl">
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Workspace analytics</p>
            <h2 class="mt-2 text-3xl font-display font-bold tracking-tight text-slate-950">Observability Command Center</h2>
            <p class="mt-3 text-sm leading-6 text-slate-600 sm:text-base">
              Track node latency, workflow bottlenecks, clustered failures, credential posture, and sampled execution timelines from one operational surface.
            </p>
          </div>

          <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
            <label class="flex items-center gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm font-semibold text-slate-700">
              Window
              <select
                v-model.number="selectedWindow"
                class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-900 outline-none transition focus:border-sky-400"
                @change="loadOverview"
              >
                <option v-for="option in windowOptions" :key="option.value" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>

            <button
              class="inline-flex items-center justify-center gap-2 rounded-2xl border border-slate-200 bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="loading"
              @click="loadOverview"
            >
              <RefreshCw class="h-4 w-4" :class="loading ? 'animate-spin' : ''" />
              Refresh telemetry
            </button>
          </div>
        </div>

        <div class="mt-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
            <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-500">Generated</p>
            <p class="mt-2 text-sm font-semibold text-slate-900">{{ overview ? formatTimestamp(overview.generatedAt) : 'Loading...' }}</p>
          </div>
          <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
            <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-500">Queued / running</p>
            <p class="mt-2 text-sm font-semibold text-slate-900">
              {{ overview ? `${overview.queuedExecutionCount} queued / ${overview.runningExecutionCount} running` : 'Loading...' }}
            </p>
          </div>
          <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
            <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-500">Waiting executions</p>
            <p class="mt-2 text-sm font-semibold text-slate-900">{{ overview ? overview.waitingExecutionCount : 'Loading...' }}</p>
          </div>
          <div class="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4">
            <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-500">Failure clusters</p>
            <p class="mt-2 text-sm font-semibold text-slate-900">{{ overview ? overview.failureClusters.length : 'Loading...' }}</p>
          </div>
        </div>
      </section>

      <div
        v-if="error"
        class="rounded-2xl border border-rose-200 bg-rose-50 px-4 py-4 text-sm font-medium text-rose-700"
      >
        {{ error }}
      </div>

      <section v-if="hasData" class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <article
          v-for="card in metricCards()"
          :key="card.title"
          class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm"
        >
          <div class="flex items-start justify-between gap-4">
            <div>
              <p class="text-sm font-semibold text-slate-600">{{ card.title }}</p>
              <p class="mt-3 text-3xl font-display font-bold tracking-tight text-slate-950">{{ card.value }}</p>
            </div>
            <div :class="['flex h-11 w-11 items-center justify-center rounded-2xl ring-1', toneClasses(card.tone)]">
              <component :is="card.icon" class="h-5 w-5" />
            </div>
          </div>
          <p class="mt-4 text-sm text-slate-500">{{ card.detail }}</p>
        </article>
      </section>

      <section v-if="hasData" class="grid gap-6 xl:grid-cols-[1.3fr,0.9fr]">
        <article class="rounded-[28px] border border-slate-200 bg-white shadow-sm">
          <div class="flex items-center justify-between gap-4 border-b border-slate-200 px-5 py-4 sm:px-6">
            <div>
              <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Latency histograms</p>
              <h3 class="mt-1 text-lg font-bold text-slate-950">Slowest node paths</h3>
            </div>
            <div class="inline-flex items-center gap-2 rounded-2xl bg-slate-100 px-3 py-2 text-xs font-semibold text-slate-600">
              <Flame class="h-4 w-4 text-amber-500" />
              P95 ranked
            </div>
          </div>

          <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-slate-200 text-sm">
              <thead class="bg-slate-50 text-left text-xs font-bold uppercase tracking-[0.18em] text-slate-500">
                <tr>
                  <th class="px-5 py-3 sm:px-6">Node</th>
                  <th class="px-5 py-3">Runs</th>
                  <th class="px-5 py-3">Avg / P95</th>
                  <th class="px-5 py-3">Histogram</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100">
                <tr v-for="item in overview?.nodeLatencyHistograms" :key="`${item.workflowId}-${item.nodeName}`">
                  <td class="px-5 py-4 align-top sm:px-6">
                    <p class="font-semibold text-slate-950">{{ latencyLabel(item) }}</p>
                    <p class="mt-1 text-xs text-slate-500">{{ item.nodeType }}</p>
                  </td>
                  <td class="px-5 py-4 align-top text-slate-700">
                    <p class="font-semibold text-slate-900">{{ item.samples }}</p>
                    <p class="mt-1 text-xs text-slate-500">{{ item.failedRuns }} failed</p>
                  </td>
                  <td class="px-5 py-4 align-top text-slate-700">
                    <p class="font-semibold text-slate-900">{{ formatDuration(item.avgDurationMs) }}</p>
                    <p class="mt-1 text-xs text-slate-500">P95 {{ formatDuration(item.p95DurationMs) }} · Max {{ formatDuration(item.maxDurationMs) }}</p>
                  </td>
                  <td class="px-5 py-4 align-top">
                    <div class="flex min-w-[280px] gap-2">
                      <div v-for="bucket in item.histogram" :key="bucket.label" class="min-w-0 flex-1">
                        <div class="h-2 rounded-full bg-slate-100">
                          <div
                            class="h-2 rounded-full bg-sky-500"
                            :style="{ width: histogramBarWidth(bucket.count, item.samples) }"
                          ></div>
                        </div>
                        <p class="mt-2 truncate text-[11px] font-semibold uppercase tracking-[0.14em] text-slate-400">
                          {{ bucket.label }}
                        </p>
                        <p class="text-xs font-medium text-slate-600">{{ bucket.count }}</p>
                      </div>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>

        <article class="rounded-[28px] border border-slate-200 bg-white shadow-sm">
          <div class="border-b border-slate-200 px-5 py-4 sm:px-6">
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Workflow bottlenecks</p>
            <h3 class="mt-1 text-lg font-bold text-slate-950">Highest contribution nodes</h3>
          </div>

          <div class="space-y-4 p-5 sm:p-6">
            <article
              v-for="item in overview?.workflowBottlenecks"
              :key="`${item.workflowId}-${item.nodeName}`"
              class="rounded-2xl border border-slate-200 bg-slate-50 p-4"
            >
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="font-semibold text-slate-950">{{ bottleneckLabel(item) }}</p>
                  <p class="mt-1 text-xs uppercase tracking-[0.18em] text-slate-500">{{ item.nodeType }}</p>
                </div>
                <RouterLink
                  :to="`/workflow/${item.workflowId}`"
                  class="inline-flex items-center rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-semibold text-slate-700 transition hover:border-slate-300 hover:text-slate-950"
                >
                  Open workflow
                </RouterLink>
              </div>

              <div class="mt-4 grid gap-3 sm:grid-cols-3">
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Avg</p>
                  <p class="mt-2 text-lg font-bold text-slate-950">{{ formatDuration(item.avgDurationMs) }}</p>
                </div>
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">P95</p>
                  <p class="mt-2 text-lg font-bold text-slate-950">{{ formatDuration(item.p95DurationMs) }}</p>
                </div>
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Contribution</p>
                  <p class="mt-2 text-lg font-bold text-slate-950">{{ item.contributionRate.toFixed(1) }}%</p>
                </div>
              </div>

              <div class="mt-4 flex flex-wrap items-center gap-2 text-xs text-slate-500">
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ item.samples }} samples</span>
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ item.failureCount }} failures</span>
              </div>
            </article>
          </div>
        </article>
      </section>

      <section v-if="hasData" class="grid gap-6 xl:grid-cols-[1.15fr,0.85fr]">
        <article class="rounded-[28px] border border-slate-200 bg-white shadow-sm">
          <div class="border-b border-slate-200 px-5 py-4 sm:px-6">
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Failure clustering</p>
            <h3 class="mt-1 text-lg font-bold text-slate-950">Repeated break patterns</h3>
          </div>

          <div class="space-y-4 p-5 sm:p-6">
            <article
              v-for="cluster in overview?.failureClusters"
              :key="cluster.clusterKey"
              class="rounded-2xl border border-slate-200 bg-slate-50 p-4"
            >
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="font-semibold text-slate-950">{{ asyncLabel(cluster) }}</p>
                  <p class="mt-2 text-sm leading-6 text-slate-600">{{ cluster.message }}</p>
                </div>
                <span class="inline-flex rounded-full px-3 py-1.5 text-xs font-semibold ring-1" :class="statusBadgeClasses(cluster.level)">
                  {{ cluster.level }}
                </span>
              </div>

              <div class="mt-4 flex flex-wrap items-center gap-2 text-xs text-slate-500">
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ cluster.failureCount }} occurrences</span>
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ cluster.affectedExecutionCount }} executions</span>
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ cluster.eventType || 'event' }}</span>
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">Last seen {{ formatRelative(cluster.lastSeenAt) }}</span>
              </div>
            </article>
          </div>
        </article>

        <article class="rounded-[28px] border border-slate-200 bg-white shadow-sm">
          <div class="border-b border-slate-200 px-5 py-4 sm:px-6">
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Credential health</p>
            <h3 class="mt-1 text-lg font-bold text-slate-950">Validation and rotation posture</h3>
          </div>

          <div class="space-y-4 p-5 sm:p-6">
            <article
              v-for="credential in overview?.credentialHealth"
              :key="credential.credentialId"
              class="rounded-2xl border border-slate-200 bg-slate-50 p-4"
            >
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="font-semibold text-slate-950">{{ credential.name }}</p>
                  <p class="mt-1 text-xs uppercase tracking-[0.18em] text-slate-500">{{ credential.credentialType }}</p>
                </div>
                <span class="inline-flex rounded-full px-3 py-1.5 text-xs font-semibold capitalize ring-1" :class="statusBadgeClasses(credential.health)">
                  {{ credential.health }}
                </span>
              </div>

              <div class="mt-4 grid gap-3 sm:grid-cols-2">
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Latest validation</p>
                  <p class="mt-2 text-sm font-semibold text-slate-900">{{ credential.lastTestStatus || 'Untested' }}</p>
                  <p class="mt-1 text-xs text-slate-500">{{ formatRelative(credential.lastTestedAt) }}</p>
                </div>
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Usage</p>
                  <p class="mt-2 text-sm font-semibold text-slate-900">{{ credential.usageCount }} executions</p>
                  <p class="mt-1 text-xs text-slate-500">Last used {{ formatRelative(credential.lastUsedAt) }}</p>
                </div>
              </div>

              <div class="mt-4 space-y-2">
                <div v-for="issue in emptyIssues(credential)" :key="issue" class="flex items-start gap-2 text-sm text-slate-600">
                  <component :is="credential.health === 'healthy' ? ShieldCheck : credential.health === 'critical' ? ShieldAlert : AlertTriangle" class="mt-0.5 h-4 w-4 shrink-0" :class="credential.health === 'healthy' ? 'text-emerald-500' : credential.health === 'critical' ? 'text-rose-500' : 'text-amber-500'" />
                  <span>{{ issue }}</span>
                </div>
              </div>
            </article>
          </div>
        </article>
      </section>

      <section v-if="hasData" class="rounded-[28px] border border-slate-200 bg-white shadow-sm">
        <div class="flex flex-col gap-4 border-b border-slate-200 px-5 py-4 sm:px-6 xl:flex-row xl:items-center xl:justify-between">
          <div>
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Execution spans</p>
            <h3 class="mt-1 text-lg font-bold text-slate-950">Flamegraph-style sampled runs</h3>
          </div>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="sample in overview?.executionFlamegraphs"
              :key="sample.executionId"
              class="rounded-2xl border px-4 py-3 text-left text-sm font-semibold transition"
              :class="sample.executionId === selectedFlamegraphId ? 'border-slate-950 bg-slate-950 text-white' : 'border-slate-200 bg-white text-slate-700 hover:border-slate-300 hover:text-slate-950'"
              @click="selectedFlamegraphId = sample.executionId"
            >
              <div class="flex items-center gap-2">
                <Workflow class="h-4 w-4" />
                <span>{{ sample.workflowName }}</span>
              </div>
              <p class="mt-1 text-xs opacity-80">{{ sample.status }} · {{ formatDuration(sample.totalDurationMs) }}</p>
            </button>
          </div>
        </div>

        <div v-if="selectedFlamegraph" class="space-y-5 p-5 sm:p-6">
          <div class="grid gap-4 lg:grid-cols-[0.9fr,1.1fr]">
            <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-sm font-semibold text-slate-600">Selected execution</p>
                  <h4 class="mt-2 text-xl font-bold text-slate-950">{{ selectedFlamegraph.workflowName }}</h4>
                </div>
                <span class="inline-flex rounded-full px-3 py-1.5 text-xs font-semibold capitalize ring-1" :class="statusBadgeClasses(selectedFlamegraph.status)">
                  {{ selectedFlamegraph.status }}
                </span>
              </div>

              <div class="mt-4 grid gap-3 sm:grid-cols-2">
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Duration</p>
                  <p class="mt-2 text-lg font-bold text-slate-950">{{ formatDuration(selectedFlamegraph.totalDurationMs) }}</p>
                </div>
                <div class="rounded-2xl bg-white px-3 py-3 ring-1 ring-slate-200">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Started</p>
                  <p class="mt-2 text-sm font-semibold text-slate-900">{{ formatTimestamp(selectedFlamegraph.startedAt) }}</p>
                </div>
              </div>

              <div class="mt-4 flex flex-wrap items-center gap-2 text-xs text-slate-500">
                <span class="rounded-full bg-white px-3 py-1.5 ring-1 ring-slate-200">{{ selectedFlamegraph.spans.length }} spans</span>
                <RouterLink :to="`/executions`" class="rounded-full bg-white px-3 py-1.5 font-semibold text-slate-700 ring-1 ring-slate-200 transition hover:text-slate-950">
                  Open execution monitor
                </RouterLink>
              </div>
            </div>

            <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4">
              <div class="flex items-center gap-2 text-sm font-semibold text-slate-700">
                <Activity class="h-4 w-4 text-sky-500" />
                Timeline lanes
              </div>
              <div class="mt-4 space-y-4">
                <article
                  v-for="span in selectedFlamegraph.spans"
                  :key="`${selectedFlamegraph.executionId}-${span.nodeName}-${span.offsetMs}`"
                  class="rounded-2xl bg-white p-4 ring-1 ring-slate-200"
                >
                  <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                    <div>
                      <p class="font-semibold text-slate-950">{{ span.nodeName }}</p>
                      <p class="mt-1 text-xs uppercase tracking-[0.18em] text-slate-500">{{ span.nodeType }}</p>
                    </div>
                    <div class="flex flex-wrap items-center gap-2 text-xs">
                      <span class="rounded-full px-3 py-1.5 font-semibold capitalize ring-1" :class="statusBadgeClasses(span.status)">
                        {{ span.status }}
                      </span>
                      <span class="rounded-full bg-slate-100 px-3 py-1.5 font-semibold text-slate-700 ring-1 ring-slate-200">
                        {{ formatDuration(span.durationMs) }}
                      </span>
                    </div>
                  </div>

                  <div class="relative mt-4 h-3 rounded-full bg-slate-100">
                    <div
                      class="absolute top-0 h-3 rounded-full"
                      :class="flamegraphBarClasses(span.status)"
                      :style="flamegraphBarStyle(span, selectedFlamegraph)"
                    ></div>
                  </div>

                  <div class="mt-3 grid gap-2 text-xs text-slate-500 sm:grid-cols-3">
                    <p>Offset {{ formatDuration(span.offsetMs) }}</p>
                    <p>{{ span.inputItems }} input / {{ span.outputItems }} output</p>
                    <p>{{ formatTimestamp(span.finishedAt) }}</p>
                  </div>
                </article>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section v-if="!loading && !hasData && !error" class="rounded-[28px] border border-dashed border-slate-300 bg-white p-8 text-center shadow-sm">
        <p class="text-lg font-semibold text-slate-950">No observability data is available yet.</p>
        <p class="mt-2 text-sm text-slate-600">Run workflows in the active workspace, then refresh this surface to populate latency, failure, and credential insights.</p>
      </section>
    </div>
  </div>
</template>
