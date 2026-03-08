<script setup lang="ts">
import { ref, computed } from 'vue'
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
const selectedCategory = ref('All')

const categories = computed(() => {
  const cats = new Set(nodeStore.nodeTypes.map(n => n.category))
  return ['All', ...Array.from(cats)]
})

const filteredNodes = computed(() => {
  return nodeStore.nodeTypes.filter(nt => {
    const matchesSearch = (nt.name || '').toLowerCase().includes(searchQuery.value.toLowerCase()) || 
                          (nt.schema?.name || '').toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesCategory = selectedCategory.value === 'All' || nt.category === selectedCategory.value
    return matchesSearch && matchesCategory
  }).sort((a, b) => (a.name || '').localeCompare(b.name || ''))
})

function onDragStart(event: DragEvent, nodeTypeObj: any) {
  emit('dragstart', event, nodeTypeObj)
}
</script>

<template>
  <div 
    class="fixed inset-y-0 right-0 w-[420px] bg-white shadow-2xl z-40 transform transition-transform duration-300 ease-in-out flex flex-col border-l border-slate-200"
    :class="show ? 'translate-x-0' : 'translate-x-full'"
  >
    <!-- Header -->
    <div class="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
      <h2 class="text-lg font-bold text-slate-800 flex items-center gap-2">
        <Plus class="w-5 h-5 text-brand-500" />
        Nodes NodeCreator
      </h2>
      <button 
        @click="emit('close')"
        class="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-200 rounded-lg transition-colors"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    <!-- Search & Categories -->
    <div class="p-4 border-b border-slate-100 space-y-4">
      <div class="relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Search by node name..."
          class="w-full pl-9 pr-4 py-2 border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-transparent transition-all"
        />
      </div>
      
      <div class="flex gap-2 overflow-x-auto pb-1 scrollbar-hide">
        <button
          v-for="cat in categories"
          :key="cat"
          @click="selectedCategory = cat"
          class="px-3 py-1.5 text-xs font-semibold rounded-full whitespace-nowrap transition-colors border"
          :class="selectedCategory === cat ? 'bg-slate-800 text-white border-slate-800 shadow-sm' : 'bg-white text-slate-600 border-slate-200 hover:bg-slate-50 hover:border-slate-300'"
        >
          {{ cat }}
        </button>
      </div>
    </div>

    <!-- Node List -->
    <div class="flex-1 overflow-y-auto p-4 space-y-3 bg-slate-50/50">
      <div 
        v-for="nt in filteredNodes" 
        :key="nt.schema?.name"
        class="bg-white border border-slate-200 hover:border-slate-300 hover:shadow-md p-3.5 rounded-xl cursor-grab active:cursor-grabbing transition-all group flex flex-col gap-2"
        draggable="true"
        @dragstart="onDragStart($event, nt)"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg shrink-0 flex items-center justify-center text-slate-600 bg-slate-100 shadow-inner"
               :style="{ backgroundColor: getNodeVisuals(nt.schema?.name || '').iconBgColor, color: getNodeVisuals(nt.schema?.name || '').iconColor }">
            <component :is="getNodeVisuals(nt.schema?.name || '').icon" class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <h4 class="text-[14px] font-bold text-slate-900 truncate">{{ nt.name || nt.schema?.name }}</h4>
            <p class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider mt-0.5">{{ nt.category }}</p>
          </div>
        </div>
        <p class="text-[12px] text-slate-500 line-clamp-2 mt-1">{{ nt.description }}</p>
      </div>
      
      <div v-if="filteredNodes.length === 0" class="text-center py-10 flex flex-col items-center justify-center text-slate-500">
        <Search class="w-8 h-8 text-slate-300 mb-3" />
        <p class="text-sm font-medium">No nodes found</p>
        <p class="text-xs mt-1">Try adjusting your search or category.</p>
      </div>
    </div>
  </div>
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
