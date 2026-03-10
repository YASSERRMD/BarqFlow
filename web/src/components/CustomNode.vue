<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position, type NodeProps } from '@vue-flow/core'
import { Play } from 'lucide-vue-next'
import { getNodeVisuals } from '../utils/nodeVisuals'

const props = defineProps<NodeProps>()

// data.schema.name is the backend name, e.g. "barqflow-nodes.postgres"
const visuals = computed(() => {
  const backendName = props.data.schema?.name || '';
  return getNodeVisuals(backendName);
})

// Correct text presentation: Top line should be the Node Description/Display Name, bottom line is generic execution or subtitle
const primaryLabel = computed(() => {
  return props.data.schema?.displayName || props.data.schema?.display_name || props.data.label || 'Unknown Node';
})
</script>

<template>
  <div 
    class="min-w-[220px] bg-white rounded-md border shadow-sm transition-all duration-200 group relative"
    :class="[
      selected ? 'ring-1 shadow-md' : 'hover:border-slate-400',
    ]"
    :style="{ 
      borderLeftWidth: '4px', 
      borderLeftColor: visuals.color,
      borderColor: selected ? visuals.color : '#cbd5e1'
    }"
  >
    <!-- Handle (Input) -->
    <Handle 
      v-if="!data.isTrigger"
      id="a" 
      type="target" 
      :position="Position.Left" 
      class="!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-ml-2 !transition-colors !z-10" 
    />

    <!-- Header Section -->
    <div class="px-3 py-2 flex items-center justify-between gap-3">
      <!-- Icon & Title -->
      <div class="flex items-center gap-2 overflow-hidden">
        <div class="w-7 h-7 rounded flex items-center justify-center shrink-0 shadow-sm"
          :style="{ backgroundColor: visuals.iconBgColor, color: visuals.iconColor }"
        >
          <component :is="visuals.icon" class="w-4 h-4" />
        </div>
        <div class="min-w-0">
          <div class="text-[13px] font-bold text-slate-800 truncate">{{ primaryLabel }}</div>
          <div class="text-[10px] text-slate-500 uppercase tracking-wide truncate">{{ data.kind || data.type }}</div>
        </div>
      </div>

      <!-- Action (Run) button - visible on hover -->
      <button 
        class="opacity-0 group-hover:opacity-100 p-1.5 text-slate-400 hover:text-brand-600 hover:bg-brand-50 rounded transition-all shrink-0"
        title="Execute Node"
      >
        <Play class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Execution Status indicators if needed later -->
    <div v-if="data.status" class="px-3 py-1.5 border-t border-slate-100 bg-slate-50 rounded-b-md flex items-center gap-2">
       <span class="w-2 h-2 rounded-full" 
            :class="{
              'bg-blue-500 animate-pulse': data.status === 'running',
              'bg-green-500': data.status === 'success',
              'bg-red-500': data.status === 'error'
            }"></span>
       <span class="text-xs font-medium text-slate-600 capitalize">{{ data.status }}</span>
    </div>

    <!-- Handle (Output) -->
    <Handle 
      id="b" 
      type="source" 
      :position="Position.Right" 
      class="!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-mr-2 !transition-colors !z-10" 
    />
  </div>
</template>

<style scoped>
.scale-102 {
  transform: scale(1.02);
}
</style>
