<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import {
  FolderGit2,
  History,
  LogOut,
  Menu,
  Settings,
  Workflow,
  X,
} from 'lucide-vue-next'
import { useAuthStore } from './stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const mobileNavOpen = ref(false)

const userEmail = computed(() => authStore.user?.email || 'local@workspace')
const userName = computed(() => authStore.userName)
const userInitial = computed(() => userName.value.charAt(0).toUpperCase())
const activeWorkspaceName = computed(() => authStore.activeWorkspace?.name || 'Workspace')
const activeWorkspaceRole = computed(() => authStore.user?.workspaceRole || 'viewer')
const isAuthScreen = computed(() => route.path === '/login')

const navItems = [
  {
    name: 'Workflow Operations',
    path: '/workflows',
    icon: Workflow,
    description: 'Catalog, templates, and rollout controls',
    matches: (path: string) => path.startsWith('/workflows') || path.startsWith('/workflow/'),
    pageTitle: 'Workflow Operations',
    pageDescription: 'Manage workflow inventory, publishing, and authoring surfaces from a single control plane.',
  },
  {
    name: 'Execution Monitor',
    path: '/executions',
    icon: History,
    description: 'Run history, retries, and event timelines',
    matches: (path: string) => path.startsWith('/executions'),
    pageTitle: 'Execution Monitor',
    pageDescription: 'Inspect workflow runs, recovery paths, and execution telemetry.',
  },
  {
    name: 'Credential Operations',
    path: '/credentials',
    icon: FolderGit2,
    description: 'Secrets, OAuth bindings, and lifecycle state',
    matches: (path: string) => path.startsWith('/credentials'),
    pageTitle: 'Credential Operations',
    pageDescription: 'Govern reusable credentials, validation state, and integration handoffs.',
  },
  {
    name: 'Platform Settings',
    path: '/settings',
    icon: Settings,
    description: 'Runtime posture and instance configuration',
    matches: (path: string) => path.startsWith('/settings'),
    pageTitle: 'Platform Settings',
    pageDescription: 'Review environment posture, encryption state, and runtime capacity.',
  },
]

const currentNavigationItem = computed(() => {
  return navItems.find((item) => item.matches(route.path)) || null
})

const headerTitle = computed(() => {
  if (route.path.startsWith('/workflow/')) return 'Workflow Designer'
  return currentNavigationItem.value?.pageTitle || 'BarqFlow Platform'
})

const headerDescription = computed(() => {
  if (route.path.startsWith('/workflow/')) {
    return 'Author, validate, and execute production workflows with a structured node detail view.'
  }

  return currentNavigationItem.value?.pageDescription || 'Automation orchestration control plane.'
})

function isActiveNav(path: string) {
  return currentNavigationItem.value?.path === path
}

function handleLogout() {
  mobileNavOpen.value = false
  authStore.logout()
  router.push('/login')
}

watch(
  () => route.fullPath,
  () => {
    mobileNavOpen.value = false
  },
)
</script>

