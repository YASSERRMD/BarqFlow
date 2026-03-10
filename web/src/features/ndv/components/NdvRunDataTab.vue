<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  testState: { nodeId: string; status: 'running' | 'success' | 'error'; message: string } | null
  runData: any
}>()

const formattedPayload = computed(() => {
  if (!props.runData) return ''
  try {
    return JSON.stringify(props.runData, null, 2)
  } catch {
    return String(props.runData)
  }
})

function formatTimestamp(value?: string | null) {
  if (!value) return 'Unknown'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}
</script>

<template>
  <section class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
    <div>
      <h3 class="font-semibold text-slate-800">Run Data</h3>
      <p class="mt-1 text-xs text-slate-500">
        Review the latest test or execution result captured for this node.
      </p>
    </div>

    <div
      v-if="props.testState"
      :class="[
        'rounded-lg border px-3 py-2 text-xs font-medium',
        props.testState.status === 'success' && 'border-green-200 bg-green-50 text-green-700',
        props.testState.status === 'error' && 'border-red-200 bg-red-50 text-red-700',
        props.testState.status === 'running' && 'border-blue-200 bg-blue-50 text-blue-700',
      ]"
    >
      {{ props.testState.message }}
    </div>

    <div v-if="props.runData" class="space-y-4">
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Source</p>
          <p class="mt-1 text-sm text-slate-700">{{ props.runData.source || 'unknown' }}</p>
        </div>
        <div class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Status</p>
          <p class="mt-1 text-sm text-slate-700">{{ props.runData.status || 'unknown' }}</p>
        </div>
        <div class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Updated</p>
          <p class="mt-1 text-sm text-slate-700">{{ formatTimestamp(props.runData.updatedAt) }}</p>
        </div>
        <div class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Execution</p>
          <p class="mt-1 break-all text-sm text-slate-700">{{ props.runData.executionId || 'n/a' }}</p>
        </div>
      </div>

      <div
        v-if="props.runData.preview"
        class="rounded-lg border border-slate-200 bg-white px-3 py-3 text-sm text-slate-700"
      >
        {{ props.runData.preview }}
      </div>

      <div>
        <p class="mb-2 text-[10px] font-bold uppercase tracking-wide text-slate-400">Payload</p>
        <pre class="max-h-80 overflow-auto rounded-lg border border-slate-200 bg-slate-950 p-3 text-xs text-slate-100">{{ formattedPayload }}</pre>
      </div>
    </div>

    <div
      v-else
      class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-5 text-sm text-slate-500"
    >
      Run this node or execute the workflow to populate node-level run data.
    </div>
  </section>
</template>
