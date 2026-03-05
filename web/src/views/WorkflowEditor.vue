<script setup lang="ts">
import { ref } from 'vue'
import { Plus, Play, Save, Settings2 } from 'lucide-vue-next'
import NodePanel from '../components/NodePanel.vue'

// Mock state
const nodes = ref([
  { id: '1', name: 'Webhook', type: 'trigger', x: 100, y: 150, selected: false },
  { id: '2', name: 'HTTP Request', type: 'action', x: 350, y: 150, selected: true },
  { id: '3', name: 'Set', type: 'manipulation', x: 600, y: 150, selected: false }
])

const edges = ref([
  { source: '1', target: '2' },
  { source: '2', target: '3' }
])

const selectedNode = ref(nodes.value[1])

function selectNode(id: string) {
  nodes.value.forEach(n => n.selected = n.id === id)
  selectedNode.value = nodes.value.find(n => n.id === id) || nodes.value[0]
}
</script>

<template>
  <div class="h-full w-full flex overflow-hidden">
    <!-- Main Canvas Area -->
    <div class="flex-1 relative bg-canvas bg-graph-pattern overflow-hidden">
      <!-- Toolbar -->
      <div class="absolute top-4 left-4 right-4 flex justify-between items-center z-10">
        <div class="bg-white/80 backdrop-blur-md shadow-sm border border-slate-200 rounded-lg px-4 py-2 flex items-center gap-3">
          <h1 class="font-semibold text-slate-800">My First Workflow</h1>
          <span class="px-2 py-0.5 bg-green-100 text-green-700 text-xs font-medium rounded-full">Active</span>
        </div>
        
        <div class="flex gap-2">
          <button class="bg-white/80 backdrop-blur-md shadow-sm border border-slate-200 hover:bg-slate-50 text-slate-700 px-3 py-2 rounded-lg flex items-center gap-2 transition-colors text-sm font-medium">
            <Save class="w-4 h-4" /> Save
          </button>
          <button class="bg-brand-500 hover:bg-brand-600 shadow-md shadow-brand-500/20 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors text-sm font-medium">
            <Play class="w-4 h-4" /> Execute
          </button>
        </div>
      </div>

      <!-- Add Node FAB -->
      <button class="absolute bottom-6 left-6 w-12 h-12 bg-white shadow-lg border border-slate-200 rounded-full flex items-center justify-center text-slate-600 hover:text-brand-600 hover:border-brand-300 transition-colors z-10 group">
        <Plus class="w-6 h-6 group-hover:scale-110 transition-transform" />
      </button>

      <!-- Nodes -->
      <div class="absolute inset-0 z-0 origin-top-left" style="transform: scale(1) translate(0px, 0px);">
        <!-- SVG for Edges -->
        <svg class="absolute inset-0 w-full h-full pointer-events-none">
          <path 
            v-for="(edge, idx) in edges" :key="idx"
            :d="`M ${nodes.find(n => n.id === edge.source)!.x + 200} ${nodes.find(n => n.id === edge.source)!.y + 35} C ${nodes.find(n => n.id === edge.source)!.x + 250} ${nodes.find(n => n.id === edge.source)!.y + 35}, ${nodes.find(n => n.id === edge.target)!.x - 50} ${nodes.find(n => n.id === edge.target)!.y + 35}, ${nodes.find(n => n.id === edge.target)!.x} ${nodes.find(n => n.id === edge.target)!.y + 35}`"
            fill="none" 
            stroke="#cbd5e1" 
            stroke-width="2"
            class="transition-colors duration-200"
          />
        </svg>

        <!-- Node Components -->
        <div 
          v-for="node in nodes" :key="node.id"
          @click="selectNode(node.id)"
          :class="[
            'absolute w-[200px] bg-white rounded-xl shadow-node border-2 transition-all cursor-pointer select-none',
            node.selected ? 'border-brand-500 ring-2 ring-brand-500/20 shadow-node-hover z-10' : 'border-slate-200 hover:border-slate-300 hover:shadow-md z-0'
          ]"
          :style="`transform: translate(${node.x}px, ${node.y}px)`"
        >
          <!-- Node Header -->
          <div class="px-4 py-3 border-b border-slate-100 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div :class="[
                'w-6 h-6 rounded flex items-center justify-center',
                node.type === 'trigger' ? 'bg-purple-100 text-purple-600' : 'bg-brand-100 text-brand-600'
              ]">
                <Settings2 class="w-4 h-4" />
              </div>
              <span class="font-medium text-slate-800 text-sm">{{ node.name }}</span>
            </div>
          </div>
          
          <!-- Node Body -->
          <div class="p-4 bg-slate-50/50 rounded-b-xl flex items-center">
             <p class="text-xs text-slate-500 leading-tight">Configured node settings appear here.</p>
          </div>

          <!-- Connection points -->
          <div v-if="node.type !== 'trigger'" class="absolute -left-2 top-1/2 -translate-y-1/2 w-4 h-4 bg-white border-2 border-slate-300 rounded-full cursor-crosshair hover:bg-slate-100"></div>
          <div class="absolute -right-2 top-1/2 -translate-y-1/2 w-4 h-4 bg-white border-2 border-slate-300 rounded-full cursor-crosshair hover:bg-slate-100 opacity-0 group-hover:opacity-100 transition-opacity"></div>
        </div>
      </div>
    </div>

    <!-- Properties Panel -->
    <NodePanel :node="selectedNode" />
  </div>
</template>
