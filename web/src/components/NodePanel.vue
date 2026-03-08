<script setup lang="ts">
import { computed, ref } from 'vue'
import { X, Play, Trash2, Info, ExternalLink, Settings2 } from 'lucide-vue-next'
import { useNodeStore } from '../stores/nodes'

const nodeStore = useNodeStore()

interface NodeTestState {
  nodeId: string
  status: 'running' | 'success' | 'error'
  message: string
}

const props = defineProps<{
  node?: any
  testState?: NodeTestState | null
}>()

const emit = defineEmits<{
  (event: 'close'): void
  (event: 'test-node', node: any): void
  (event: 'delete-node', nodeId: string): void
}>()

const localNotice = ref<string | null>(null)

const nodeSchema = computed(() => {
  if (!props.node) return null

  const matchedType = nodeStore.nodeTypes.find(
    (n: any) => n.schema?.name === props.node?.data?.type,
  )
  if (matchedType) return matchedType.schema

  return props.node.data.schema || null
})

const documentationUrl = computed(() => {
  const schema = nodeSchema.value as any
  return schema?.documentation_url || schema?.documentationUrl || null
})

function getCategoryColor(type: string) {
  switch (type) {
    case 'trigger':
      return 'bg-purple-100 text-purple-700'
    case 'logic':
      return 'bg-amber-100 text-amber-700'
    case 'manipulation':
      return 'bg-blue-100 text-blue-700'
    default:
      return 'bg-brand-100 text-brand-700'
  }
}

const panelTestState = computed(() => {
  if (!props.node || !props.testState) return null
  if (props.testState.nodeId !== props.node.id) return null
  return props.testState
})

const isTesting = computed(() => panelTestState.value?.status === 'running')

function mockValueForProperty(prop: any) {
  const name = String(prop?.name || '').toLowerCase()

  if (prop?.type === 'options') {
    return prop?.options?.[0]?.value ?? null
  }

  if (prop?.type === 'boolean') {
    return true
  }

  if (prop?.type === 'string' || prop?.type === 'text') {
    if (name.includes('url')) return 'http://localhost:11434'
    if (name.includes('model')) return 'llama3.2'
    if (name.includes('prompt')) return 'Write one sentence about workflow automation.'
    if (name.includes('api') && name.includes('key')) return 'test-key'
    return 'sample-value'
  }

  return null
}

function applyMockData() {
  if (!props.node || !nodeSchema.value) return

  const schema: any = nodeSchema.value
  const target = props.node.data.properties || {}
  let updated = 0

  for (const prop of schema.properties || []) {
    const current = target[prop.name]
    if (current !== undefined && current !== null && String(current).length > 0) {
      continue
    }

    const next = mockValueForProperty(prop)
    if (next !== null) {
      target[prop.name] = next
      updated += 1
    }
  }

  props.node.data.properties = { ...target }
  localNotice.value =
    updated > 0
      ? `Injected mock values for ${updated} parameter(s).`
      : 'No empty parameters found for mock injection.'
}

function onDeleteNode() {
  if (!props.node?.id) return
  emit('delete-node', String(props.node.id))
}
</script>

