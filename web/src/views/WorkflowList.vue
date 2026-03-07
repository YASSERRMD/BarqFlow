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
            type="text" 
            placeholder="Search workflows by name or tags..." 
            class="w-full pl-14 pr-4 py-3.5 bg-white/50 backdrop-blur-sm shadow-inner rounded-xl text-sm focus:ring-4 focus:ring-brand-500/10 focus:border-brand-500 outline-none border border-white transition-all font-medium text-slate-900" 
          />
        </div>
        <div class="flex gap-3">
          <select class="bg-white/50 backdrop-blur-sm border border-white shadow-inner rounded-xl text-sm font-bold text-slate-700 px-6 focus:ring-4 focus:ring-brand-500/10 outline-none transition-all">
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
            <p class="text-slate-500 font-medium line-clamp-2 leading-relaxed">Automate your data sync seamlessly connecting native integrations.</p>
          </div>

          <div class="flex items-center justify-between mt-6 relative z-10 border-t border-slate-100 pt-4">
            <div class="flex items-center gap-4">
              <span class="flex items-center gap-1.5 text-xs font-bold text-slate-400 bg-slate-50 px-2.5 py-1 rounded-lg">
                <Calendar class="w-3.5 h-3.5" /> 2h ago
              </span>
              <span class="px-2.5 py-1 bg-green-100 border border-green-200 text-green-700 text-[10px] font-black rounded-lg uppercase tracking-widest shadow-sm">Active</span>
            </div>
            <div class="flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity duration-300 translate-x-4 group-hover:translate-x-0">
               <button class="w-9 h-9 flex items-center justify-center rounded-xl bg-white border border-slate-100 text-slate-400 hover:text-brand-600 hover:border-brand-200 hover:bg-brand-50 transition-all shadow-sm"><Edit2 class="w-4 h-4" /></button>
               <button class="w-9 h-9 flex items-center justify-center rounded-xl bg-white border border-slate-100 text-slate-400 hover:text-red-500 hover:border-red-200 hover:bg-red-50 transition-all shadow-sm"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
