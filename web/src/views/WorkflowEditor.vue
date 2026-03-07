<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import { Plus, Play, Save, Settings2, Loader2 } from 'lucide-vue-next'

import CustomNode from '../components/CustomNode.vue'
import NodeCreator from '../components/NodeCreator.vue'
import NodePanel from '../components/NodePanel.vue'
import { useWorkflowStore } from '../stores/workflows'
import { useNodeStore } from '../stores/nodes'
import { useRoute } from 'vue-router'
import { v4 as uuidv4 } from 'uuid'

const route = useRoute()
const workflowStore = useWorkflowStore()
const nodeStore = useNodeStore()
const { onConnect, addEdges, toObject, setNodes, setEdges, screenToFlowCoordinate } = useVueFlow()

const nodes = ref<any[]>([])
const edges = ref<any[]>([])
const selectedNode = ref<any>(null)
const showNodeCreator = ref(false)

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
  // Calculate drag position taking into account the canvas bounding box and scale
  const position = screenToFlowCoordinate({ x: event.clientX, y: event.clientY }) 
  
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
  <div class="h-full w-full flex overflow-hidden bg-transparent">
    <!-- Main Canvas Area -->
    <div class="flex-1 relative overflow-hidden">
      <!-- Toolbar -->
      <div class="absolute top-4 left-4 right-4 flex justify-between items-center z-10 pointer-events-none">
        <div class="bg-white rounded-lg shadow-sm border border-slate-200 px-4 py-2 flex items-center gap-3 pointer-events-auto">
          <div>
            <h1 class="font-bold text-slate-800 text-base leading-tight">My First Workflow</h1>
            <p class="text-xs text-slate-500">Last saved 2m ago</p>
          </div>
          <div class="h-6 w-px bg-slate-200 mx-1"></div>
          <span class="px-2 py-1 bg-green-100 border border-green-200 text-green-700 text-[10px] font-bold uppercase tracking-wider rounded">Active</span>
        </div>
        
        <div class="flex gap-2 pointer-events-auto">
          <button 
            @click="handleSave"
            class="bg-white hover:bg-slate-50 border border-slate-200 text-slate-700 px-4 py-2 rounded-lg flex items-center gap-2 transition-colors font-semibold text-sm shadow-sm"
          >
            <Save class="w-4 h-4" /> Save
          </button>
          <button 
            @click="handleExecute"
            :disabled="workflowStore.loading"
            class="bg-brand-500 hover:bg-brand-600 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors font-semibold text-sm disabled:opacity-70 shadow-sm"
          >
            <Loader2 v-if="workflowStore.loading" class="w-4 h-4 animate-spin" />
            <Play v-else class="w-4 h-4 fill-current" /> 
            {{ workflowStore.loading ? 'Executing...' : 'Execute Workflow' }}
          </button>
        </div>
      </div>

      <!-- Add Node FAB -->
      <div class="absolute bottom-6 right-6 z-10 pointer-events-auto">
        <button 
          @click="showNodeCreator = true"
          class="w-12 h-12 bg-brand-500 shadow-lg text-white rounded-full flex items-center justify-center hover:bg-brand-600 hover:scale-105 transition-all"
        >
          <Plus class="w-6 h-6" />
        </button>
      </div>

      <!-- Vue Flow Canvas -->
      <div class="h-full w-full bg-[#f8f9fa]" @drop="onDrop" @dragover.prevent>
        <VueFlow
          v-model:nodes="nodes"
          v-model:edges="edges"
          @node-click="onNodeClick"
          :node-types="{ custom: CustomNode }"
          class="n8n-canvas"
          :default-viewport="{ zoom: 1, x: 0, y: 0 }"
          :min-zoom="0.2"
          :max-zoom="2"
        >
          <Background pattern-color="#ccc" :gap="20" />
          <Controls position="bottom-left" class="!bg-white !border-slate-200 !shadow-sm !rounded-md overflow-hidden mb-6 ml-6" />
          <MiniMap class="!bg-white !border-slate-200 !shadow-sm !rounded-md mr-20 mb-6" />
        </VueFlow>
      </div>
    </div>

    <!-- Node Creator Overlay -->
    <NodeCreator 
      :show="showNodeCreator" 
      @close="showNodeCreator = false" 
      @dragstart="onDragStart"
    />

    <!-- Properties Panel Overlay -->
    <NodePanel :node="selectedNode" @close="selectedNode = null" />
  </div>
</template>

<style>
/* n8n grid background */
.n8n-canvas {
  background-image: radial-gradient(#e5e7eb 1px, transparent 1px);
  background-size: 20px 20px;
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
