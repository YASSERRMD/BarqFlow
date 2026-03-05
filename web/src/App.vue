<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { 
  Menu, 
  Workflow, 
  History, 
  Settings, 
  Puzzle, 
  FolderGit2 
} from 'lucide-vue-next'

const route = useRoute()

const navItems = [
  { name: 'Workflows', path: '/workflows', icon: Workflow },
  { name: 'Executions', path: '/executions', icon: History },
  { name: 'Credentials', path: '/credentials', icon: FolderGit2 },
  { name: 'Settings', path: '/settings', icon: Settings },
]
</script>

<template>
  <div class="flex h-screen w-full bg-canvas text-slate-800 font-sans overflow-hidden">
    
    <!-- Sidebar -->
    <aside class="w-16 md:w-64 bg-white border-r border-slate-200 flex flex-col justify-between transition-all duration-300 z-50 shadow-sm relative">
      <div>
        <div class="h-16 flex items-center justify-center md:justify-start px-0 md:px-6 border-b border-slate-100 mb-4">
          <img src="/logo.png" alt="Logo" class="w-8 h-8 md:w-10 md:h-10" />
          <span class="hidden md:ml-3 md:block text-xl font-bold bg-gradient-to-r from-brand-600 to-purple-600 bg-clip-text text-transparent tracking-tight">BarqFlow</span>
        </div>

        <nav class="flex-1 px-2 space-y-1 mt-4">
          <RouterLink 
            v-for="item in navItems" 
            :key="item.name" 
            :to="item.path"
            :class="[
              route.path.startsWith(item.path) ? 'bg-brand-50 text-brand-700 border-r-4 border-brand-500' : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900',
              'group flex items-center px-2 py-3 md:px-3 text-sm font-medium rounded-l-md transition-colors'
            ]"
          >
            <component :is="item.icon" class="w-5 h-5 mx-auto md:mx-0 md:mr-3 flex-shrink-0" />
            <span class="hidden md:block">{{ item.name }}</span>
          </RouterLink>
        </nav>
      </div>

      <div class="p-4 border-t border-slate-100 flex items-center justify-center md:justify-start">
        <div class="w-8 h-8 rounded-full bg-gradient-to-tr from-brand-500 to-purple-500 flex items-center justify-center text-white font-bold text-xs shadow-md">
          A
        </div>
        <div class="hidden md:block ml-3">
          <p class="text-sm font-semibold text-slate-700 leading-none">Admin User</p>
          <p class="text-xs text-slate-500 mt-1">Workspace ID: local</p>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col min-w-0 bg-canvas overflow-hidden">
      <!-- Top header for mobile / responsive -->
      <header class="h-14 bg-white border-b border-slate-200 flex items-center justify-between px-4 md:hidden shadow-sm z-40">
        <Menu class="w-6 h-6 text-slate-600" />
        <span class="font-bold text-slate-800">BarqFlow</span>
        <div class="w-6"></div>
      </header>

      <!-- Router View Container -->
      <div class="flex-1 relative overflow-hidden h-full">
        <RouterView v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </RouterView>
      </div>
    </main>
  </div>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
