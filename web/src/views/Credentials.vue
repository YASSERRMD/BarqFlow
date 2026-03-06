<script setup lang="ts">
import { ref } from 'vue'
import { Plus, Search, Shield, Key, Lock, MoreVertical, ExternalLink } from 'lucide-vue-next'

const credentials = ref([
  { id: '1', name: 'My Postgres', type: 'Database', status: 'connected', lastUsed: '5m ago' },
  { id: '2', name: 'Slack Bot Token', type: 'Messaging', status: 'connected', lastUsed: '2h ago' },
  { id: '3', name: 'OpenAI API Key', type: 'AI', status: 'expired', lastUsed: '4d ago' },
])

const categories = ['All', 'Database', 'Messaging', 'AI', 'Marketing', 'Storage']
const activeCategory = ref('All')
</script>

<template>
  <div class="h-full bg-slate-50/50 overflow-auto p-6 md:p-10 text-slate-900">
    <div class="max-w-6xl mx-auto">
      
      <!-- Header -->
      <div class="flex flex-col md:flex-row md:items-end justify-between mb-10 gap-6">
        <div>
          <h1 class="text-4xl font-extrabold text-slate-900 tracking-tight">Credentials</h1>
          <p class="text-slate-500 text-lg mt-2 font-medium">Securely managed keys and OAuth tokens for your integrations.</p>
        </div>
        
        <button 
          class="bg-brand-500 hover:bg-brand-600 text-white px-6 py-3.5 rounded-2xl flex items-center gap-2.5 shadow-xl shadow-brand-500/20 transition-all hover:-translate-y-1 active:translate-y-0 font-bold"
        >
          <Plus class="w-5 h-5" /> Add Credential
        </button>
      </div>

      <!-- Categories & Search -->
      <div class="flex flex-col md:flex-row gap-6 mb-8 items-center">
        <div class="flex gap-2 overflow-x-auto pb-2 w-full md:w-auto">
          <button 
            v-for="cat in categories" 
            :key="cat"
            @click="activeCategory = cat"
            :class="[
              activeCategory === cat ? 'bg-slate-900 text-white shadow-lg' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200',
              'px-5 py-2.5 rounded-xl text-sm font-bold transition-all whitespace-nowrap'
            ]"
          >
            {{ cat }}
          </button>
        </div>
        
        <div class="flex-1 relative group w-full md:w-auto">
          <Search class="w-5 h-5 absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-brand-500 transition-colors" />
          <input 
            type="text" 
            placeholder="Search credentials..." 
            class="w-full pl-12 pr-4 py-3 bg-white border border-slate-200 rounded-2xl text-sm focus:ring-4 focus:ring-brand-500/10 focus:border-brand-500 transition-all font-medium" 
          />
        </div>
      </div>

      <!-- Credentials List -->
      <div class="bg-white/80 backdrop-blur-md border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr class="bg-slate-50/50 border-b border-slate-100">
              <th class="px-8 py-5 text-xs font-bold text-slate-400 uppercase tracking-widest">Name</th>
              <th class="px-8 py-5 text-xs font-bold text-slate-400 uppercase tracking-widest">Type</th>
              <th class="px-8 py-5 text-xs font-bold text-slate-400 uppercase tracking-widest">Status</th>
              <th class="px-8 py-5 text-xs font-bold text-slate-400 uppercase tracking-widest">Last Used</th>
              <th class="px-8 py-5 text-xs font-bold text-slate-400 uppercase tracking-widest text-right">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr v-for="cred in credentials" :key="cred.id" class="hover:bg-slate-50/50 transition-colors group">
              <td class="px-8 py-6">
                <div class="flex items-center gap-4">
                  <div class="w-10 h-10 bg-slate-100 rounded-xl flex items-center justify-center text-slate-500 group-hover:bg-brand-50 group-hover:text-brand-600 transition-colors">
                    <Key v-if="cred.type !== 'Database'" class="w-5 h-5" />
                    <Shield v-else class="w-5 h-5" />
                  </div>
                  <span class="font-bold text-slate-800 group-hover:text-brand-600 transition-colors">{{ cred.name }}</span>
                </div>
              </td>
              <td class="px-8 py-6">
                <span class="text-sm font-bold text-slate-500 bg-slate-100 px-3 py-1 rounded-lg">{{ cred.type }}</span>
              </td>
              <td class="px-8 py-6">
                <div class="flex items-center gap-2">
                  <div :class="[
                    'w-2 h-2 rounded-full',
                    cred.status === 'connected' ? 'bg-green-500' : 'bg-red-500'
                  ]"></div>
                  <span :class="[
                    'text-sm font-bold capitalize',
                    cred.status === 'connected' ? 'text-green-600' : 'text-red-600'
                  ]">{{ cred.status }}</span>
                </div>
              </td>
              <td class="px-8 py-6">
                <span class="text-sm font-medium text-slate-400">{{ cred.lastUsed }}</span>
              </td>
              <td class="px-8 py-6 text-right">
                <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button class="p-2 text-slate-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-all"><ExternalLink class="w-4 h-4" /></button>
                  <button class="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"><Trash2 class="w-4 h-4" /></button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Info Box -->
      <div class="mt-8 bg-brand-50 border border-brand-100 rounded-2xl p-6 flex gap-4 items-start">
        <div class="w-10 h-10 bg-brand-100 rounded-xl flex items-center justify-center text-brand-600 shrink-0">
          <Lock class="w-5 h-5" />
        </div>
        <div>
          <h4 class="font-bold text-brand-900">Bank-grade security</h4>
          <p class="text-sm text-brand-700/80 mt-1 font-medium leading-relaxed">All credentials are encrypted using AES-256-GCM before being stored. Your secrets never leave the server in plain text.</p>
        </div>
      </div>

    </div>
  </div>
</template>
