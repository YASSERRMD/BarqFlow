<script setup lang="ts">
import { ref } from 'vue'
import { Search, X, Plus } from 'lucide-vue-next'
import { useNodeStore } from '../stores/nodes'
import { getNodeVisuals } from '../utils/nodeVisuals'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  dragstart: [event: DragEvent, nodeType: any]
}>()

const nodeStore = useNodeStore()
const searchQuery = ref('')

function onDragStart(event: DragEvent, nodeTypeObj: any) {
  emit('dragstart', event, nodeTypeObj)
}
</script>

<template>
  <div 
    class="fixed inset-y-0 right-0 w-[400px] bg-white shadow-2xl z-40 transform transition-transform duration-300 ease-in-out flex flex-col border-l border-slate-200"
    :class="show ? 'translate-x-0' : 'translate-x-full'"
  >
    <!-- Header -->
    <div class="px-6 py-4 border-b border-slate-100 flex items-center justify-between">
      <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
        <Plus class="w-5 h-5 text-brand-500" />
        Add Node
      </h2>
      <button 
        @click="emit('close')"
        class="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-50 rounded-lg transition-colors"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    <!-- Search -->
    <div class="p-4 border-b border-slate-100">
      <div class="relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Search nodes..."
          class="w-full pl-9 pr-4 py-2 border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-transparent"
        />
      </div>
    </div>

    <!-- Node List -->
    <div class="flex-1 overflow-y-auto p-4 space-y-2 bg-slate-50/50">
      <div 
        v-for="nt in nodeStore.nodeTypes" 
        :key="nt.name"
        class="bg-white border border-slate-200 hover:border-brand-500 hover:shadow-sm p-3 rounded-lg cursor-grab active:cursor-grabbing transition-all group flex flex-col gap-1"
        draggable="true"
        @dragstart="onDragStart($event, nt)"
      >
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded shrink-0 flex items-center justify-center text-slate-600 bg-slate-100"
               :style="{ backgroundColor: getNodeVisuals(nt.schema?.name || '').iconBgColor, color: getNodeVisuals(nt.schema?.name || '').iconColor }">
            <component :is="getNodeVisuals(nt.schema?.name || '').icon" class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <h4 class="text-[13px] font-bold text-slate-900 truncate">{{ nt.name }}</h4>
            <p class="text-[10px] text-slate-500 font-medium uppercase tracking-wider">{{ nt.type }}</p>
          </div>
        </div>
        <p class="text-[11px] text-slate-500 line-clamp-2 mt-1">{{ nt.description }}</p>
      </div>
    </div>
  </div>
</template>
