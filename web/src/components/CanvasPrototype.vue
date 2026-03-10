<template>
  <!-- Deprecated Phase 50 prototype. Keep only as a reference during editor refactors. -->
  <div class="canvas-wrapper" @mousemove="onMouseMove" @mouseup="onMouseUp" @mouseleave="onMouseUp">
    <!-- SVG layer for neural glow wires -->
    <svg class="wires-layer">
      <defs>
        <linearGradient id="neonGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="var(--neon-cyan)" />
          <stop offset="100%" stop-color="var(--neon-purple)" />
        </linearGradient>
        <filter id="glow">
          <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
          <feMerge>
            <feMergeNode in="coloredBlur"/>
            <feMergeNode in="SourceGraphic"/>
          </feMerge>
        </filter>
      </defs>
      
      <!-- Static Wires -->
      <path 
        v-for="(wire, i) in wires" 
        :key="i"
        :d="generateBezierPath(wire.startX, wire.startY, wire.endX, wire.endY)"
        class="wire-path"
      />

      <!-- Active Drawing Wire -->
      <path 
        v-if="isDrawingWire"
        :d="generateBezierPath(drawingWireStart.x, drawingWireStart.y, mousePos.x, mousePos.y)"
        class="wire-drawing"
      />
    </svg>

    <!-- HTML layer for nodes -->
    <div class="nodes-layer">
      <GlassNode
        v-for="node in nodes"
        :key="node.id"
        :title="node.title"
        :description="node.description"
        :x="node.x"
        :y="node.y"
        @dragStart="startDragNode($event, node)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import GlassNode from './NodeEditor/GlassNode.vue'

// Basic mock state
const nodes = ref([
  { id: '1', title: 'Webhook Trigger', description: 'Listens for incoming HTTP POSTs across your custom endpoints.', x: 150, y: 200 },
  { id: '2', title: 'Rhai Sandbox', description: 'Executes your custom scripts securely mapping outputs properly.', x: 550, y: 250 }
])

const wires = ref([
  { startX: 410, startY: 280, endX: 550, endY: 330 } // Mock wire
])

// Drag State
const draggedNode = ref<any>(null)
const dragOffset = ref({ x: 0, y: 0 })

// Wire State
const isDrawingWire = ref(false)
const drawingWireStart = ref({ x: 0, y: 0 })
const mousePos = ref({ x: 0, y: 0 })

const generateBezierPath = (x1: number, y1: number, x2: number, y2: number) => {
  const dx = Math.abs(x2 - x1) * 0.5
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
}

const startDragNode = (event: MouseEvent, node: any) => {
  draggedNode.value = node
  dragOffset.value = {
    x: event.clientX - node.x,
    y: event.clientY - node.y
  }
}

const onMouseMove = (event: MouseEvent) => {
  mousePos.value = { x: event.clientX, y: event.clientY - 60 } // Account for 60px TopNav
  
  if (draggedNode.value) {
    draggedNode.value.x = mousePos.value.x - dragOffset.value.x
    draggedNode.value.y = mousePos.value.y + 60 - dragOffset.value.y
  }
}

const onMouseUp = () => {
  draggedNode.value = null
  isDrawingWire.value = false
}
</script>

<style scoped>
.canvas-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.wires-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
}

.wire-path, .wire-drawing {
  fill: none;
  stroke: url(#neonGradient);
  stroke-width: 3;
  filter: url(#glow);
}

.wire-drawing {
  stroke-dasharray: 8;
  animation: marching-ants 0.5s linear infinite;
}

@keyframes marching-ants {
  to { stroke-dashoffset: -16; }
}

.nodes-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 2;
}
</style>
