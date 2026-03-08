<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RefreshCw, ShieldCheck, Clock3, Workflow, KeyRound } from 'lucide-vue-next'
import api from '../api'

interface RuntimeSettings {
  server_time: string
  environment: string
  node_types_count: number
  credential_types_count: number
  encryption_key_configured: boolean
}

const runtime = ref<RuntimeSettings | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

const formattedServerTime = computed(() => {
  if (!runtime.value?.server_time) return '-'
  return new Date(runtime.value.server_time).toLocaleString()
})

async function fetchRuntimeSettings() {
  loading.value = true
  error.value = null
  try {
    const response = await api.get('/settings/runtime')
    runtime.value = response.data
  } catch (err: any) {
    error.value = err?.response?.data || err?.message || 'Failed to load runtime settings'
  } finally {
    loading.value = false
  }
}

onMounted(fetchRuntimeSettings)
</script>

<template>
  <div class="h-full bg-slate-50 overflow-auto p-6 md:p-10">
    <div class="max-w-4xl mx-auto space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold text-slate-900">Settings</h1>
          <p class="text-slate-500 text-sm mt-1">
            Runtime and security state for this BarqFlow instance.
          </p>
        </div>
        <button
          @click="fetchRuntimeSettings"
          :disabled="loading"
          class="inline-flex items-center gap-2 px-3 py-2 rounded-lg bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 disabled:opacity-60"
        >
          <RefreshCw class="w-4 h-4" />
          Refresh
        </button>
      </div>

      <div
        v-if="loading"
        class="bg-white border border-slate-200 rounded-2xl p-6 text-sm text-slate-500"
      >
        Loading runtime settings...
      </div>

      <div
        v-else-if="error"
        class="bg-red-50 border border-red-200 text-red-700 rounded-2xl p-4 text-sm"
      >
        {{ error }}
      </div>

      <div v-else-if="runtime" class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="bg-white border border-slate-200 rounded-2xl p-5">
          <div class="flex items-center gap-2 text-slate-500 text-sm font-medium">
            <Clock3 class="w-4 h-4" />
            Server time
          </div>
          <p class="text-slate-900 text-lg font-semibold mt-2">{{ formattedServerTime }}</p>
        </div>

        <div class="bg-white border border-slate-200 rounded-2xl p-5">
          <div class="flex items-center gap-2 text-slate-500 text-sm font-medium">
            <ShieldCheck class="w-4 h-4" />
            Environment
          </div>
          <p class="text-slate-900 text-lg font-semibold mt-2 capitalize">{{ runtime.environment }}</p>
        </div>

        <div class="bg-white border border-slate-200 rounded-2xl p-5">
          <div class="flex items-center gap-2 text-slate-500 text-sm font-medium">
            <Workflow class="w-4 h-4" />
            Registered node types
          </div>
          <p class="text-slate-900 text-lg font-semibold mt-2">{{ runtime.node_types_count }}</p>
        </div>

        <div class="bg-white border border-slate-200 rounded-2xl p-5">
          <div class="flex items-center gap-2 text-slate-500 text-sm font-medium">
            <KeyRound class="w-4 h-4" />
            Registered credential types
          </div>
          <p class="text-slate-900 text-lg font-semibold mt-2">{{ runtime.credential_types_count }}</p>
        </div>

        <div class="bg-white border border-slate-200 rounded-2xl p-5 md:col-span-2">
          <div class="flex items-center gap-2 text-slate-500 text-sm font-medium">
            <ShieldCheck class="w-4 h-4" />
            Encryption key status
          </div>
          <p
            :class="[
              'text-lg font-semibold mt-2',
              runtime.encryption_key_configured ? 'text-green-700' : 'text-red-700',
            ]"
          >
            {{ runtime.encryption_key_configured ? 'Configured' : 'Missing' }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
