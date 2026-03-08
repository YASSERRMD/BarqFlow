<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import { 
  Menu, 
  Workflow, 
  History, 
  Settings, 
  FolderGit2,
  LogOut,
} from 'lucide-vue-next'
import { useAuthStore } from './stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const userEmail = computed(() => authStore.user?.email || 'local@workspace')
const userName = computed(() => {
  const name = authStore.user?.email?.split('@')[0]
  return name ? name.charAt(0).toUpperCase() + name.slice(1) : 'User'
})
const userInitial = computed(() => userName.value.charAt(0).toUpperCase())

const navItems = [
  { name: 'Workflows', path: '/workflows', icon: Workflow },
  { name: 'Executions', path: '/executions', icon: History },
  { name: 'Credentials', path: '/credentials', icon: FolderGit2 },
  { name: 'Settings', path: '/settings', icon: Settings },
]

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>

<template>
  <div class="flex h-screen w-full bg-mesh-gradient text-slate-800 overflow-hidden relative">
    
    <!-- Sidebar (Hidden on Login) -->
    <aside 
      v-if="route.path !== '/login'"
      class="hidden md:flex w-64 glass-panel flex-col justify-between transition-all duration-300 z-50 shadow-glass m-4 md:mr-0 rounded-[2rem] overflow-hidden"
    >
      <div>
        <div class="h-24 flex items-center justify-start px-8 border-b border-white/30 backdrop-blur-sm mb-6 pb-2 pt-4">
          <div class="w-12 h-12 bg-gradient-to-br from-brand-400 to-brand-600 rounded-2xl flex items-center justify-center shadow-[0_8px_30px_rgb(14,165,233,0.3)]">
            <img src="/logo.png" alt="Logo" class="w-7 h-7 brightness-0 invert" />
          </div>
          <span class="ml-4 text-3xl font-display font-black tracking-tight text-slate-900 leading-none mt-1">Barq<span class="text-brand-500">Flow</span></span>
        </div>

        <nav class="flex-1 px-4 space-y-2 mt-2">
          <RouterLink 
            v-for="item in navItems" 
            :key="item.name" 
            :to="item.path"
            :class="[
              route.path.startsWith(item.path) ? 'bg-white/60 text-brand-600 shadow-sm border border-white/50 backdrop-blur-md' : 'text-slate-600 hover:bg-white/40 hover:text-slate-900',
              'group flex items-center px-4 py-3.5 text-sm font-bold rounded-2xl transition-all duration-300'
            ]"
          >
            <component :is="item.icon" :class="[route.path.startsWith(item.path) ? 'text-brand-500' : 'text-slate-400 group-hover:text-slate-600', 'w-5 h-5 mr-3 flex-shrink-0 transition-colors']" />
            <span>{{ item.name }}</span>
          </RouterLink>
        </nav>
      </div>

      <div class="p-6 border-t border-white/30 bg-white/30 backdrop-blur-md flex items-center justify-start mt-4">
        <div class="w-10 h-10 rounded-2xl bg-gradient-to-tr from-brand-500 to-purple-500 flex items-center justify-center text-white font-bold text-sm shadow-[0_8px_30px_rgb(14,165,233,0.3)]">
          {{ userInitial }}
        </div>
        <div class="ml-4">
          <p class="text-sm font-bold text-slate-800 leading-none mb-1">{{ userName }}</p>
          <p class="text-[10px] text-slate-500 font-medium tracking-wide">{{ userEmail }}</p>
        </div>
        <button
          @click="handleLogout"
          class="ml-auto p-2 rounded-xl text-slate-500 hover:text-red-600 hover:bg-red-50 transition-colors"
          title="Logout"
        >
          <LogOut class="w-4 h-4" />
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col min-w-0 bg-transparent overflow-hidden">
      <!-- Top header for mobile / responsive (Hidden on Login) -->
      <header 
        v-if="route.path !== '/login'"
        class="h-16 glass-panel border-b border-white/40 flex items-center justify-between px-6 md:hidden z-40"
      >
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 bg-gradient-to-br from-brand-400 to-brand-600 rounded-lg flex items-center justify-center shadow-md">
            <img src="/logo.png" alt="Logo" class="w-5 h-5 brightness-0 invert" />
          </div>
          <span class="font-display font-black text-slate-900 tracking-tight text-lg mt-0.5">Barq<span class="text-brand-500">Flow</span></span>
        </div>
        <Menu class="w-6 h-6 text-slate-600" />
      </header>
      
      <!-- Router View Container -->
      <div class="flex-1 relative overflow-hidden h-full rounded-l-[3rem] md:my-4 mr-4">
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
