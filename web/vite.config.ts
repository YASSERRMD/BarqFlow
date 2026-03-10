import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) {
            return
          }

          if (id.includes('@vue-flow/')) {
            return 'vue-flow'
          }

          if (id.includes('vue-router') || id.includes('/vue/dist/')) {
            return 'vue-core'
          }

          if (id.includes('pinia')) {
            return 'state'
          }

          if (id.includes('axios')) {
            return 'network'
          }

          if (id.includes('lucide-vue-next')) {
            return 'icons'
          }

          return 'vendor'
        }
      }
    }
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    port: 3001,
    proxy: {
      '/rest': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true
      },
      '/webhook': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true
      }
    }
  }
})
