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
    
    <!-- Sidebar (Hidden on Login) -->
    <aside 
      v-if="route.path !== '/login'"
      class="w-16 md:w-64 bg-white border-r border-slate-200 flex flex-col justify-between transition-all duration-300 z-50 shadow-sm relative"
    >
      <div>
        <div class="h-20 flex items-center justify-center md:justify-start px-0 md:px-7 border-b border-slate-100 mb-4">
          <div class="w-10 h-10 bg-brand-50 rounded-xl flex items-center justify-center">
            <img src="/logo.png" alt="Logo" class="w-7 h-7" />
          </div>
          <span class="hidden md:ml-3 md:block text-2xl font-black text-slate-900 tracking-tighter uppercase italic">Barq<span class="text-brand-500">Flow</span></span>
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
      <!-- Top header for mobile / responsive (Hidden on Login) -->
      <header 
        v-if="route.path !== '/login'"
        class="h-16 bg-white border-b border-slate-200 flex items-center justify-between px-6 md:hidden shadow-sm z-40"
      >
        <div class="flex items-center gap-3">
          <img src="/logo.png" alt="Logo" class="w-7 h-7" />
          <span class="font-black text-slate-900 tracking-tighter uppercase italic text-sm">Barq<span class="text-brand-500">Flow</span></span>
        </div>
        <Menu class="w-6 h-6 text-slate-600" />
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
