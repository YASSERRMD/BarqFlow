<script setup lang="ts">
import { X, Save, Trash2, Info, ExternalLink } from 'lucide-vue-next'

import { useNodeStore } from '../stores/nodes'
import { computed } from 'vue'

const nodeStore = useNodeStore()

const props = defineProps({
  node: {
    type: Object,
    required: false
  }
})

const nodeSchema = computed(() => {
  if (!props.node) return null;
  // Match the node type from the store
  const matchedType = nodeStore.nodeTypes.find((n: any) => n.schema?.name === props.node?.data?.type);
  if (matchedType) return matchedType.schema;

  // Fallback for mock nodes
  return props.node.data.schema || null;
})

function getCategoryColor(type: string) {
  switch (type) {
    case 'trigger': return 'bg-purple-50 text-purple-700 border-purple-100'
    case 'logic': return 'bg-amber-50 text-amber-700 border-amber-100'
    case 'manipulation': return 'bg-blue-50 text-blue-700 border-blue-100'
    default: return 'bg-brand-50 text-brand-700 border-brand-100'
  }
}
</script>

<template>
  <aside 
    class="w-[380px] bg-white border-l border-slate-200 flex flex-col transition-all duration-500 ease-in-out transform shadow-[-10px_0_30px_-5px_rgba(0,0,0,0.03)] z-50 overflow-hidden"
    :class="node ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0 pointer-events-none'"
  >
    <div v-if="node" class="flex-1 flex flex-col h-full">
      <!-- Header -->
      <div class="px-7 py-6 border-b border-slate-100 flex items-center justify-between bg-white relative">
        <div class="flex flex-col">
          <div :class="['inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-widest border mb-2 w-fit', getCategoryColor(node.data.type)]">
            {{ node.data.type }}
          </div>
          <h2 class="font-black text-slate-900 text-xl tracking-tight leading-none">{{ node.data.label }}</h2>
        </div>
        <button class="w-10 h-10 rounded-xl hover:bg-slate-100 flex items-center justify-center text-slate-400 hover:text-slate-900 transition-all">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Scrollable Properties Area -->
      <div class="flex-1 overflow-y-auto px-7 py-8 space-y-8 scrollbar-hide">
        
        <!-- Description Info Box -->
        <div class="bg-slate-50 border border-slate-100 rounded-2xl p-4 flex gap-3 items-start">
          <Info class="w-4 h-4 text-slate-400 shrink-0 mt-0.5" />
          <p class="text-xs text-slate-500 font-medium leading-relaxed">
            {{ node.data.description || 'Configure this node to handle your workflow data processing requirements.' }}
          </p>
        </div>

        <!-- Node Configuration -->
        <div class="space-y-6">
          <!-- Dynamic Node Schema Renderer -->
          <div v-if="nodeSchema && nodeSchema.properties">
            <template v-for="(prop, pIdx) in nodeSchema.properties" :key="pIdx">
              <div class="mb-5">
                <label class="block text-sm font-bold text-slate-700 mb-2">{{ prop.displayName }}</label>
                
                <div v-if="prop.type === 'string' || prop.type === 'text'" class="relative group">
                  <input 
                    type="text" 
                    :placeholder="prop.placeholder || ''"
                    class="w-full pl-4 pr-16 py-3 bg-slate-50 border-2 border-transparent focus:border-brand-500 focus:bg-white rounded-xl text-sm font-medium transition-all outline-none"
                  />
                  <div v-if="prop.type === 'string'" class="absolute right-3 top-1/2 -translate-y-1/2 px-2 py-1 bg-white border border-slate-200 text-[10px] font-black text-slate-400 rounded-lg shadow-sm">EXPR</div>
                </div>

                <div v-else-if="prop.type === 'options'">
                  <select class="w-full px-4 py-3 bg-slate-50 border-2 border-transparent focus:border-brand-500 focus:bg-white rounded-xl text-sm font-bold text-slate-800 transition-all outline-none">
                    <option v-for="opt in prop.options" :key="opt.value" :value="opt.value">
                      {{ opt.name }}
                    </option>
                  </select>
                </div>

                <div v-else-if="prop.type === 'boolean'" class="flex items-center gap-2 mt-2">
                  <input type="checkbox" class="w-4 h-4 text-brand-500 bg-slate-50 border-slate-300 rounded focus:ring-brand-500" />
                  <span class="text-sm font-medium text-slate-700">{{ prop.description || prop.displayName }}</span>
                </div>
                
                <div v-else-if="prop.type === 'collection' || prop.type === 'fixedCollection'">
                  <button class="w-full py-3 border-2 border-dashed border-slate-200 rounded-xl text-xs font-bold text-slate-400 hover:border-brand-300 hover:text-brand-600 hover:bg-brand-50/30 transition-all">+ Add {{ prop.displayName }}</button>
                </div>
                
                <p v-if="prop.description && prop.type !== 'boolean'" class="mt-2 text-xs text-slate-400">{{ prop.description }}</p>
              </div>
            </template>
          </div>

          <!-- Fallback Hardcoded UI for unlinked schema types -->
          <div v-else-if="node.data.type === 'action'">
            <p class="text-xs text-slate-400 italic mb-4">No dynamic schema available from backend. Using fallback rendering.</p>
            <label class="block text-xs font-black text-slate-400 uppercase tracking-widest mb-3">HTTP Configuration</label>
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-bold text-slate-700 mb-2">Request Method</label>
                <select class="w-full px-4 py-3 bg-slate-50 border-2 border-transparent focus:border-brand-500 focus:bg-white rounded-xl text-sm font-bold text-slate-800 transition-all outline-none">
                  <option>GET</option>
                  <option>POST</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Documentation Link -->
          <div class="pt-4">
            <a href="#" class="inline-flex items-center gap-1.5 text-xs font-bold text-brand-600 hover:text-brand-700 transition-colors">
              <ExternalLink class="w-3 h-3" /> View Node Documentation
            </a>
          </div>
        </div>
      </div>

      <!-- Fixed Footer -->
      <div class="px-7 py-6 border-t border-slate-100 bg-white flex items-center justify-between gap-4">
        <button class="w-12 h-12 rounded-2xl border-2 border-red-50 text-red-400 hover:bg-red-50 hover:text-red-500 hover:border-red-100 flex items-center justify-center transition-all group">
          <Trash2 class="w-5 h-5 group-hover:scale-110 transition-transform" />
        </button>
        <div class="flex-1 flex gap-3">
          <button class="flex-1 py-3 text-sm font-bold text-slate-500 hover:bg-slate-50 rounded-2xl transition-all">Discard</button>
          <button class="flex-[1.5] py-3 text-sm font-bold text-white bg-slate-900 hover:bg-slate-800 rounded-2xl shadow-xl shadow-slate-900/10 transition-all hover:-translate-y-1 active:translate-y-0">Apply Changes</button>
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
