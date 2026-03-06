<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, Search, MoreVertical, Play, Calendar, Trash2, Edit2 } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { useWorkflowStore } from '../stores/workflows'

const router = useRouter()
const workflowStore = useWorkflowStore()

onMounted(async () => {
  await workflowStore.fetchWorkflows()
})

function editWorkflow(id: string) {
  router.push(`/workflow/${id}`)
}

async function createWorkflow() {
  // const newWf = await workflowStore.saveWorkflow({ name: 'Untitled Workflow', nodes: [], connections: {} })
  // router.push(`/workflow/${newWf.id}`)
  router.push('/workflow/new')
}
</script>

<template>
  <div class="h-full bg-slate-50/50 overflow-auto p-6 md:p-10">
    <div class="max-w-6xl mx-auto">
      
      <!-- Header Section -->
      <div class="flex flex-col md:flex-row md:items-end justify-between mb-10 gap-6">
        <div>
          <h1 class="text-4xl font-extrabold text-slate-900 tracking-tight">Workflows</h1>
          <p class="text-slate-500 text-lg mt-2 font-medium">Automate your processes with powerful visual flows.</p>
        </div>
        
        <button 
          @click="createWorkflow"
          class="bg-brand-500 hover:bg-brand-600 text-white px-6 py-3.5 rounded-2xl flex items-center gap-2.5 shadow-xl shadow-brand-500/20 transition-all hover:-translate-y-1 active:translate-y-0 font-bold"
        >
          <Plus class="w-5 h-5" /> New Workflow
        </button>
      </div>

      <!-- Search and Filter Bar -->
      <div class="bg-white/80 backdrop-blur-md p-3 border border-slate-200 rounded-2xl mb-8 flex flex-col md:flex-row gap-3 shadow-sm">
        <div class="flex-1 relative group">
          <Search class="w-5 h-5 absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-brand-500 transition-colors" />
          <input 
            type="text" 
            placeholder="Search workflows by name or tags..." 
            class="w-full pl-12 pr-4 py-3 bg-slate-50 border-none rounded-xl text-sm focus:ring-2 focus:ring-brand-500/20 transition-all font-medium" 
          />
        </div>
        <div class="flex gap-2">
          <select class="bg-slate-50 border-none rounded-xl text-sm font-bold text-slate-600 px-4 focus:ring-2 focus:ring-brand-500/20">
            <option>All Workflows</option>
            <option>Active</option>
            <option>Inactive</option>
          </select>
        </div>
      </div>

      <!-- Workflows Grid -->
      <div v-if="workflowStore.loading" class="flex flex-col items-center justify-center py-20 grayscale opacity-50">
        <Loader2 class="w-10 h-10 animate-spin text-brand-500 mb-4" />
        <p class="font-bold text-slate-400">Loading your workflows...</p>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <!-- New Workflow Card -->
        <div 
          @click="createWorkflow"
          class="group border-2 border-dashed border-slate-200 rounded-3xl p-8 flex flex-col items-center justify-center gap-4 hover:border-brand-300 hover:bg-brand-50/30 transition-all cursor-pointer h-[240px]"
        >
          <div class="w-14 h-14 rounded-2xl bg-slate-50 flex items-center justify-center text-slate-400 group-hover:bg-brand-100 group-hover:text-brand-600 transition-all group-hover:scale-110">
            <Plus class="w-8 h-8" />
          </div>
          <p class="font-bold text-slate-500 group-hover:text-brand-700 transition-colors">Create new workflow</p>
        </div>

        <!-- Workflow Cards -->
        <div 
          v-for="wf in workflowStore.workflows" 
          :key="wf.id"
          class="bg-white border border-slate-200 rounded-3xl p-6 shadow-sm hover:shadow-xl hover:shadow-slate-200/50 transition-all hover:-translate-y-1 cursor-pointer flex flex-col justify-between group h-[240px]"
          @click="editWorkflow(wf.id)"
        >
          <div>
            <div class="flex items-start justify-between mb-4">
              <div class="w-12 h-12 bg-indigo-50 text-indigo-600 rounded-2xl flex items-center justify-center group-hover:bg-indigo-600 group-hover:text-white transition-all">
                <Workflow class="w-6 h-6" />
              </div>
              <button class="text-slate-300 hover:text-slate-600 p-1">
                <MoreVertical class="w-5 h-5" />
              </button>
            </div>
            
            <h3 class="text-xl font-bold text-slate-900 mb-1 truncate">{{ wf.name }}</h3>
            <p class="text-slate-400 text-sm font-medium line-clamp-2">Automate your data sync between Postgres and Slack.</p>
          </div>

          <div class="flex items-center justify-between mt-6">
            <div class="flex items-center gap-3">
              <span class="flex items-center gap-1.5 text-xs font-bold text-slate-400">
                <Calendar class="w-3.5 h-3.5" /> 2h ago
              </span>
              <span class="px-2 py-0.5 bg-green-100 text-green-700 text-[10px] font-bold rounded-lg uppercase tracking-wider">Active</span>
            </div>
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
               <button class="p-2 text-slate-400 hover:text-brand-600 transition-colors"><Edit2 class="w-4 h-4" /></button>
               <button class="p-2 text-slate-400 hover:text-red-500 transition-colors"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
