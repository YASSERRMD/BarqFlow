<script setup lang="ts">
const props = defineProps<{
  node: any
  nodeSchema: any
  documentationUrl: string | null
}>()
</script>

<template>
  <section class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
    <div>
      <h3 class="font-semibold text-slate-800">Settings</h3>
      <p class="mt-1 text-xs text-slate-500">
        Control node identity, execution behavior, and documentation shortcuts.
      </p>
    </div>

    <div class="space-y-4">
      <div>
        <label class="mb-1.5 block text-sm font-medium text-slate-700">Node label</label>
        <input
          v-model="props.node.data.label"
          type="text"
          class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500"
        />
      </div>

      <label class="flex items-center justify-between rounded-lg border border-slate-200 bg-slate-50 px-3 py-3">
        <div>
          <p class="text-sm font-medium text-slate-700">Disable node</p>
          <p class="mt-0.5 text-xs text-slate-500">Keep the node in the graph without executing it.</p>
        </div>
        <input
          v-model="props.node.data.disabled"
          type="checkbox"
          class="h-4 w-4 rounded border-slate-300 bg-white text-brand-600 focus:ring-brand-500"
        />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-lg border border-slate-200 bg-white px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Node ID</p>
          <p class="mt-1 break-all text-sm text-slate-700">{{ props.node.id }}</p>
        </div>
        <div class="rounded-lg border border-slate-200 bg-white px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Type Version</p>
          <p class="mt-1 text-sm text-slate-700">
            {{ props.node.data.typeVersion || props.nodeSchema?.typeVersion || 1 }}
          </p>
        </div>
        <div class="rounded-lg border border-slate-200 bg-white px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Internal Type</p>
          <p class="mt-1 break-all text-sm text-slate-700">{{ props.node.data.type }}</p>
        </div>
        <div class="rounded-lg border border-slate-200 bg-white px-3 py-3">
          <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Node Kind</p>
          <p class="mt-1 text-sm text-slate-700">{{ props.node.data.kind || 'action' }}</p>
        </div>
      </div>

      <div class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-3">
        <p class="text-[10px] font-bold uppercase tracking-wide text-slate-400">Schema Summary</p>
        <p class="mt-1 text-sm text-slate-700">
          {{ props.nodeSchema?.properties?.length || 0 }} parameters,
          {{ props.nodeSchema?.credentials?.length || 0 }} credential refs,
          max inputs {{ props.nodeSchema?.maxInputs || 1 }}.
        </p>
      </div>

      <div v-if="props.documentationUrl">
        <a
          :href="props.documentationUrl"
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex items-center gap-1.5 text-sm font-medium text-brand-600 hover:text-brand-700"
        >
          Open node documentation
        </a>
      </div>
    </div>
  </section>
</template>
