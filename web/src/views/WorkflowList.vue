<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Plus, Search, MoreVertical, Calendar, Trash2, Edit2, Loader2, Workflow, Power, Copy } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { useWorkflowStore } from '../stores/workflows'
import api from '../api'

const router = useRouter()
const workflowStore = useWorkflowStore()
const searchTerm = ref('')
const statusFilter = ref('all')
let searchTimer: ReturnType<typeof setTimeout> | null = null
const latestExecutionByWorkflow = ref<Record<string, any>>({})

function formatRelativeTime(iso?: string | null): string {
  if (!iso) return 'Unknown'

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return 'Unknown'

  const diffSeconds = Math.floor((date.getTime() - Date.now()) / 1000)
  const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })
  const absSeconds = Math.abs(diffSeconds)

  if (absSeconds < 60) return rtf.format(diffSeconds, 'second')

  const diffMinutes = Math.floor(diffSeconds / 60)
  if (Math.abs(diffMinutes) < 60) return rtf.format(diffMinutes, 'minute')

  const diffHours = Math.floor(diffMinutes / 60)
  if (Math.abs(diffHours) < 24) return rtf.format(diffHours, 'hour')

  const diffDays = Math.floor(diffHours / 24)
  if (Math.abs(diffDays) < 30) return rtf.format(diffDays, 'day')

  return date.toLocaleDateString()
}

function workflowRelativeUpdatedAt(wf: any): string {
  return formatRelativeTime(wf?.updated_at || wf?.created_at)
}

function workflowTimestampTitle(wf: any): string {
  const iso = wf?.updated_at || wf?.created_at
  if (!iso) return 'Unknown'

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return 'Unknown'
  return date.toLocaleString()
}

function buildListParams() {
  return {
    search: searchTerm.value.trim() || undefined,
    active:
      statusFilter.value === 'all'
        ? undefined
        : statusFilter.value === 'active',
    limit: 200,
  }
}

async function fetchWorkflows() {
  await workflowStore.fetchWorkflows(buildListParams())
}

async function fetchExecutionMetadata() {
  try {
    const response = await api.get('/executions', { params: { limit: 500 } })
    const executions = Array.isArray(response.data) ? response.data : []
    const nextMap: Record<string, any> = {}

    for (const exec of executions) {
      const workflowId = String(exec?.workflow_id || '')
      if (!workflowId) continue

      const current = nextMap[workflowId]
      const currentTs = current?.started_at
        ? new Date(current.started_at).getTime()
        : Number.NEGATIVE_INFINITY
      const candidateTs = exec?.started_at
        ? new Date(exec.started_at).getTime()
        : Number.NEGATIVE_INFINITY

      if (!current || candidateTs >= currentTs) {
        nextMap[workflowId] = exec
      }
    }

    latestExecutionByWorkflow.value = nextMap
  } catch (err) {
    console.error('Failed to fetch executions metadata', err)
  }
}

function workflowNodeCount(wf: any): number {
  const nodes = wf?.nodes
  if (Array.isArray(nodes)) return nodes.length

  if (typeof nodes === 'string') {
    try {
      const parsed = JSON.parse(nodes)
      if (Array.isArray(parsed)) return parsed.length
    } catch {
      return 0
    }
  }

  return 0
}

function nodeCountLabel(wf: any): string {
  const count = workflowNodeCount(wf)
  return `${count} node${count === 1 ? '' : 's'}`
}

function workflowLastExecutionIso(wf: any): string | null {
  const execution = latestExecutionByWorkflow.value[String(wf?.id || '')]
  return execution?.started_at || execution?.stopped_at || null
}

function workflowLastExecutionLabel(wf: any): string {
  const iso = workflowLastExecutionIso(wf)
  return iso ? formatRelativeTime(iso) : 'No runs yet'
}

function workflowLastExecutionTitle(wf: any): string {
  const iso = workflowLastExecutionIso(wf)
  if (!iso) return 'No runs yet'

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return 'No runs yet'
  return date.toLocaleString()
}

onMounted(async () => {
  await Promise.all([fetchWorkflows(), fetchExecutionMetadata()])
})

watch([searchTerm, statusFilter], () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    fetchWorkflows()
  }, 250)
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
})

function editWorkflow(id: string) {
  router.push(`/workflow/${id}`)
}

