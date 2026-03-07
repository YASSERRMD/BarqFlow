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
import { useNodeStore } from '../stores/nodes'
import { useRoute } from 'vue-router'
import { v4 as uuidv4 } from 'uuid'

const route = useRoute()
const workflowStore = useWorkflowStore()
const nodeStore = useNodeStore()
const { onConnect, addEdges, toObject, setNodes, setEdges } = useVueFlow()

const nodes = ref<any[]>([])
const edges = ref<any[]>([])
const selectedNode = ref<any>(null)

// Load Nodes and Workflow on Mount
onMounted(async () => {
  await nodeStore.fetchNodeTypes()
  if (route.params.id && route.params.id !== 'new') {
    await workflowStore.fetchWorkflow(route.params.id as string)
    const activeWf = workflowStore.activeWorkflow;
    if (activeWf && activeWf.nodes) {
      // Reconstitute nodes mapping backend IWorkflow structure to VueFlow structure
      const loadedNodes: any[] = [];
      const loadedEdges: any[] = [];
      // Backend IWorkflow nodes are an array of node objects, connections are a map Object
      if (Array.isArray(activeWf.nodes)) {
         activeWf.nodes.forEach((n: any) => loadedNodes.push(n))
      }
      if (activeWf.connections) {
         Object.keys(activeWf.connections).forEach(sourceNodeName => {
             const targets = activeWf.connections[sourceNodeName].main[0] || [];
             targets.forEach((t: any) => {
                 const sourceNode = loadedNodes.find(n => n.data.label === sourceNodeName);
                 const targetNode = loadedNodes.find(n => n.data.label === t.node);
                 if (sourceNode && targetNode) {
                    loadedEdges.push({ id: `e-${sourceNode.id}-${targetNode.id}`, source: sourceNode.id, target: targetNode.id, animated: true, style: { stroke: '#0ea5e9', strokeWidth: 2 } });
                 }
             })
         });
      }
      setNodes(loadedNodes);
      setEdges(loadedEdges);
      nodes.value = loadedNodes;
      edges.value = loadedEdges;
    }
  }
})

onConnect((params) => {
  addEdges([{
    ...params,
    animated: true,
    style: { stroke: '#0ea5e9', strokeWidth: 2 }
  }])
})

function onNodeClick({ node }: any) {
  selectedNode.value = node
}

async function handleExecute() {
  if (workflowStore.loading) return
  
  nodes.value.forEach(n => n.data.status = 'running')
  
  try {
    const wfId = route.params.id !== 'new' ? route.params.id as string : '00000000-0000-0000-0000-000000000000' 
    const result = await workflowStore.executeWorkflow(wfId)
    
    nodes.value.forEach(n => {
      const nodeName = n.data.label;
      if (result.data && result.data[nodeName]) {
         n.data.status = result.data[nodeName].success ? 'success' : 'error'
      } else {
         n.data.status = 'success'
      }
    })
  } catch (err) {
    nodes.value.forEach(n => n.data.status = 'error')
  }
}

async function handleSave() {
  const flow = toObject()
  // Translate VueFlow topology to internal BarqFlow JSON schemas
  const payloadConnections: Record<string, { main: any[][] }> = {}
  
  flow.edges.forEach(edge => {
      const sourceNode = flow.nodes.find(n => n.id === edge.source)
      const targetNode = flow.nodes.find(n => n.id === edge.target)
      if (sourceNode && targetNode) {
          if (!payloadConnections[sourceNode.data.label]) {
              payloadConnections[sourceNode.data.label] = { main: [[]] }
          }
          payloadConnections[sourceNode.data.label].main[0].push({
              node: targetNode.data.label,
              type: "main",
              index: 0
          })
      }
  })

  const payloadStr = {
      id: route.params.id !== 'new' ? route.params.id : undefined,
      name: workflowStore.activeWorkflow?.name || 'My New Workflow',
      nodes: flow.nodes,
      connections: payloadConnections,
      settings: {}
  }
  
  await workflowStore.saveWorkflow(payloadStr)
}

function onDragStart(event: DragEvent, nodeTypeObj: any) {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', JSON.stringify(nodeTypeObj))
    event.dataTransfer.effectAllowed = 'move'
  }
}

function onDrop(event: DragEvent) {
  const nodeDataStr = event.dataTransfer?.getData('application/vueflow')
  if (!nodeDataStr) return

  const nodeSchema = JSON.parse(nodeDataStr)
  const position = { x: event.clientX - 300, y: event.clientY - 60 } // Adjust for sidebar
  
  // Set default properties based on schema if not exist
  const propertiesObj: Record<string, any> = {}
  if (nodeSchema.schema && nodeSchema.schema.properties) {
      nodeSchema.schema.properties.forEach((p: any) => {
          if (p.default !== undefined) propertiesObj[p.name] = p.default
      })
  }

  const newNode = {
    id: uuidv4(),
    type: 'custom',
    position,
    data: {
      type: nodeSchema.type,
      label: nodeSchema.name, // Usually needs to be unique per node class
      description: nodeSchema.description,
      status: null,
      schema: nodeSchema.schema,
      properties: propertiesObj
    }
  }

  nodes.value.push(newNode)
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
      <div class="h-full w-full" @drop="onDrop" @dragover.prevent>
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
    </div>

    <!-- Nodes Palette Sidebar -->
    <div v-if="!selectedNode" class="w-80 bg-white border-l border-slate-200 flex flex-col shadow-[-10px_0_20px_rgba(0,0,0,0.02)] z-20">
      <div class="p-6 border-b border-slate-100 dark:border-slate-800 bg-slate-50/50">
        <h3 class="font-bold text-slate-800 text-lg flex items-center gap-2">
          <Settings2 class="w-5 h-5 text-brand-500" /> Available Nodes
        </h3>
        <p class="text-sm text-slate-500 mt-1">Drag and drop nodes onto the canvas to build your workflow.</p>
      </div>
      
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <div v-if="nodeStore.isLoading" class="flex justify-center p-8">
            <Loader2 class="w-6 h-6 animate-spin text-brand-500" />
        </div>
        
        <div 
          v-else
          v-for="nt in nodeStore.nodeTypes" 
          :key="nt.name"
          class="bg-white border hover:border-brand-300 border-slate-200 p-3 rounded-xl shadow-sm cursor-grab active:cursor-grabbing hover:shadow-md transition-all group"
          draggable="true"
          @dragstart="onDragStart($event, nt)"
        >
          <div class="flex items-center gap-3">
            <div :class="[
              'w-8 h-8 rounded-lg flex items-center justify-center transition-colors',
              nt.type === 'trigger' ? 'bg-purple-100 text-purple-600' : 'bg-brand-100 text-brand-600'
            ]">
              <Settings2 class="w-5 h-5" />
            </div>
            <div>
              <h4 class="text-sm font-bold text-slate-800">{{ nt.name }}</h4>
              <p class="text-xs text-slate-500 line-clamp-1 mt-0.5">{{ nt.description }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Properties Panel -->
    <NodePanel v-else :node="selectedNode" @close="selectedNode = null" />
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
