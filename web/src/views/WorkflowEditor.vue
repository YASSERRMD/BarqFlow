<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import { Plus, Play, Save, Settings2, Loader2 } from 'lucide-vue-next'

import CustomNode from '../components/CustomNode.vue'
import NodePanel from '../components/NodePanel.vue'
import { useWorkflowStore } from '../stores/workflows'
import { useRoute } from 'vue-router'

const route = useRoute()
const workflowStore = useWorkflowStore()
const { onConnect, addEdges, toObject } = useVueFlow()

const nodes = ref<any[]>([
  { 
    id: '1', 
    type: 'custom',
    label: 'Manual Trigger', 
    position: { x: 100, y: 150 },
    data: { type: 'trigger', label: 'Manual Trigger', description: 'Click execute to start' }
  },
  { 
    id: '2', 
    type: 'custom',
    label: 'HTTP Request', 
    position: { x: 400, y: 150 },
    data: { type: 'action', label: 'HTTP Request', description: 'GET https://api.example.com' }
  }
])

const edges = ref<any[]>([
  { id: 'e1-2', source: '1', target: '2', animated: true, style: { stroke: '#0ea5e9', strokeWidth: 2 } }
])

const selectedNode = ref<any>(null)

onConnect((params) => {
  addEdges([params])
})

function onNodeClick({ node }: any) {
  selectedNode.value = node
}

async function handleExecute() {
  if (workflowStore.loading) return
  
  // Set running status on all nodes for visual effect
  nodes.value.forEach(n => n.data.status = 'running')
  
  try {
    // In a real app, we'd use route.params.id
    const mockWorkflowId = '00000000-0000-0000-0000-000000000000' 
    const result = await workflowStore.executeWorkflow(mockWorkflowId)
    
    // Update node statuses based on result
    nodes.value.forEach(n => {
      const nodeName = n.data.label;
      if (result.data && result.data[nodeName]) {
         n.data.status = result.data[nodeName].success ? 'success' : 'error'
      } else {
         n.data.status = 'success' // Default for triggers
      }
    })
  } catch (err) {
    nodes.value.forEach(n => n.data.status = 'error')
  }
}

async function handleSave() {
  const flow = toObject()
  console.log('Saving flow:', flow)
  // await workflowStore.saveWorkflow({ ...flow, name: 'My First Workflow' })
}
</script>

<template>
  <div class="h-full w-full flex overflow-hidden bg-canvas">
    <!-- Main Canvas Area -->
    <div class="flex-1 relative overflow-hidden">
      <!-- Toolbar -->
      <div class="absolute top-6 left-6 right-6 flex justify-between items-center z-10 pointer-events-none">
        <div class="bg-white/90 backdrop-blur-xl shadow-lg shadow-slate-200/50 border border-slate-200 rounded-2xl px-6 py-3 flex items-center gap-4 pointer-events-auto">
          <div class="w-10 h-10 bg-brand-50 rounded-xl flex items-center justify-center text-brand-600">
            <Settings2 class="w-6 h-6" />
          </div>
          <div>
            <h1 class="font-bold text-slate-800 text-lg leading-tight">My First Workflow</h1>
            <p class="text-xs text-slate-400 font-medium">Last saved 2m ago</p>
          </div>
          <div class="h-8 w-px bg-slate-100 mx-2"></div>
          <span class="px-2.5 py-1 bg-green-100 text-green-700 text-[10px] font-bold uppercase tracking-wider rounded-lg">Active</span>
        </div>
        
        <div class="flex gap-3 pointer-events-auto">
          <button 
            @click="handleSave"
            class="bg-white/90 backdrop-blur-xl shadow-lg shadow-slate-200/50 border border-slate-200 hover:bg-slate-50 text-slate-700 px-5 py-3 rounded-2xl flex items-center gap-2 transition-all hover:-translate-y-0.5 active:translate-y-0 font-bold text-sm"
          >
            <Save class="w-4 h-4" /> Save
          </button>
          <button 
            @click="handleExecute"
            :disabled="workflowStore.loading"
            class="bg-brand-500 hover:bg-brand-600 shadow-xl shadow-brand-500/30 text-white px-6 py-3 rounded-2xl flex items-center gap-2 transition-all hover:-translate-y-1 active:translate-y-0 font-bold text-sm disabled:opacity-70"
          >
            <Loader2 v-if="workflowStore.loading" class="w-4 h-4 animate-spin" />
            <Play v-else class="w-4 h-4 fill-current" /> 
            {{ workflowStore.loading ? 'Executing...' : 'Execute' }}
          </button>
        </div>
      </div>

      <!-- Add Node FAB -->
      <button class="absolute bottom-8 left-8 w-14 h-14 bg-white shadow-2xl shadow-slate-300 border border-slate-200 rounded-2xl flex items-center justify-center text-slate-600 hover:text-brand-600 hover:border-brand-300 transition-all hover:-translate-y-1 z-10 group">
        <Plus class="w-7 h-7 group-hover:rotate-90 transition-transform duration-300" />
      </button>

      <!-- Vue Flow Canvas -->
      <VueFlow
        v-model:nodes="nodes"
        v-model:edges="edges"
        @node-click="onNodeClick"
        :node-types="{ custom: CustomNode }"
        class="bg-graph-pattern"
        :default-viewport="{ zoom: 1.2, x: 0, y: 0 }"
        :min-zoom="0.2"
        :max-zoom="4"
      >
        <Background pattern-color="#e2e8f0" :gap="24" />
        <Controls position="bottom-right" class="!bg-white !border-slate-200 !shadow-lg !rounded-xl overflow-hidden" />
        <MiniMap class="!bg-white/80 !backdrop-blur-md !border-slate-200 !shadow-xl !rounded-2xl" />
      </VueFlow>
    </div>

    <!-- Properties Panel -->
    <NodePanel :node="selectedNode" />
  </div>
</template>

<style>
/* Custom grid background */
.bg-graph-pattern {
  background-image: radial-gradient(#e2e8f0 1px, transparent 1px);
  background-size: 24px 24px;
}

/* Vue Flow overrides to match theme */
.vue-flow__edge-path {
  stroke-dasharray: 5;
  animation: dash 1s linear infinite;
}

@keyframes dash {
  from {
    stroke-dashoffset: 10;
  }
  to {
    stroke-dashoffset: 0;
  }
}

.vue-flow__handle {
  width: 12px !important;
  height: 12px !important;
  border-radius: 4px !important;
}

.vue-flow__controls-button {
  border-bottom: 1px solid #f1f5f9 !important;
  fill: #64748b !important;
}

.vue-flow__controls-button:hover {
  background-color: #f8fafc !important;
}
</style>
