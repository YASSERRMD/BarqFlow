<template>
  <div class="app-container">
    <header class="top-nav">
      <div class="logo">
        <span class="neon-text">Barq</span>Flow
      </div>
      <div class="nav-actions">
        <button class="glass-btn primary">Deploy</button>
      </div>
    </header>

    <main class="canvas-area">
      <Canvas />
    </main>
  </div>
</template>

<script setup lang="ts">
import Canvas from './components/Canvas.vue';
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}

.top-nav {
  height: 60px;
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-bottom: 1px solid var(--glass-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  z-index: 100;
}

.logo {
  font-family: 'Outfit', sans-serif;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.neon-text {
  color: var(--neon-cyan);
  text-shadow: 0 0 10px rgba(0, 243, 255, 0.5);
}

.glass-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--glass-border);
  color: white;
  padding: 8px 16px;
  border-radius: 8px;
  font-family: 'Inter', sans-serif;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(8px);
}

.glass-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: translateY(-1px);
}

.glass-btn.primary {
  border-color: var(--neon-purple);
  box-shadow: 0 0 15px rgba(157, 0, 255, 0.2);
}

.glass-btn.primary:hover {
  box-shadow: 0 0 25px rgba(157, 0, 255, 0.4);
  background: rgba(157, 0, 255, 0.15);
}

.canvas-area {
  flex: 1;
  position: relative;
  overflow: hidden;
}
</style>
