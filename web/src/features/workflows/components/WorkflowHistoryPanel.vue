<script setup lang="ts">
import { computed } from 'vue'
import { GitCompareArrows, Loader2, Clock3, X } from 'lucide-vue-next'
import type { WorkflowHistoryDiff, WorkflowHistoryEntry } from '../../../types/contracts'

const props = defineProps<{
  show: boolean
  loading: boolean
  diffLoading: boolean
  history: WorkflowHistoryEntry[]
  diff: WorkflowHistoryDiff | null
  fromVersion: number | null
  toVersion: number | null
}>()

const emit = defineEmits<{
  (event: 'close'): void
  (event: 'update:fromVersion', value: number | null): void
  (event: 'update:toVersion', value: number | null): void
  (event: 'load-diff'): void
}>()

const canCompare = computed(() => {
  return props.fromVersion !== null && props.toVersion !== null && props.fromVersion !== props.toVersion
})

function formatRelativeTime(iso?: string | null) {
  if (!iso) return 'Unknown'

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return 'Unknown'

  const diffSeconds = Math.floor((date.getTime() - Date.now()) / 1000)
  const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })
  const absSeconds = Math.abs(diffSeconds)

  if (absSeconds < 60) return rtf.format(diffSeconds, 'second')

  const diffMinutes = Math.floor(diffSeconds / 60)
  if (Math.abs(diffMinutes) < 60) return rtf.format(diffMinutes, 'minute')

  const diffHours = Math.floor(diffMinutes / 60)
  if (Math.abs(diffHours) < 24) return rtf.format(diffHours, 'hour')

  const diffDays = Math.floor(diffHours / 24)
  if (Math.abs(diffDays) < 30) return rtf.format(diffDays, 'day')

  return date.toLocaleDateString()
}
</script>