async function deleteWorkflow(id: string) {
  const confirmed = window.confirm('Delete this workflow permanently?')
  if (!confirmed) return

  try {
    await workflowStore.deleteWorkflow(id)
    await fetchExecutionMetadata()
  } catch (err) {
    console.error('Failed to delete workflow', err)
  }
}

async function toggleWorkflowActive(id: string, current: boolean) {
  try {
    await workflowStore.toggleWorkflowActive(id, !current)
    await fetchExecutionMetadata()
    if (statusFilter.value !== 'all') {
      await fetchWorkflows()
    }
  } catch (err) {
    console.error('Failed to update workflow activation', err)
  }
}

async function duplicateWorkflow(id: string) {
  try {
    const duplicated = await workflowStore.duplicateWorkflow(id)
    await fetchExecutionMetadata()
    router.push(`/workflow/${duplicated.id}`)
  } catch (err) {
    console.error('Failed to duplicate workflow', err)
  }
}

async function createWorkflow() {
  // const newWf = await workflowStore.saveWorkflow({ name: 'Untitled Workflow', nodes: [], connections: {} })
  // router.push(`/workflow/${newWf.id}`)
  router.push('/workflow/new')
}
</script>

<template>
  <div class="h-full bg-transparent overflow-auto p-6 md:p-10">
    <div class="max-w-6xl mx-auto pt-6">
      
      <!-- Header Section -->
      <div class="flex flex-col md:flex-row md:items-end justify-between mb-12 gap-6">
        <div>
          <h1 class="text-5xl font-display font-black text-slate-900 tracking-tight leading-tight">Workflows</h1>
          <p class="text-slate-600 text-lg mt-3 font-medium">Automate your processes with powerful visual flows.</p>
        </div>
        
        <button 
          @click="createWorkflow"
          class="bg-gradient-to-r from-brand-500 to-brand-600 hover:from-brand-600 hover:to-brand-700 text-white px-8 py-4 rounded-[1.25rem] flex items-center gap-3 shadow-[0_8px_30px_rgb(14,165,233,0.3)] transition-all duration-300 hover:-translate-y-1 active:translate-y-0 font-bold text-lg"
        >
          <Plus class="w-5 h-5" /> New Workflow
        </button>
      </div>

      <!-- Search and Filter Bar -->
      <div class="glass-panel p-4 border border-white/60 rounded-[1.5rem] mb-10 flex flex-col md:flex-row gap-4">
        <div class="flex-1 relative group">
          <Search class="w-5 h-5 absolute left-5 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-brand-500 transition-colors" />
          <input 
            v-model="searchTerm"
            type="text" 
            placeholder="Search workflows by name or tags..." 
            class="w-full pl-14 pr-4 py-3.5 bg-white/50 backdrop-blur-sm shadow-inner rounded-xl text-sm focus:ring-4 focus:ring-brand-500/10 focus:border-brand-500 outline-none border border-white transition-all font-medium text-slate-900" 
          />
        </div>
        <div class="flex gap-3">
          <select v-model="statusFilter" class="bg-white/50 backdrop-blur-sm border border-white shadow-inner rounded-xl text-sm font-bold text-slate-700 px-6 focus:ring-4 focus:ring-brand-500/10 outline-none transition-all">
            <option value="all">All Workflows</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </div>
      </div>

      <!-- Workflows Grid -->
      <div v-if="workflowStore.loading" class="flex flex-col items-center justify-center py-20 grayscale opacity-50">
        <Loader2 class="w-10 h-10 animate-spin text-brand-500 mb-4" />
        <p class="font-bold text-slate-400">Loading your workflows...</p>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        <!-- New Workflow Card -->
        <div 
          @click="createWorkflow"
          class="group glass-panel border-2 border-dashed border-slate-300 rounded-[2rem] p-8 flex flex-col items-center justify-center gap-5 hover:border-brand-400 hover:bg-brand-50/50 transition-all duration-300 cursor-pointer h-[260px] hover:shadow-[0_20px_40px_rgba(14,165,233,0.1)] hover:-translate-y-2"
        >
          <div class="w-16 h-16 rounded-[1.25rem] bg-white border border-slate-200 shadow-sm flex items-center justify-center text-slate-400 group-hover:bg-brand-100 group-hover:text-brand-600 transition-all duration-500 group-hover:scale-110">
            <Plus class="w-8 h-8" />
          </div>
          <p class="font-bold text-slate-500 group-hover:text-brand-700 transition-colors text-lg">Create new workflow</p>
        </div>

        <!-- Workflow Cards -->
        <div 
          v-for="wf in workflowStore.workflows" 
          :key="wf.id"
          class="glass-card rounded-[2rem] p-8 hover:shadow-node-hover transition-all duration-300 hover:-translate-y-2 cursor-pointer flex flex-col justify-between group h-[260px] relative overflow-hidden"
          @click="editWorkflow(wf.id)"
        >
          <!-- Gradient Hover Glow -->
          <div class="absolute inset-x-0 -top-10 h-20 bg-gradient-to-b from-brand-300/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"></div>
          
          <div class="relative z-10">
            <div class="flex items-start justify-between mb-5">
              <div class="w-14 h-14 bg-indigo-50 border border-indigo-100/50 text-indigo-600 rounded-2xl flex items-center justify-center group-hover:bg-indigo-600 group-hover:text-white group-hover:shadow-[0_8px_20px_rgb(79,70,229,0.3)] transition-all duration-300">
                <Workflow class="w-7 h-7" />
              </div>
              <button class="text-slate-300 hover:text-slate-600 p-2 bg-white/50 hover:bg-white rounded-xl transition-colors shadow-sm">
                <MoreVertical class="w-5 h-5" />
              </button>
            </div>
            
            <h3 class="text-2xl font-display font-bold text-slate-900 mb-2 truncate group-hover:text-brand-600 transition-colors">{{ wf.name }}</h3>
            <p class="text-slate-500 font-medium leading-relaxed">
              {{ nodeCountLabel(wf) }} configured.
            </p>
            <p class="text-slate-400 text-sm font-medium mt-1" :title="workflowLastExecutionTitle(wf)">
              Last run: {{ workflowLastExecutionLabel(wf) }}
            </p>
          </div>

          <div class="flex items-center justify-between mt-6 relative z-10 border-t border-slate-100 pt-4">
            <div class="flex items-center gap-4">
              <span class="flex items-center gap-1.5 text-xs font-bold text-slate-400 bg-slate-50 px-2.5 py-1 rounded-lg">
                <Calendar class="w-3.5 h-3.5" />
                <span :title="workflowTimestampTitle(wf)">
                  {{ workflowRelativeUpdatedAt(wf) }}
                </span>
              </span>
              <span
                :class="[
                  'px-2.5 py-1 text-[10px] font-black rounded-lg uppercase tracking-widest shadow-sm border',
                  wf.active
                    ? 'bg-green-100 border-green-200 text-green-700'
                    : 'bg-slate-100 border-slate-200 text-slate-600'
                ]"
              >
                {{ wf.active ? 'Active' : 'Inactive' }}
              </span>
            </div>
            <div class="flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 translate-x-4 group-hover:translate-x-0">
               <button @click.stop="duplicateWorkflow(wf.id)" class="w-9 h-9 flex items-center justify-center rounded-xl bg-white border border-slate-100 text-slate-400 hover:text-sky-600 hover:border-sky-200 hover:bg-sky-50 transition-all shadow-sm"><Copy class="w-4 h-4" /></button>
               <button @click.stop="toggleWorkflowActive(wf.id, wf.active)" class="w-9 h-9 flex items-center justify-center rounded-xl bg-white border border-slate-100 text-slate-400 hover:text-amber-600 hover:border-amber-200 hover:bg-amber-50 transition-all shadow-sm"><Power class="w-4 h-4" /></button>
               <button @click.stop="editWorkflow(wf.id)" class="w-9 h-9 flex items-center justify-center rounded-xl bg-white border border-slate-100 text-slate-400 hover:text-brand-600 hover:border-brand-200 hover:bg-brand-50 transition-all shadow-sm"><Edit2 class="w-4 h-4" /></button>
               <button @click.stop="deleteWorkflow(wf.id)" class="w-9 h-9 flex items-center justify-center rounded-xl bg-white border border-slate-100 text-slate-400 hover:text-red-500 hover:border-red-200 hover:bg-red-50 transition-all shadow-sm"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
