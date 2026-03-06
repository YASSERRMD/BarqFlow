<template>
  <div 
    class="glass-node" 
    :class="{ selected }" 
    :style="{ transform: `translate(${x}px, ${y}px)` }"
    @mousedown="$emit('dragStart', $event)"
  >
    <div class="node-header">
      <div class="node-icon">
        <!-- Pulse effect for active nodes -->
        <div class="pulse-ring"></div>
        <div class="icon-inner"></div>
      </div>
      <span class="node-title">{{ title }}</span>
    </div>
    
    <div class="node-body">
      <p class="node-desc">{{ description }}</p>
    </div>

    <!-- Output Ports -->
    <div class="node-ports">
      <div class="port output" @mousedown.stop="$emit('portDragStart', $event, 'output')"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  description: string
  x: number
  y: number
  selected?: boolean
}>()

defineEmits(['dragStart', 'portDragStart'])
</script>

<style scoped>
.glass-node {
  position: absolute;
  top: 0;
  left: 0;
  width: 260px;
  background: var(--glass-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  box-shadow: var(--node-shadow);
  color: var(--text-primary);
  font-family: 'Inter', sans-serif;
  cursor: grab;
  transition: box-shadow 0.3s ease, border-color 0.3s ease, transform 0.1s linear;
  z-index: 10;
}

.glass-node:active {
  cursor: grabbing;
}

.glass-node:hover {
  background: var(--glass-bg-hover);
  border-color: rgba(255, 255, 255, 0.2);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
}

.glass-node.selected {
  border-color: var(--neon-cyan);
  box-shadow: 0 0 20px rgba(0, 243, 255, 0.2), inset 0 0 10px rgba(0, 243, 255, 0.1);
}

.node-header {
  padding: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  display: flex;
  align-items: center;
  gap: 12px;
}

.node-icon {
  position: relative;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-inner {
  width: 12px;
  height: 12px;
  background: var(--neon-cyan);
  border-radius: 50%;
  box-shadow: 0 0 10px var(--neon-cyan);
  z-index: 2;
}

.pulse-ring {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 1px solid var(--neon-cyan);
  animation: pulse 2s cubic-bezier(0.2, 0.8, 0.2, 1) infinite;
  z-index: 1;
}

@keyframes pulse {
  0% { transform: translate(-50%, -50%) scale(0.8); opacity: 1; }
  100% { transform: translate(-50%, -50%) scale(2.5); opacity: 0; }
}

.node-title {
  font-family: 'Outfit', sans-serif;
  font-weight: 500;
  font-size: 16px;
  letter-spacing: 0.5px;
}

.node-body {
  padding: 16px;
}

.node-desc {
  margin: 0;
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
}

.node-ports {
  position: absolute;
  right: -6px;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.port {
  width: 12px;
  height: 12px;
  background: var(--bg-surface);
  border: 2px solid var(--neon-purple);
  border-radius: 50%;
  cursor: crosshair;
  transition: all 0.2s ease;
  box-shadow: 0 0 8px rgba(157, 0, 255, 0.4);
}

.port:hover {
  transform: scale(1.3);
  background: var(--neon-purple);
  box-shadow: 0 0 15px rgba(157, 0, 255, 0.8);
}
</style>