<template>
  <div
    v-if="show"
    class="pointer-events-auto absolute right-4 top-44 z-30 flex h-[calc(100%-11rem)] w-full max-w-[28rem] flex-col overflow-hidden rounded-[28px] border border-slate-200 bg-white/95 shadow-[0_28px_90px_rgba(15,23,42,0.16)] backdrop-blur"
  >
    <div class="border-b border-slate-100 px-5 py-4">
      <div class="flex items-start justify-between gap-4">
        <div>
          <p class="text-[11px] font-black uppercase tracking-[0.22em] text-brand-600">Workflow History</p>
          <h3 class="mt-2 text-2xl font-display font-black text-slate-950">Snapshots and visual diff</h3>
          <p class="mt-2 text-sm font-medium leading-6 text-slate-500">
            Compare saved workflow versions across tags, settings, nodes, and connections.
          </p>
        </div>
        <button
          class="inline-flex h-10 w-10 items-center justify-center rounded-2xl border border-slate-200 text-slate-500 transition hover:border-slate-300 hover:text-slate-700"
          @click="emit('close')"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-slate-500">
      <Loader2 class="h-8 w-8 animate-spin text-brand-500" />
      <p class="text-sm font-bold">Loading workflow history…</p>
    </div>

    <div v-else-if="history.length === 0" class="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center text-slate-500">
      <Clock3 class="h-8 w-8 text-slate-400" />
      <p class="text-sm font-bold">No snapshots yet.</p>
      <p class="max-w-xs text-xs font-medium leading-5">
        Save this workflow to create the first history entry, then compare future revisions here.
      </p>
    </div>

    <div v-else class="flex min-h-0 flex-1 flex-col">
      <div class="border-b border-slate-100 px-5 py-4">
        <div class="grid gap-3 sm:grid-cols-2">
          <label class="block">
            <span class="mb-2 block text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">From Version</span>
            <select
              :value="fromVersion ?? ''"
              class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm font-bold text-slate-700 outline-none transition focus:border-brand-400 focus:bg-white"
              @change="emit('update:fromVersion', $event.target ? Number(($event.target as HTMLSelectElement).value) : null)"
            >
              <option v-for="entry in history" :key="`from-${entry.version}`" :value="entry.version">
                v{{ entry.version }} · {{ entry.source }}
              </option>
            </select>
          </label>
          <label class="block">
            <span class="mb-2 block text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">To Version</span>
            <select
              :value="toVersion ?? ''"
              class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm font-bold text-slate-700 outline-none transition focus:border-brand-400 focus:bg-white"
              @change="emit('update:toVersion', $event.target ? Number(($event.target as HTMLSelectElement).value) : null)"
            >
              <option v-for="entry in history" :key="`to-${entry.version}`" :value="entry.version">
                v{{ entry.version }} · {{ entry.source }}
              </option>
            </select>
          </label>
        </div>

        <button
          class="mt-4 inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-black text-white transition hover:bg-slate-900 disabled:cursor-not-allowed disabled:opacity-60"
          :disabled="!canCompare || diffLoading"
          @click="emit('load-diff')"
        >
          <Loader2 v-if="diffLoading" class="h-4 w-4 animate-spin" />
          <GitCompareArrows v-else class="h-4 w-4" />
          {{ diffLoading ? 'Comparing…' : 'Compare Versions' }}
        </button>
      </div>

      <div class="grid min-h-0 flex-1 gap-0 md:grid-cols-[0.94fr_1.06fr]">
        <div class="min-h-0 overflow-auto border-r border-slate-100 px-5 py-4">
          <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Snapshots</p>
          <div class="mt-4 space-y-3">
            <article
              v-for="entry in history"
              :key="entry.version"
              class="rounded-2xl border px-4 py-4 transition"
              :class="[
                fromVersion === entry.version || toVersion === entry.version
                  ? 'border-brand-300 bg-brand-50/50'
                  : 'border-slate-200 bg-white'
              ]"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-black text-slate-900">v{{ entry.version }} · {{ entry.name }}</p>
                  <p class="mt-1 text-[11px] font-bold uppercase tracking-[0.16em] text-slate-400">
                    {{ entry.source }}
                  </p>
                </div>
                <div class="rounded-full px-3 py-1 text-[10px] font-black uppercase tracking-[0.18em]" :class="entry.active ? 'bg-emerald-100 text-emerald-700' : 'bg-slate-100 text-slate-500'">
                  {{ entry.active ? 'Active' : 'Inactive' }}
                </div>
              </div>
              <div class="mt-4 flex flex-wrap gap-2">
                <span
                  v-for="tag in entry.tags"
                  :key="`${entry.version}-${tag}`"
                  class="rounded-full bg-slate-100 px-3 py-1 text-[10px] font-bold uppercase tracking-[0.14em] text-slate-500"
                >
                  {{ tag }}
                </span>
                <span v-if="entry.tags.length === 0" class="rounded-full border border-dashed border-slate-200 px-3 py-1 text-[10px] font-bold uppercase tracking-[0.14em] text-slate-400">
                  No tags
                </span>
              </div>
              <div class="mt-4 grid grid-cols-3 gap-2 text-center">
                <div class="rounded-xl bg-slate-50 px-2 py-2">
                  <p class="text-[10px] font-black uppercase tracking-[0.14em] text-slate-400">Nodes</p>
                  <p class="mt-1 text-sm font-bold text-slate-800">{{ entry.summary.nodeCount }}</p>
                </div>
                <div class="rounded-xl bg-slate-50 px-2 py-2">
                  <p class="text-[10px] font-black uppercase tracking-[0.14em] text-slate-400">Triggers</p>
                  <p class="mt-1 text-sm font-bold text-slate-800">{{ entry.summary.triggerCount }}</p>
                </div>
                <div class="rounded-xl bg-slate-50 px-2 py-2">
                  <p class="text-[10px] font-black uppercase tracking-[0.14em] text-slate-400">Updated</p>
                  <p class="mt-1 text-sm font-bold text-slate-800">{{ formatRelativeTime(entry.createdAt) }}</p>
                </div>
              </div>
            </article>
          </div>
        </div>

        <div class="min-h-0 overflow-auto px-5 py-4">
          <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Diff Summary</p>

          <div v-if="!diff" class="mt-6 rounded-2xl border border-dashed border-slate-200 bg-slate-50/80 p-5 text-sm font-medium leading-6 text-slate-500">
            Choose two versions and run a comparison to see what changed.
          </div>

          <div v-else class="mt-4 space-y-4">
            <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4">
              <p class="text-sm font-black text-slate-900">v{{ diff.fromVersion }} → v{{ diff.toVersion }}</p>
              <p class="mt-2 text-sm font-medium text-slate-600">
                {{ diff.fromName }} → {{ diff.toName }}
              </p>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <div class="rounded-2xl border border-slate-200 bg-white p-4">
                <p class="text-[10px] font-black uppercase tracking-[0.16em] text-slate-400">Name Changed</p>
                <p class="mt-2 text-sm font-bold text-slate-800">{{ diff.nameChanged ? 'Yes' : 'No' }}</p>
              </div>
              <div class="rounded-2xl border border-slate-200 bg-white p-4">
                <p class="text-[10px] font-black uppercase tracking-[0.16em] text-slate-400">Activation Changed</p>
                <p class="mt-2 text-sm font-bold text-slate-800">{{ diff.activeChanged ? 'Yes' : 'No' }}</p>
              </div>
            </div>

            <section class="rounded-2xl border border-slate-200 bg-white p-4">
              <p class="text-[10px] font-black uppercase tracking-[0.16em] text-slate-400">Tags</p>
              <div class="mt-3 flex flex-wrap gap-2">
                <span v-for="tag in diff.tagsAdded" :key="`added-${tag}`" class="rounded-full bg-emerald-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.14em] text-emerald-700">
                  + {{ tag }}
                </span>
                <span v-for="tag in diff.tagsRemoved" :key="`removed-${tag}`" class="rounded-full bg-rose-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.14em] text-rose-700">
                  - {{ tag }}
                </span>
                <span v-if="diff.tagsAdded.length === 0 && diff.tagsRemoved.length === 0" class="text-sm font-medium text-slate-500">
                  No tag changes.
                </span>
              </div>
            </section>

            <section class="rounded-2xl border border-slate-200 bg-white p-4">
              <p class="text-[10px] font-black uppercase tracking-[0.16em] text-slate-400">Settings</p>
              <div class="mt-3 flex flex-wrap gap-2">
                <span v-for="setting in diff.settingsChanged" :key="setting" class="rounded-full bg-amber-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.14em] text-amber-700">
                  {{ setting }}
                </span>
                <span v-if="diff.settingsChanged.length === 0" class="text-sm font-medium text-slate-500">
                  No settings changed.
                </span>
              </div>
            </section>

            <section class="rounded-2xl border border-slate-200 bg-white p-4">
              <p class="text-[10px] font-black uppercase tracking-[0.16em] text-slate-400">Nodes</p>
              <div class="mt-3 space-y-3">
                <div class="flex flex-wrap gap-2">
                  <span v-for="node in diff.nodesAdded" :key="`node-added-${node}`" class="rounded-full bg-emerald-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.14em] text-emerald-700">
                    + {{ node }}
                  </span>
                  <span v-for="node in diff.nodesRemoved" :key="`node-removed-${node}`" class="rounded-full bg-rose-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.14em] text-rose-700">
                    - {{ node }}
                  </span>
                </div>
                <div v-if="diff.nodesChanged.length > 0" class="space-y-2">
                  <article
                    v-for="node in diff.nodesChanged"
                    :key="node.nodeId"
                    class="rounded-xl bg-slate-50 px-3 py-3"
                  >
                    <p class="text-sm font-bold text-slate-800">{{ node.nodeName }}</p>
                    <div class="mt-2 flex flex-wrap gap-2">
                      <span
                        v-for="field in node.changedFields"
                        :key="`${node.nodeId}-${field}`"
                        class="rounded-full bg-white px-3 py-1 text-[10px] font-black uppercase tracking-[0.14em] text-slate-500"
                      >
                        {{ field }}
                      </span>
                    </div>
                  </article>
                </div>
                <span v-if="diff.nodesAdded.length === 0 && diff.nodesRemoved.length === 0 && diff.nodesChanged.length === 0" class="text-sm font-medium text-slate-500">
                  No node changes.
                </span>
              </div>
            </section>

            <section class="rounded-2xl border border-slate-200 bg-white p-4">
              <p class="text-[10px] font-black uppercase tracking-[0.16em] text-slate-400">Connections</p>
              <div class="mt-3 space-y-2 text-sm font-medium text-slate-600">
                <p v-for="connection in diff.connectionsAdded" :key="`connection-added-${connection}`" class="rounded-xl bg-emerald-50 px-3 py-2 text-emerald-700">
                  + {{ connection }}
                </p>
                <p v-for="connection in diff.connectionsRemoved" :key="`connection-removed-${connection}`" class="rounded-xl bg-rose-50 px-3 py-2 text-rose-700">
                  - {{ connection }}
                </p>
                <p v-if="diff.connectionsAdded.length === 0 && diff.connectionsRemoved.length === 0" class="text-sm font-medium text-slate-500">
                  No connection changes.
                </p>
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
