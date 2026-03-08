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

function buildDefaultProperties(schema: any): Record<string, any> {
  const defaults: Record<string, any> = {}
  if (schema?.properties) {
    schema.properties.forEach((p: any) => {
      if (p.default !== undefined) defaults[p.name] = p.default
    })
  }
  return defaults
}

function findTypeEntry(typeName: string) {
  return nodeStore.nodeTypes.find((n: any) => n.schema?.name === typeName) || null
}

function makeUniqueNodeLabel(baseName: string): string {
  const existing = new Set(nodes.value.map((n: any) => n?.data?.label))
  if (!existing.has(baseName)) return baseName

  let i = 2
  while (existing.has(`${baseName} ${i}`)) i += 1
  return `${baseName} ${i}`
}

function toCanvasNode(inode: any): any {
  const nodeType = inode.type || ''
  const typeEntry = findTypeEntry(nodeType)
  const schema = typeEntry?.schema || null

  const positionArray = Array.isArray(inode.position) ? inode.position : [0, 0]
  const properties = {
    ...buildDefaultProperties(schema),
    ...(inode.parameters || {}),
  }

  return {
    id: inode.id,
    type: 'custom',
    position: {
      x: Number(positionArray[0] ?? 0),
      y: Number(positionArray[1] ?? 0),
    },
    data: {
      type: nodeType,
      kind: typeEntry?.kind || (schema?.is_trigger ? 'trigger' : 'action'),
      isTrigger: !!(typeEntry?.isTrigger || schema?.is_trigger),
      label: inode.name,
      description: typeEntry?.description || schema?.description || '',
      status: null,
      schema,
      properties,
    },
  }
}

function toWorkflowNode(flowNode: any): any {
  const nodeType = flowNode?.data?.schema?.name || flowNode?.data?.type
  return {
    id: String(flowNode.id),
    name: flowNode?.data?.label || String(flowNode.id),
    type: nodeType,
    typeVersion: 1.0,
    position: [
      Number(flowNode?.position?.x ?? 0),
      Number(flowNode?.position?.y ?? 0),
    ],
    parameters: flowNode?.data?.properties || {},
    disabled: false,
  }
}

function buildWorkflowConnections(flowNodes: any[], flowEdges: any[]) {
  const byId = new Map(flowNodes.map((n: any) => [String(n.id), n]))
  const connections: Record<string, { main: any[][] }> = {}

  flowEdges.forEach((edge: any) => {
    const sourceNode = byId.get(String(edge.source))
    const targetNode = byId.get(String(edge.target))
    if (!sourceNode || !targetNode) return

    const sourceName = sourceNode?.data?.label || String(sourceNode.id)
    const targetName = targetNode?.data?.label || String(targetNode.id)
    if (!connections[sourceName]) connections[sourceName] = { main: [[]] }

    connections[sourceName].main[0].push({
      node: targetName,
      type: 'main',
      index: 0,
    })
  })

  return connections
}

function buildCanvasEdges(loadedNodes: any[], rawConnections: any): any[] {
  const byName = new Map(
    loadedNodes.map((n: any) => [n?.data?.label, n]),
  )

  const loadedEdges: any[] = []
  Object.keys(rawConnections || {}).forEach((sourceName) => {
    const sourceConn = rawConnections[sourceName]
    const outputGroups = sourceConn?.main || sourceConn?.Main || []

    outputGroups.forEach((targets: any[]) => {
      ;(targets || []).forEach((target: any) => {
        const sourceNode = byName.get(sourceName)
        const targetNode = byName.get(target?.node)
        if (!sourceNode || !targetNode) return

        loadedEdges.push({
          id: `e-${sourceNode.id}-${targetNode.id}-${loadedEdges.length}`,
          source: sourceNode.id,
          target: targetNode.id,
          animated: true,
          style: { stroke: '#0ea5e9', strokeWidth: 2 },
        })
      })
    })
  })

  return loadedEdges
}

// Load Nodes and Workflow on Mount
onMounted(async () => {
  await nodeStore.fetchNodeTypes()
  if (!(route.params.id && route.params.id !== 'new')) return

  await workflowStore.fetchWorkflow(route.params.id as string)
  const activeWf = workflowStore.activeWorkflow
  if (!activeWf || !Array.isArray(activeWf.nodes)) return

  const loadedNodes = activeWf.nodes.map((n: any) => toCanvasNode(n))
  const loadedEdges = buildCanvasEdges(loadedNodes, activeWf.connections || {})

  setNodes(loadedNodes)
  setEdges(loadedEdges)
  nodes.value = loadedNodes
  edges.value = loadedEdges
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
  if (route.params.id === 'new') return
  
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
  const payloadNodes = flow.nodes.map((n: any) => toWorkflowNode(n))
  const payloadConnections = buildWorkflowConnections(flow.nodes, flow.edges)

  const payloadStr = {
      id: route.params.id !== 'new' ? route.params.id : undefined,
      name: workflowStore.activeWorkflow?.name || 'My New Workflow',
      nodes: payloadNodes,
      connections: payloadConnections,
      settings: workflowStore.activeWorkflow?.settings || {}
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
  
  const typeName = nodeSchema.schema?.name || nodeSchema.type || ''
  const typeEntry = findTypeEntry(typeName)
  const schema = nodeSchema.schema || typeEntry?.schema || null

  // Set default properties based on schema if not exist
  const propertiesObj = buildDefaultProperties(schema)
  const label = makeUniqueNodeLabel(nodeSchema.name || schema?.display_name || typeName)

  const newNode = {
    id: uuidv4(),
    type: 'custom',
    position,
    data: {
      type: typeName,
      kind: typeEntry?.kind || nodeSchema.kind || (schema?.is_trigger ? 'trigger' : 'action'),
      isTrigger: !!(typeEntry?.isTrigger || nodeSchema.isTrigger || schema?.is_trigger),
      label,
      description: nodeSchema.description || schema?.description || '',
      status: null,
      schema,
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
