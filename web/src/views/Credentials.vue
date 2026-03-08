<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Plus, Search, Shield, Key, Lock, X, Trash2, FlaskConical, CheckCircle2, XCircle, Loader2 } from 'lucide-vue-next'
import api from '../api'

const credentials = ref<any[]>([])
const credentialTypes = ref<any[]>([])

const categories = ['All', 'Database', 'Messaging', 'AI', 'Marketing', 'Storage']
const activeCategory = ref('All')
const searchTerm = ref('')

const isModalOpen = ref(false)
const selectedType = ref<any>(null)
const newCredentialData = ref<any>({})
const newCredentialName = ref('')

const saveLoading = ref(false)
const testLoading = ref(false)
const modalError = ref<string | null>(null)
const modalSuccess = ref<string | null>(null)
const lastTestValid = ref<boolean | null>(null)

const filteredCredentials = computed(() => {
  const query = searchTerm.value.trim().toLowerCase()
  return credentials.value.filter((cred) => {
    const type = (cred.credential_type || '').toLowerCase()
    const name = (cred.name || '').toLowerCase()
    const matchesCategory =
      activeCategory.value === 'All' ||
      type.includes(activeCategory.value.toLowerCase())
    const matchesQuery =
      query.length === 0 || name.includes(query) || type.includes(query)

    return matchesCategory && matchesQuery
  })
})

watch(
  () => JSON.stringify(newCredentialData.value),
  () => {
    if (lastTestValid.value !== null) {
      lastTestValid.value = null
      modalSuccess.value = null
    }
  },
)

onMounted(async () => {
  await fetchCredentials()
  await fetchCredentialTypes()
})

function resetModalState() {
  modalError.value = null
  modalSuccess.value = null
  lastTestValid.value = null
}

function openCreateModal() {
  isModalOpen.value = true
  selectedType.value = null
  newCredentialData.value = {}
  newCredentialName.value = ''
  resetModalState()
}

function chooseType(type: any) {
  selectedType.value = type
  newCredentialData.value = {}
  resetModalState()
}

function changeType() {
  selectedType.value = null
  newCredentialData.value = {}
  resetModalState()
}

async function fetchCredentials() {
  try {
    const res = await api.get('/credentials')
    credentials.value = res.data
  } catch (err) {
    console.error(err)
  }
}

async function fetchCredentialTypes() {
  try {
    const res = await api.get('/credentials/types')
    credentialTypes.value = res.data
  } catch (err) {
    console.error(err)
  }
}

async function testCredential() {
  if (!selectedType.value) return

  testLoading.value = true
  modalError.value = null
  modalSuccess.value = null

  try {
    const res = await api.post('/credentials/test', {
      cred_type: selectedType.value.name,
      data: newCredentialData.value,
    })

    if (res.data?.valid) {
      lastTestValid.value = true
      modalSuccess.value = 'Credential test passed.'
    } else {
      lastTestValid.value = false
      modalError.value = 'Credential test failed.'
    }
  } catch (err: any) {
    lastTestValid.value = false
    modalError.value = err?.response?.data || err?.message || 'Credential test failed.'
  } finally {
    testLoading.value = false
  }
}

async function saveCredential() {
  if (!selectedType.value || !newCredentialName.value) return

  if (lastTestValid.value !== true) {
    modalError.value = 'Run and pass credential test before saving.'
    return
  }

  saveLoading.value = true
  modalError.value = null
  modalSuccess.value = null

  try {
    await api.post('/credentials', {
      name: newCredentialName.value,
      cred_type: selectedType.value.name,
      data: newCredentialData.value,
    })

    isModalOpen.value = false
    newCredentialData.value = {}
    newCredentialName.value = ''
    selectedType.value = null
    resetModalState()
    await fetchCredentials()
  } catch (err: any) {
    modalError.value = err?.response?.data || err?.message || 'Failed to save credential.'
  } finally {
    saveLoading.value = false
  }
}

