<script setup lang="ts">
import { Handle, Position, type NodeProps } from '@vue-flow/core'
import { Settings2, Play, AlertCircle, CheckCircle2 } from 'lucide-vue-next'

const props = defineProps<NodeProps>()
</script>

<template>
  <div 
    :class="[
      'min-w-[180px] bg-white rounded-xl shadow-node border-2 transition-all duration-200 group',
      selected ? 'border-brand-500 ring-4 ring-brand-500/10 shadow-lg scale-102' : 'border-slate-200 hover:border-slate-300'
    ]"
  >
    <!-- Status Indicator -->
    <div v-if="data.status" class="absolute -top-2 -right-2 z-10">
      <div v-if="data.status === 'success'" class="bg-green-500 text-white p-1 rounded-full shadow-sm">
        <CheckCircle2 class="w-3.5 h-3.5" />
      </div>
      <div v-else-if="data.status === 'error'" class="bg-red-500 text-white p-1 rounded-full shadow-sm">
        <AlertCircle class="w-3.5 h-3.5" />
      </div>
      <div v-else-if="data.status === 'running'" class="bg-brand-500 text-white p-1 rounded-full shadow-sm animate-pulse">
        <Play class="w-3.5 h-3.5" />
      </div>
    </div>

    <!-- Node Header -->
    <div class="px-4 py-3 border-b border-slate-100 flex items-center gap-3">
      <div :class="[
        'w-8 h-8 rounded-lg flex items-center justify-center shadow-sm transition-colors',
        data.type === 'trigger' ? 'bg-purple-100 text-purple-600' : 'bg-brand-100 text-brand-600'
      ]">
        <Settings2 class="w-5 h-5" />
      </div>
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-semibold text-slate-800 truncate">{{ data.label }}</h3>
        <p class="text-[10px] text-slate-400 font-medium uppercase tracking-wider">{{ data.type }}</p>
      </div>
    </div>

    <!-- Node Body -->
    <div class="px-4 py-3 bg-slate-50/50 rounded-b-xl">
      <p class="text-[11px] text-slate-500 line-clamp-2 leading-relaxed italic">
        {{ data.description || 'No configuration set' }}
      </p>
    </div>

    <!-- Handles -->
    <Handle
      v-if="data.type !== 'trigger'"
      type="target"
      :position="Position.Left"
      class="!w-3 !h-3 !bg-white !border-2 !border-slate-300 hover:!border-brand-400 !transition-colors !z-20"
    />
    <Handle
      type="source"
      :position="Position.Right"
      class="!w-3 !h-3 !bg-white !border-2 !border-slate-300 hover:!border-brand-400 !transition-colors !z-20"
    />
  </div>
</template>

<style scoped>
.scale-102 {
  transform: scale(1.02);
}
</style>