<template>
  <div v-if="isAuthScreen" class="min-h-screen bg-mesh-gradient text-slate-900">
    <RouterView v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </RouterView>
  </div>

  <div v-else class="flex h-screen w-full overflow-hidden bg-mesh-gradient text-slate-900">
    <aside class="hidden h-screen w-72 shrink-0 flex-col border-r border-slate-200/80 bg-slate-950 text-white lg:flex">
      <div class="border-b border-white/10 px-6 py-6">
        <div class="flex items-center gap-4">
          <div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-white/10 ring-1 ring-white/15">
            <img src="/logo.png" alt="BarqFlow" class="h-7 w-7 brightness-0 invert" />
          </div>
          <div>
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-400">Automation Platform</p>
            <p class="mt-1 font-display text-2xl font-bold tracking-tight text-white">BarqFlow</p>
          </div>
        </div>
        <p class="mt-5 text-sm leading-6 text-slate-300">
          Control workflow operations, credential governance, and runtime execution from one product surface.
        </p>
      </div>

      <nav class="flex-1 space-y-2 px-4 py-6">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          :class="[
            'group flex items-start gap-3 rounded-2xl px-4 py-3.5 transition',
            isActiveNav(item.path)
              ? 'bg-white/10 text-white ring-1 ring-white/10'
              : 'text-slate-300 hover:bg-white/6 hover:text-white',
          ]"
        >
          <component
            :is="item.icon"
            :class="[
              'mt-0.5 h-5 w-5 shrink-0 transition',
              isActiveNav(item.path) ? 'text-sky-300' : 'text-slate-500 group-hover:text-slate-200',
            ]"
          />
          <div class="min-w-0">
            <p class="text-sm font-bold">{{ item.name }}</p>
            <p class="mt-1 text-xs leading-5" :class="isActiveNav(item.path) ? 'text-slate-300' : 'text-slate-400'">
              {{ item.description }}
            </p>
          </div>
        </RouterLink>
      </nav>

      <div class="border-t border-white/10 px-6 py-5">
        <div class="flex items-center gap-3 rounded-2xl bg-white/5 px-4 py-3 ring-1 ring-white/10">
          <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-sky-500/15 text-sm font-bold text-sky-200">
            {{ userInitial }}
          </div>
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-semibold text-white">{{ userName }}</p>
            <p class="truncate text-xs text-slate-400">{{ userEmail }}</p>
          </div>
          <button
            class="rounded-xl p-2 text-slate-400 transition hover:bg-white/8 hover:text-white"
            title="Logout"
            @click="handleLogout"
          >
            <LogOut class="h-4 w-4" />
          </button>
        </div>
      </div>
    </aside>

    <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <header class="border-b border-slate-200/80 bg-white/92 backdrop-blur-sm">
        <div class="flex items-center justify-between gap-4 px-4 py-4 md:px-6 lg:px-8">
          <div class="flex min-w-0 items-center gap-3">
            <button
              class="inline-flex h-10 w-10 items-center justify-center rounded-2xl border border-slate-200 bg-white text-slate-700 shadow-sm transition hover:bg-slate-50 lg:hidden"
              @click="mobileNavOpen = true"
            >
              <Menu class="h-5 w-5" />
            </button>
            <div class="min-w-0">
              <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">BarqFlow Platform</p>
              <h1 class="truncate text-2xl font-display font-bold text-slate-950">{{ headerTitle }}</h1>
              <p class="mt-1 hidden max-w-3xl text-sm text-slate-600 md:block">
                {{ headerDescription }}
              </p>
            </div>
          </div>

          <div class="hidden items-center gap-3 sm:flex">
            <div class="rounded-2xl border border-slate-200 bg-white px-4 py-2 shadow-sm">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500">Workspace</p>
              <p class="mt-1 text-sm font-semibold text-slate-900">{{ activeWorkspaceName }}</p>
              <p class="text-xs text-slate-500 capitalize">{{ activeWorkspaceRole }}</p>
            </div>
            <button
              class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50"
              @click="handleLogout"
            >
              <LogOut class="h-4 w-4" />
              Sign Out
            </button>
          </div>
        </div>
      </header>

      <main class="min-h-0 flex-1 overflow-hidden">
        <RouterView v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </RouterView>
      </main>
    </div>

    <transition name="fade">
      <div
        v-if="mobileNavOpen"
        class="fixed inset-0 z-40 bg-slate-950/45 backdrop-blur-[2px] lg:hidden"
        @click="mobileNavOpen = false"
      ></div>
    </transition>

    <aside
      class="fixed inset-y-0 left-0 z-50 flex w-[20rem] max-w-[88vw] flex-col border-r border-slate-200/80 bg-white shadow-2xl transition-transform duration-200 lg:hidden"
      :class="mobileNavOpen ? 'translate-x-0' : '-translate-x-full'"
    >
      <div class="flex items-center justify-between border-b border-slate-200 px-5 py-5">
        <div class="flex items-center gap-3">
          <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-slate-950 text-white">
            <img src="/logo.png" alt="BarqFlow" class="h-6 w-6 brightness-0 invert" />
          </div>
          <div>
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Automation Platform</p>
            <p class="font-display text-xl font-bold text-slate-950">BarqFlow</p>
          </div>
        </div>
        <button
          class="rounded-xl p-2 text-slate-500 transition hover:bg-slate-100 hover:text-slate-900"
          @click="mobileNavOpen = false"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <nav class="flex-1 space-y-2 px-4 py-5">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          :class="[
            'flex items-start gap-3 rounded-2xl border px-4 py-3 transition',
            isActiveNav(item.path)
              ? 'border-slate-950 bg-slate-950 text-white'
              : 'border-slate-200 bg-white text-slate-700 hover:bg-slate-50',
          ]"
        >
          <component :is="item.icon" class="mt-0.5 h-5 w-5 shrink-0" />
          <div>
            <p class="text-sm font-bold">{{ item.name }}</p>
            <p class="mt-1 text-xs leading-5" :class="isActiveNav(item.path) ? 'text-slate-300' : 'text-slate-500'">
              {{ item.description }}
            </p>
          </div>
        </RouterLink>
      </nav>

      <div class="border-t border-slate-200 px-5 py-5">
        <div class="mb-4 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
          <p class="text-sm font-semibold text-slate-900">{{ userName }}</p>
          <p class="mt-1 text-xs text-slate-500">{{ userEmail }}</p>
        </div>
        <button
          class="inline-flex w-full items-center justify-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800"
          @click="handleLogout"
        >
          <LogOut class="h-4 w-4" />
          Sign Out
        </button>
      </div>
    </aside>
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