async function deleteCredential(id: string) {
  const confirmed = window.confirm('Delete this credential?')
  if (!confirmed) return

  try {
    await api.delete(`/credentials/${id}`)
    credentials.value = credentials.value.filter((c) => c.id !== id)
  } catch (err) {
    console.error('Failed to delete credential', err)
  }
}
</script>

<template>
  <div class="h-full bg-slate-50/50 overflow-auto p-6 md:p-10 text-slate-900">
    <div class="max-w-6xl mx-auto">
      <div class="flex flex-col md:flex-row md:items-end justify-between mb-10 gap-6">
        <div>
          <h1 class="text-4xl font-extrabold text-slate-900 tracking-tight">Credentials</h1>
          <p class="text-slate-500 text-lg mt-2 font-medium">Securely managed keys and OAuth tokens for your integrations.</p>
        </div>

        <button
          @click="openCreateModal"
          class="bg-brand-500 hover:bg-brand-600 text-white px-6 py-3.5 rounded-2xl flex items-center gap-2.5 shadow-xl shadow-brand-500/20 transition-all hover:-translate-y-1 active:translate-y-0 font-bold"
        >
          <Plus class="w-5 h-5" /> Add Credential
        </button>
      </div>

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
            v-model="searchTerm"
            type="text"
            placeholder="Search credentials..."
            class="w-full pl-12 pr-4 py-3 bg-white border border-slate-200 rounded-2xl text-sm focus:ring-4 focus:ring-brand-500/10 focus:border-brand-500 transition-all font-medium"
          />
        </div>
      </div>

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
            <tr v-for="cred in filteredCredentials" :key="cred.id" class="hover:bg-slate-50/50 transition-colors group">
              <td class="px-8 py-6">
                <div class="flex items-center gap-4">
                  <div class="w-10 h-10 bg-slate-100 rounded-xl flex items-center justify-center text-slate-500 group-hover:bg-brand-50 group-hover:text-brand-600 transition-colors">
                    <Key v-if="cred.credential_type !== 'Database'" class="w-5 h-5" />
                    <Shield v-else class="w-5 h-5" />
                  </div>
                  <span class="font-bold text-slate-800 group-hover:text-brand-600 transition-colors">{{ cred.name }}</span>
                </div>
              </td>
              <td class="px-8 py-6">
                <span class="text-sm font-bold text-slate-500 bg-slate-100 px-3 py-1 rounded-lg">{{ cred.credential_type }}</span>
              </td>
              <td class="px-8 py-6">
                <div class="flex items-center gap-2">
                  <div class="w-2 h-2 rounded-full bg-green-500"></div>
                  <span class="text-sm font-bold capitalize text-green-600">Saved</span>
                </div>
              </td>
              <td class="px-8 py-6">
                <span class="text-sm font-medium text-slate-400">Unknown</span>
              </td>
              <td class="px-8 py-6 text-right">
                <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button @click="deleteCredential(cred.id)" class="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"><Trash2 class="w-4 h-4" /></button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

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

    <div v-if="isModalOpen" class="fixed inset-0 bg-slate-900/40 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div class="bg-white rounded-3xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col overflow-hidden">
        <div class="px-8 py-6 border-b border-slate-100 flex items-center justify-between">
          <h2 class="text-2xl font-black text-slate-900">Add Credential</h2>
          <button @click="isModalOpen = false" class="p-2 text-slate-400 hover:text-slate-900 bg-slate-50 hover:bg-slate-100 rounded-xl transition-colors">
            <X class="w-5 h-5" />
          </button>
        </div>

        <div class="p-8 overflow-y-auto flex-1 bg-slate-50/50">
          <div v-if="!selectedType" class="space-y-4">
            <label class="block text-sm font-bold text-slate-700">Select Credential Type</label>
            <div class="grid grid-cols-2 gap-4">
              <button
                v-for="type in credentialTypes"
                :key="type.name"
                @click="chooseType(type)"
                class="p-4 bg-white border-2 border-slate-100 hover:border-brand-500 rounded-2xl flex flex-col items-start gap-2 text-left transition-all"
              >
                <div class="w-10 h-10 bg-brand-50 text-brand-600 rounded-xl flex items-center justify-center">
                  <Key class="w-5 h-5" />
                </div>
                <span class="font-bold text-slate-800">{{ type.displayName || type.name }}</span>
              </button>
            </div>
          </div>

          <div v-else class="space-y-6">
            <div class="flex items-center justify-between bg-white p-4 rounded-2xl border border-slate-100">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 bg-brand-50 text-brand-600 rounded-xl flex items-center justify-center">
                  <Key class="w-5 h-5" />
                </div>
                <div>
                  <h3 class="font-bold text-slate-900">{{ selectedType.displayName || selectedType.name }}</h3>
                  <button @click="changeType" class="text-xs font-bold text-brand-600 hover:text-brand-700">Change Type</button>
                </div>
              </div>
            </div>

            <div v-if="modalError" class="flex items-center gap-2 text-sm text-red-700 bg-red-50 border border-red-200 rounded-lg px-3 py-2">
              <XCircle class="w-4 h-4" />
              {{ modalError }}
            </div>
            <div v-if="modalSuccess" class="flex items-center gap-2 text-sm text-green-700 bg-green-50 border border-green-200 rounded-lg px-3 py-2">
              <CheckCircle2 class="w-4 h-4" />
              {{ modalSuccess }}
            </div>

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-bold text-slate-700 mb-2">Credential Name</label>
                <input
                  v-model="newCredentialName"
                  type="text"
                  placeholder="e.g. Production Database"
                  class="w-full px-4 py-3 bg-white border border-slate-200 focus:border-brand-500 rounded-xl text-sm font-medium transition-all outline-none"
                />
              </div>

              <div v-for="(prop, idx) in selectedType.properties" :key="idx" class="pt-4 border-t border-slate-100">
                <label class="block text-sm font-bold text-slate-700 mb-2">{{ prop.displayName }}</label>
                <input
                  v-if="prop.type === 'string' || prop.type === 'text'"
                  v-model="newCredentialData[prop.name]"
                  :type="prop.type === 'string' && prop.name.toLowerCase().includes('password') ? 'password' : 'text'"
                  class="w-full px-4 py-3 bg-white border border-slate-200 focus:border-brand-500 rounded-xl text-sm font-medium transition-all outline-none"
                />
                <select
                  v-else-if="prop.type === 'options'"
                  v-model="newCredentialData[prop.name]"
                  class="w-full px-4 py-3 bg-white border border-slate-200 focus:border-brand-500 rounded-xl text-sm font-bold text-slate-800 transition-all outline-none"
                >
                  <option v-for="opt in prop.options" :key="opt.value" :value="opt.value">{{ opt.name }}</option>
                </select>
                <p v-if="prop.description" class="mt-2 text-xs text-slate-400">{{ prop.description }}</p>
              </div>
            </div>
          </div>
        </div>

        <div class="px-8 py-6 border-t border-slate-100 bg-white flex justify-end gap-3">
          <button @click="isModalOpen = false" class="px-6 py-3 font-bold text-slate-500 hover:bg-slate-50 rounded-xl transition-colors">Cancel</button>
          <button
            v-if="selectedType"
            @click="testCredential"
            :disabled="testLoading"
            class="px-6 py-3 font-bold text-slate-700 bg-slate-100 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-slate-200 rounded-xl transition-all flex items-center gap-2"
          >
            <Loader2 v-if="testLoading" class="w-4 h-4 animate-spin" />
            <FlaskConical v-else class="w-4 h-4" />
            Test Credential
          </button>
          <button
            v-if="selectedType"
            @click="saveCredential"
            :disabled="!newCredentialName || saveLoading || lastTestValid !== true"
            class="px-8 py-3 font-bold text-white bg-slate-900 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-slate-800 rounded-xl transition-all shadow-xl shadow-slate-900/10 hover:-translate-y-0.5 active:translate-y-0 flex items-center gap-2"
          >
            <Loader2 v-if="saveLoading" class="w-4 h-4 animate-spin" />
            Save Credential
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
