<script setup lang="ts">
import { computed } from 'vue'
import type { ExecutionEvent } from '../../../types/contracts'
import ExecutionStatusBadge from './ExecutionStatusBadge.vue'

const props = defineProps<{
  events: ExecutionEvent[]
  compact?: boolean
  limit?: number
  emptyMessage?: string
}>()

const visibleEvents = computed(() => {
  const ordered = [...props.events].sort((left, right) => right.sequence - left.sequence)
  if (!props.limit || props.limit <= 0) return ordered
  return ordered.slice(0, props.limit)
})

function formatTimestamp(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

function eventAccent(status: string) {
  switch (status.toLowerCase()) {
    case 'success':
      return 'bg-green-500'
    case 'running':
      return 'bg-blue-500'
    case 'waiting':
      return 'bg-amber-500'
    case 'queued':
      return 'bg-slate-400'
    case 'stopped':
    case 'cancelled':
      return 'bg-slate-600'
    default:
      return 'bg-red-500'
  }
}

function summarizeData(event: ExecutionEvent) {
  const outputItems = event.data?.outputItems
  const inputItems = event.data?.inputItems
  const error = event.data?.error
  const waitType = event.data?.waitType

  if (typeof error === 'string' && error.trim().length > 0) return error
  if (typeof outputItems === 'number') return `${outputItems} output item(s)`
  if (typeof inputItems === 'number') return `${inputItems} input item(s)`
  if (typeof waitType === 'string') return `Wait type: ${waitType}`
  return null
}
</script>

<template>
  <div class="space-y-3">
    <div
      v-if="visibleEvents.length === 0"
      class="rounded-2xl border border-dashed border-slate-300 bg-slate-50 px-4 py-6 text-sm text-slate-500"
    >
      {{ props.emptyMessage || 'No execution events captured yet.' }}
    </div>

    <div
      v-for="event in visibleEvents"
      :key="`${event.executionId}-${event.sequence}`"
      class="rounded-2xl border border-slate-200 bg-white px-4 py-3 shadow-sm"
      :class="props.compact ? 'py-2' : ''"
    >
      <div class="flex items-start gap-3">
        <div class="mt-1 h-2.5 w-2.5 rounded-full" :class="eventAccent(event.status)"></div>
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2">
            <p class="text-sm font-semibold text-slate-800">{{ event.message }}</p>
            <ExecutionStatusBadge :status="event.status" />
            <span class="text-[11px] font-medium uppercase tracking-wide text-slate-400">
              {{ event.eventType }}
            </span>
          </div>
          <p v-if="event.nodeName" class="mt-1 text-xs text-slate-500">
            Node: {{ event.nodeName }}
          </p>
          <p v-if="summarizeData(event)" class="mt-1 text-xs text-slate-600">
            {{ summarizeData(event) }}
          </p>
          <div class="mt-2 flex flex-wrap items-center gap-3 text-[11px] text-slate-400">
            <span>#{{ event.sequence }}</span>
            <span>{{ formatTimestamp(event.timestamp) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
