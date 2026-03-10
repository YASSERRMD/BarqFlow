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
const activeCategory = ref('All')
const supportFilter = ref<'supported' | 'beta' | 'all'>('supported')

const categories = computed(() => {
  const values = new Set(nodeStore.nodeTypes.map((node: any) => node.category || 'Core'))
  return ['All', ...Array.from(values).sort()]
})

const supportFilters = [
  { id: 'supported', label: 'Production' },
  { id: 'beta', label: 'Beta' },
  { id: 'all', label: 'All' },
] as const

const filteredNodes = computed(() => {
  let nodes = nodeStore.nodeTypes

  if (supportFilter.value !== 'all') {
    nodes = nodes.filter((node: any) => node.supportTier === supportFilter.value)
  }

  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    nodes = nodes.filter(
      (n: any) =>
        n.name.toLowerCase().includes(q) ||
        n.description.toLowerCase().includes(q) ||
        n.schema?.name?.toLowerCase().includes(q) ||
        n.supportNote?.toLowerCase().includes(q),
    )
  }

  if (activeCategory.value !== 'All') {
    nodes = nodes.filter((n: any) => n.category === activeCategory.value)
  }

  const grouped: Record<string, any[]> = {}
  nodes.forEach((n: any) => {
    const cat = n.category || 'Core'
    if (!grouped[cat]) grouped[cat] = []
    grouped[cat].push(n)
  })

  Object.values(grouped).forEach((items) => {
    items.sort((left: any, right: any) => left.name.localeCompare(right.name))
  })

  return grouped
})

function onDragStart(event: DragEvent, nodeTypeObj: any) {
  emit('dragstart', event, nodeTypeObj)
}

function supportBadgeClass(tier: string) {
  switch (tier) {
    case 'supported':
      return 'bg-emerald-100 text-emerald-700'
    case 'beta':
      return 'bg-amber-100 text-amber-700'
    default:
      return 'bg-slate-100 text-slate-600'
  }
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

    <!-- Search & Categories -->
    <div class="px-4 pt-4 border-b border-slate-100 bg-white">
      <div class="relative mb-3">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Search nodes..."
          class="w-full pl-9 pr-4 py-2 border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-transparent"
        />
      </div>

      <div class="mb-3 flex gap-2">
        <button
          v-for="filter in supportFilters"
          :key="filter.id"
          @click="supportFilter = filter.id"
          class="rounded-full px-3 py-1.5 text-xs font-semibold transition-colors"
          :class="supportFilter === filter.id ? 'bg-slate-950 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'"
        >
          {{ filter.label }}
        </button>
      </div>
      
      <!-- Category Tabs -->
      <div class="flex overflow-x-auto no-scrollbar gap-2 pb-3">
        <button 
          v-for="cat in categories" 
          :key="cat"
          @click="activeCategory = cat"
          class="whitespace-nowrap px-3 py-1 text-xs font-semibold rounded-full transition-colors"
          :class="activeCategory === cat ? 'bg-brand-500 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'"
        >
          {{ cat }}
        </button>
      </div>
    </div>

    <!-- Node List -->
    <div class="flex-1 overflow-y-auto p-4 bg-slate-50/50">
      <template v-for="(nodes, catName) in filteredNodes" :key="catName">
        <div class="mb-6">
          <h3 class="text-xs font-bold text-slate-400 uppercase tracking-widest mb-3 px-1">{{ catName }}</h3>
          <div class="space-y-2">
            <div 
              v-for="nt in nodes" 
              :key="nt.schema?.name || nt.name"
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
                  <div class="flex items-center gap-2">
                    <h4 class="text-[13px] font-bold text-slate-900 truncate">{{ nt.name }}</h4>
                    <span
                      class="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-[0.16em]"
                      :class="supportBadgeClass(nt.supportTier)"
                    >
                      {{ nt.supportTier }}
                    </span>
                  </div>
                  <p class="text-[10px] text-slate-500 font-medium uppercase tracking-wider">{{ nt.kind }}</p>
                </div>
              </div>
              <p class="text-[11px] text-slate-500 line-clamp-2 mt-1 px-1 leading-snug">{{ nt.description }}</p>
              <p v-if="nt.supportNote" class="px-1 text-[11px] leading-snug text-slate-400">
                {{ nt.supportNote }}
              </p>
            </div>
          </div>
        </div>
      </template>
      <div v-if="Object.keys(filteredNodes).length === 0" class="text-center py-10 text-slate-500 text-sm">
        No nodes found matching your search.
      </div>
    </div>
  </div>
</template>
