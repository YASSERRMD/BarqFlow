<script setup lang="ts">
import { Handle, Position, type NodeProps } from '@vue-flow/core'
import { Settings2, Play, AlertCircle, CheckCircle2 } from 'lucide-vue-next'

const props = defineProps<NodeProps>()
</script>

<template>
  <div 
    class="min-w-[220px] bg-white rounded-md border shadow-sm transition-all duration-200 group relative"
    :class="[
      selected ? 'border-brand-500 ring-1 ring-brand-500 shadow-md' : 'border-slate-300 hover:border-slate-400',
      data.schema?.type === 'trigger' ? 'border-l-4 border-l-purple-500' : 'border-l-4 border-l-brand-500'
    ]"
  >
    <!-- Handle (Input) -->
    <Handle 
      v-if="data.schema?.type !== 'trigger'"
      id="a" 
      type="target" 
      :position="Position.Left" 
      class="!w-4 !h-6 !bg-slate-300 !border-2 !border-white !rounded-sm hover:!bg-brand-500 !-ml-2 !transition-colors !z-10" 
    />

    <!-- Header Section -->
    <div class="px-3 py-2 flex items-center justify-between gap-3">
      <!-- Icon & Title -->
      <div class="flex items-center gap-2 overflow-hidden">
        <div class="w-6 h-6 rounded flex items-center justify-center shrink-0"
          :class="data.schema?.type === 'trigger' ? 'text-purple-600 bg-purple-50' : 'text-brand-600 bg-brand-50'"
        >
          <Settings2 class="w-4 h-4" />
        </div>
        <div class="min-w-0">
          <div class="text-sm font-semibold text-slate-800 truncate">{{ data.label }}</div>
          <div class="text-[10px] text-slate-500 uppercase tracking-wide truncate">{{ data.schema?.type || data.type }}</div>
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
