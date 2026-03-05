<script setup lang="ts">
import { ref } from 'vue'
import { Clock, CheckCircle2, XCircle, Search, Filter } from 'lucide-vue-next'

const executions = ref([
  { id: 'exec_1234abc', workflow: 'Daily Sync', status: 'success', duration: '142ms', time: '2 mins ago' },
  { id: 'exec_5678def', workflow: 'User Onboarding', status: 'error', duration: '3.1s', time: '1 hour ago' },
  { id: 'exec_9012ghi', workflow: 'Payment Webhook', status: 'success', duration: '98ms', time: '3 hours ago' },
  { id: 'exec_3456jkl', workflow: 'Daily Sync', status: 'success', duration: '135ms', time: 'Yesterday' },
])
</script>

<template>
  <div class="h-full bg-slate-50 overflow-auto p-4 md:p-8">
    <div class="max-w-6xl mx-auto">
      
      <div class="flex flex-col md:flex-row md:items-center justify-between mb-8 gap-4">
        <div>
          <h1 class="text-2xl font-bold text-slate-900">Execution History</h1>
          <p class="text-slate-500 text-sm mt-1">Review your recent workflow runs and troubleshoot errors.</p>
        </div>
        
        <div class="flex items-center gap-3">
          <div class="relative">
            <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
            <input type="text" placeholder="Search executions..." class="pl-9 pr-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-brand-500 focus:border-transparent w-full md:w-64" />
          </div>
          <button class="bg-white border border-slate-200 text-slate-600 px-3 py-2 rounded-lg hover:bg-slate-50 flex items-center gap-2 text-sm font-medium transition-colors">
            <Filter class="w-4 h-4" /> Filter
          </button>
        </div>
      </div>

      <!-- Executions List -->
      <div class="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
        <ul class="divide-y divide-slate-100">
          <li v-for="exec in executions" :key="exec.id" class="p-5 hover:bg-slate-50 transition-colors cursor-pointer group">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-4">
                <div :class="[
                  'w-10 h-10 rounded-full flex items-center justify-center shrink-0',
                  exec.status === 'success' ? 'bg-green-100 text-green-600' : 'bg-red-100 text-red-600'
                ]">
                  <CheckCircle2 v-if="exec.status === 'success'" class="w-5 h-5" />
                  <XCircle v-else class="w-5 h-5" />
                </div>
                
                <div>
                  <h3 class="font-semibold text-slate-800 text-base group-hover:text-brand-600 transition-colors">{{ exec.workflow }}</h3>
                  <div class="flex items-center text-xs text-slate-500 mt-1 gap-3">
                    <span class="font-mono text-slate-400">#{{ exec.id.substring(0, 8) }}</span>
                    <span class="flex items-center gap-1"><Clock class="w-3 h-3" /> {{ exec.duration }}</span>
                  </div>
                </div>
              </div>
              
              <div class="text-right">
                <span class="text-sm text-slate-500 block">{{ exec.time }}</span>
                <span :class="[
                  'inline-block mt-1 text-xs font-semibold px-2 py-0.5 rounded-full',
                  exec.status === 'success' ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
                ]">
                  {{ exec.status === 'success' ? 'Completed' : 'Failed' }}
                </span>
              </div>
            </div>
          </li>
        </ul>
      </div>

    </div>
  </div>
</template>