<template>
  <div v-if="node" class="fixed inset-0 bg-slate-900/20 z-40 transition-opacity" @click="emit('close')"></div>

  <aside
    class="fixed inset-y-0 right-0 w-[450px] bg-white shadow-2xl flex flex-col transition-transform duration-300 ease-in-out z-50 border-l border-slate-200"
    :class="node ? 'translate-x-0' : 'translate-x-full pointer-events-none'"
  >
    <div v-if="node" class="flex-1 flex flex-col h-full bg-slate-50">
      <div class="px-6 py-4 border-b border-slate-200 bg-white flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div :class="['w-8 h-8 rounded flex items-center justify-center', getCategoryColor(node.data.kind || node.data.type)]">
            <Settings2 class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-lg font-bold text-slate-800 leading-tight">{{ node.data.label }}</h2>
            <div class="text-[10px] font-semibold uppercase tracking-wider text-slate-500 mt-0.5">
              {{ node.data.kind || node.data.type }} Node
            </div>
          </div>
        </div>
        <button @click="emit('close')" class="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-6 py-6 space-y-6">
        <div class="bg-blue-50/50 border border-blue-100 rounded-lg p-3 flex gap-3 text-sm text-blue-800">
          <Info class="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
          <p>{{ node.data.description || 'Configure this node to handle your workflow data processing requirements.' }}</p>
        </div>

        <div class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
          <h3 class="font-semibold text-slate-800 mb-4">Parameters</h3>

          <div v-if="nodeSchema && nodeSchema.properties" class="space-y-4">
            <template v-for="(prop, pIdx) in nodeSchema.properties" :key="pIdx">
              <div>
                <label class="block text-sm font-medium text-slate-700 mb-1.5">{{ prop.displayName }}</label>

                <div v-if="prop.type === 'string' || prop.type === 'text'" class="relative">
                  <input
                    v-model="node.data.properties[prop.name]"
                    type="text"
                    :placeholder="prop.placeholder || ''"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm"
                  />
                </div>

                <div v-else-if="prop.type === 'options'">
                  <select
                    v-model="node.data.properties[prop.name]"
                    class="w-full px-3 py-2 bg-white border border-slate-300 focus:border-brand-500 focus:ring-1 focus:ring-brand-500 rounded-md text-sm text-slate-900 shadow-sm"
                  >
                    <option v-for="opt in prop.options" :key="opt.value" :value="opt.value">
                      {{ opt.name }}
                    </option>
                  </select>
                </div>

                <div v-else-if="prop.type === 'boolean'" class="flex items-center mt-2">
                  <input
                    v-model="node.data.properties[prop.name]"
                    type="checkbox"
                    class="w-4 h-4 text-brand-600 bg-white border-slate-300 rounded focus:ring-brand-500"
                  />
                  <label class="ml-2 text-sm text-slate-700">{{ prop.description || prop.displayName }}</label>
                </div>

                <div v-else-if="prop.type === 'collection' || prop.type === 'fixedCollection'">
                  <button class="w-full py-2 bg-slate-50 border border-dashed border-slate-300 rounded-md text-sm font-medium text-slate-600 hover:border-brand-500 hover:text-brand-600 transition-colors">
                    + Add {{ prop.displayName }}
                  </button>
                </div>

                <p v-if="prop.description && prop.type !== 'boolean'" class="mt-1 text-xs text-slate-500">{{ prop.description }}</p>
              </div>
            </template>
          </div>

          <div v-else-if="(node.data.kind || node.data.type) === 'action'" class="space-y-4">
            <p class="text-xs text-amber-600 bg-amber-50 p-2 rounded border border-amber-200">No dynamic schema available from backend. Using fallback rendering.</p>
          </div>
        </div>

        <div v-if="documentationUrl">
          <a :href="documentationUrl" target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1.5 text-sm font-medium text-brand-600 hover:text-brand-700">
            <ExternalLink class="w-4 h-4" /> View Documentation
          </a>
        </div>
      </div>

      <div
        v-if="panelTestState"
        :class="[
          'mx-6 mb-3 px-3 py-2 rounded-lg text-xs font-medium',
          panelTestState.status === 'success' && 'bg-green-50 text-green-700 border border-green-200',
          panelTestState.status === 'error' && 'bg-red-50 text-red-700 border border-red-200',
          panelTestState.status === 'running' && 'bg-blue-50 text-blue-700 border border-blue-200',
        ]"
      >
        {{ panelTestState.message }}
      </div>

      <div v-if="localNotice" class="mx-6 mb-3 px-3 py-2 rounded-lg text-xs font-medium bg-amber-50 text-amber-700 border border-amber-200">
        {{ localNotice }}
      </div>

      <div class="px-6 py-4 border-t border-slate-200 bg-white flex items-center justify-between gap-3">
        <button
          @click="onDeleteNode"
          class="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors border border-transparent hover:border-red-100"
          title="Delete Node"
        >
          <Trash2 class="w-5 h-5" />
        </button>
        <div class="flex gap-2">
          <button
            @click="applyMockData"
            class="px-4 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-lg hover:bg-slate-50 shadow-sm"
          >
            Mock Data
          </button>
          <button
            @click="emit('test-node', node)"
            :disabled="isTesting"
            class="px-4 py-2 text-sm font-medium text-white bg-brand-600 rounded-lg hover:bg-brand-700 shadow-sm flex items-center gap-2 disabled:opacity-60"
          >
            <Play class="w-4 h-4 fill-current" /> {{ isTesting ? 'Testing...' : 'Test Step' }}
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
