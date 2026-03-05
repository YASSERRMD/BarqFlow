<script setup lang="ts">
import { X, Save, Trash2 } from 'lucide-vue-next'

defineProps({
  node: {
    type: Object,
    required: false
  }
})
</script>

<template>
  <aside 
    class="w-80 bg-white border-l border-slate-200 flex flex-col transition-all duration-300 transform shadow-[-4px_0_15px_-3px_rgba(0,0,0,0.05)] z-20"
    :class="node ? 'translate-x-0' : 'translate-x-full'"
  >
    <div v-if="node" class="flex-1 flex flex-col h-full overflow-hidden">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
        <h2 class="font-semibold text-slate-800 text-lg">{{ node.name }}</h2>
        <button class="text-slate-400 hover:text-slate-600 transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Properties -->
      <div class="flex-1 overflow-y-auto p-5 space-y-6">
        
        <!-- Node Name -->
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1.5">Node Name</label>
          <input 
            type="text" 
            :value="node.name"
            class="w-full px-3 py-2 bg-white border border-slate-300 rounded-md text-sm shadow-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow"
          />
        </div>

        <div class="h-px bg-slate-100"></div>

        <!-- Node Parameters (Mocked based on type) -->
        <div v-if="node.type === 'action'">
          <label class="block text-sm font-medium text-slate-700 mb-1.5">Method</label>
          <select class="w-full px-3 py-2 bg-white border border-slate-300 rounded-md text-sm shadow-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-brand-500 mb-4">
            <option>GET</option>
            <option>POST</option>
            <option>PUT</option>
          </select>

          <label class="block text-sm font-medium text-slate-700 mb-1.5">URL</label>
          <div class="relative">
            <input 
              type="text" 
              value="https://api.example.com/v1/data"
              class="w-full px-3 py-2 bg-white border border-slate-300 rounded-md text-sm shadow-sm font-mono text-slate-600 focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-brand-500"
            />
            <div class="absolute right-2 top-2 px-1.5 py-0.5 bg-slate-100 text-[10px] font-bold text-slate-500 rounded border border-slate-200">EXPR</div>
          </div>
        </div>

        <div v-if="node.type === 'manipulation'">
           <label class="block text-sm font-medium text-slate-700 mb-1.5">Values to Set</label>
           <div class="bg-slate-50 border border-slate-200 rounded-md p-3">
             <div class="flex items-center gap-2 mb-2">
               <input type="text" value="myValue" class="w-1/2 px-2 py-1 text-sm border border-slate-300 rounded" />
               <span class="text-slate-400">=</span>
               <input type="text" value="100" class="w-1/2 px-2 py-1 text-sm border border-slate-300 rounded" />
             </div>
             <button class="text-xs font-medium text-brand-600 hover:text-brand-700">+ Add Value</button>
           </div>
        </div>

      </div>

      <!-- Footer Actions -->
      <div class="p-4 border-t border-slate-200 bg-slate-50 flex items-center justify-between gap-3">
        <button class="text-red-600 hover:bg-red-50 p-2 rounded-md transition-colors">
          <Trash2 class="w-5 h-5" />
        </button>
        <div class="flex gap-2">
          <button class="px-3 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-md shadow-sm hover:bg-slate-50">Cancel</button>
          <button class="px-3 py-2 text-sm font-medium text-white bg-brand-600 border border-transparent rounded-md shadow-sm hover:bg-brand-700">Save</button>
        </div>
      </div>
    </div>
  </aside>
</template>
