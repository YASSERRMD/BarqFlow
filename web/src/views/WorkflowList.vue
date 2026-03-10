<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  ArrowDownAZ,
  ArrowUpDown,
  Calendar,
  Copy,
  Download,
  Edit2,
  Filter,
  Loader2,
  Plus,
  Power,
  Rocket,
  Search,
  Sparkles,
  Tag,
  Trash2,
  Upload,
  Wand2,
  Workflow,
  X,
} from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { useWorkflowStore } from '../stores/workflows'
import { listExecutions } from '../features/executions/api'
import type {
  WorkflowExportRecord,
  WorkflowImportRequest,
  WorkflowRecord,
  WorkflowTemplateRecord,
} from '../types/contracts'

const router = useRouter()
const workflowStore = useWorkflowStore()
const fileInput = ref<HTMLInputElement | null>(null)
const searchTerm = ref('')
const statusFilter = ref<'all' | 'active' | 'inactive'>('all')
const sortBy = ref<'updatedAt' | 'createdAt' | 'name'>('updatedAt')
const sortDirection = ref<'asc' | 'desc'>('desc')
const selectedTags = ref<string[]>([])
const templateSearch = ref('')
const latestExecutionByWorkflow = ref<Record<string, any>>({})
const showStarterModal = ref(false)
const newTagName = ref('')
const busyCardId = ref<string | null>(null)
const importError = ref<string | null>(null)
let searchTimer: ReturnType<typeof setTimeout> | null = null

const totalWorkflowCount = computed(() => workflowStore.workflows.length)
const activeWorkflowCount = computed(
  () => workflowStore.workflows.filter((workflow) => workflow.active).length,
)
const totalNodeCount = computed(() =>
  workflowStore.workflows.reduce(
    (count, workflow) => count + (workflow.summary?.nodeCount || workflow.nodes.length || 0),
    0,
  ),
)
const starterTemplates = computed(() =>
  workflowStore.workflowTemplates.filter((template) => template.difficulty === 'starter').slice(0, 3),
)
const filteredTemplates = computed(() => {
  const needle = templateSearch.value.trim().toLowerCase()
  if (!needle) return workflowStore.workflowTemplates

  return workflowStore.workflowTemplates.filter((template) => {
    const haystack = [
      template.name,
      template.description,
      template.category,
      ...(template.tags || []),
      ...(template.highlights || []),
    ]
      .join(' ')
      .toLowerCase()

    return haystack.includes(needle)
  })
})

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

function workflowRelativeUpdatedAt(workflow: WorkflowRecord): string {
  return formatRelativeTime(workflow.updatedAt || workflow.createdAt)
}

function workflowTimestampTitle(workflow: WorkflowRecord): string {
  const iso = workflow.updatedAt || workflow.createdAt
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
    tags: selectedTags.value.length > 0 ? [...selectedTags.value] : undefined,
    limit: 200,
    sortBy: sortBy.value,
    sortDirection: sortDirection.value,
  }
}

async function fetchWorkflows() {
  await workflowStore.fetchWorkflows(buildListParams())
}

async function fetchPageData() {
  await Promise.all([
    fetchWorkflows(),
    fetchExecutionMetadata(),
    workflowStore.fetchWorkflowTags(),
    workflowStore.fetchWorkflowTemplates(),
  ])
}

async function fetchExecutionMetadata() {
  try {
    const response = await listExecutions({ limit: 500 })
    const executions = Array.isArray(response.data) ? response.data : []
    const nextMap: Record<string, any> = {}

    for (const execution of executions) {
      const workflowId = String(execution?.workflowId || '')
      if (!workflowId) continue

      const current = nextMap[workflowId]
      const currentTs = current?.startedAt
        ? new Date(current.startedAt).getTime()
        : Number.NEGATIVE_INFINITY
      const candidateTs = execution?.startedAt
        ? new Date(execution.startedAt).getTime()
        : Number.NEGATIVE_INFINITY

      if (!current || candidateTs >= currentTs) {
        nextMap[workflowId] = execution
      }
    }

    latestExecutionByWorkflow.value = nextMap
  } catch (error) {
    console.error('Failed to fetch execution metadata', error)
  }
}

function workflowLastExecutionIso(workflow: WorkflowRecord): string | null {
  const execution = latestExecutionByWorkflow.value[workflow.id]
  return execution?.startedAt || execution?.stoppedAt || null
}

function workflowLastExecutionLabel(workflow: WorkflowRecord): string {
  const iso = workflowLastExecutionIso(workflow)
  return iso ? formatRelativeTime(iso) : 'No runs yet'
}

function workflowLastExecutionTitle(workflow: WorkflowRecord): string {
  const iso = workflowLastExecutionIso(workflow)
  if (!iso) return 'No runs yet'

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return 'No runs yet'
  return date.toLocaleString()
}

function workflowTagNames(workflow: WorkflowRecord): string[] {
  return (workflow.tags || []).map((tag) => tag.name)
}

function workflowNodeCount(workflow: WorkflowRecord): number {
  return workflow.summary?.nodeCount || workflow.nodes.length || 0
}

function workflowTriggerCount(workflow: WorkflowRecord): number {
  return workflow.summary?.triggerCount || 0
}

function workflowVersion(workflow: WorkflowRecord): number {
  return workflow.summary?.latestVersion || 0
}

function toggleTagFilter(tagName: string) {
  if (selectedTags.value.includes(tagName)) {
    selectedTags.value = selectedTags.value.filter((tag) => tag !== tagName)
  } else {
    selectedTags.value = [...selectedTags.value, tagName]
  }
}

function clearTagFilters() {
  selectedTags.value = []
}

function openStarterModal() {
  showStarterModal.value = true
}

function closeStarterModal() {
  showStarterModal.value = false
  templateSearch.value = ''
}

function editWorkflow(id: string) {
  router.push(`/workflow/${id}`)
}

function createBlankWorkflow() {
  closeStarterModal()
  router.push('/workflow/new')
}

async function deleteWorkflow(id: string) {
  const confirmed = window.confirm('Delete this workflow permanently?')
  if (!confirmed) return

  busyCardId.value = id
  try {
    await workflowStore.deleteWorkflow(id)
    await fetchExecutionMetadata()
  } catch (error) {
    console.error('Failed to delete workflow', error)
  } finally {
    busyCardId.value = null
  }
}

async function toggleWorkflowActive(id: string, current: boolean) {
  busyCardId.value = id
  try {
    await workflowStore.toggleWorkflowActive(id, !current)
    await fetchExecutionMetadata()
    if (statusFilter.value !== 'all') {
      await fetchWorkflows()
    }
  } catch (error) {
    console.error('Failed to update workflow activation', error)
  } finally {
    busyCardId.value = null
  }
}

async function duplicateWorkflow(id: string) {
  busyCardId.value = id
  try {
    const duplicated = await workflowStore.duplicateWorkflow(id)
    await fetchExecutionMetadata()
    router.push(`/workflow/${duplicated.id}`)
  } catch (error) {
    console.error('Failed to duplicate workflow', error)
  } finally {
    busyCardId.value = null
  }
}

async function exportWorkflowJson(workflow: WorkflowRecord) {
  busyCardId.value = workflow.id
  try {
    const exported = await workflowStore.exportWorkflow(workflow.id)
    downloadWorkflowExport(exported, workflow.name)
  } catch (error) {
    console.error('Failed to export workflow', error)
  } finally {
    busyCardId.value = null
  }
}

function downloadWorkflowExport(exported: WorkflowExportRecord, workflowName: string) {
  const blob = new Blob([JSON.stringify(exported, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `${slugify(workflowName)}.barqflow.json`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64) || 'workflow'
}

function triggerImportDialog() {
  importError.value = null
  fileInput.value?.click()
}

async function handleImportFile(event: Event) {
  const input = event.target as HTMLInputElement | null
  const file = input?.files?.[0]
  if (!file) return

  importError.value = null

  try {
    const text = await file.text()
    const parsed = JSON.parse(text)
    const payload = normalizeWorkflowImportPayload(parsed)
    const imported = await workflowStore.importWorkflow(payload)
    await Promise.all([
      fetchWorkflows(),
      workflowStore.fetchWorkflowTags(),
    ])
    router.push(`/workflow/${imported.id}`)
  } catch (error: any) {
    importError.value = error?.message || 'Unable to import this workflow file.'
  } finally {
    if (input) {
      input.value = ''
    }
  }
}

function normalizeWorkflowImportPayload(raw: unknown): WorkflowImportRequest {
  if (!raw || typeof raw !== 'object') {
    throw new Error('Unsupported workflow file format.')
  }

  const candidate = raw as Record<string, any>
  const document = normalizeWorkflowDocument(
    candidate.workflow && typeof candidate.workflow === 'object'
      ? candidate.workflow
      : candidate,
  )

  return {
    workflow: document,
  }
}

function normalizeWorkflowDocument(raw: Record<string, any>) {
  if (!Array.isArray(raw.nodes)) {
    throw new Error('Workflow file is missing a valid nodes array.')
  }

  const tags = Array.isArray(raw.tags)
    ? raw.tags
        .map((tag: any) => {
          if (typeof tag === 'string') return tag
          if (tag && typeof tag === 'object' && typeof tag.name === 'string') return tag.name
          return null
        })
        .filter((tag: string | null): tag is string => !!tag)
    : []

  return {
    name: typeof raw.name === 'string' && raw.name.trim().length > 0 ? raw.name : 'Imported Workflow',
    nodes: raw.nodes,
    connections:
      raw.connections && typeof raw.connections === 'object' && !Array.isArray(raw.connections)
        ? raw.connections
        : {},
    settings:
      raw.settings && typeof raw.settings === 'object' && !Array.isArray(raw.settings)
        ? raw.settings
        : {},
    tags,
  }
}

async function instantiateTemplate(template: WorkflowTemplateRecord) {
  busyCardId.value = template.id
  try {
    const created = await workflowStore.instantiateWorkflowTemplate(template.id)
    await Promise.all([
      fetchWorkflows(),
      workflowStore.fetchWorkflowTags(),
    ])
    closeStarterModal()
    router.push(`/workflow/${created.id}`)
  } catch (error) {
    console.error('Failed to instantiate workflow template', error)
  } finally {
    busyCardId.value = null
  }
}

async function createTag() {
  const name = newTagName.value.trim()
  if (!name) return

  try {
    await workflowStore.createWorkflowTag(name)
    newTagName.value = ''
    await workflowStore.fetchWorkflowTags()
  } catch (error) {
    console.error('Failed to create workflow tag', error)
  }
}

async function removeTag(id: string) {
  const confirmed = window.confirm('Delete this tag from the workspace? It will be removed from linked workflows.')
  if (!confirmed) return

  const tagName = workflowStore.workflowTags.find((tag) => tag.id === id)?.name || null

  try {
    await workflowStore.deleteWorkflowTag(id)
    if (tagName) {
      selectedTags.value = selectedTags.value.filter((selectedTag) => selectedTag !== tagName)
    }
    await Promise.all([
      workflowStore.fetchWorkflowTags(),
      fetchWorkflows(),
    ])
  } catch (error) {
    console.error('Failed to delete workflow tag', error)
  }
}

onMounted(async () => {
  await fetchPageData()
})

watch([searchTerm, statusFilter, sortBy, sortDirection, selectedTags], () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    fetchWorkflows()
  }, 250)
})

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer)
})
</script>

<template>
  <div class="h-full overflow-auto bg-transparent px-6 py-8 md:px-10 md:py-10">
    <input
      ref="fileInput"
      type="file"
      class="hidden"
      accept="application/json,.json"
      @change="handleImportFile"
    />

    <div class="mx-auto flex max-w-7xl flex-col gap-8">
      <section class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm md:p-8">
        <div class="flex flex-col gap-6 xl:flex-row xl:items-center xl:justify-between">
          <div class="max-w-3xl">
            <p class="text-[11px] font-black uppercase tracking-[0.24em] text-brand-600">Workflow Management</p>
            <h1 class="mt-3 text-3xl font-display font-black tracking-tight text-slate-950 md:text-4xl">
              Workflow Catalog and Operations
            </h1>
            <p class="mt-3 max-w-2xl text-sm font-medium leading-6 text-slate-600 md:text-base">
              Search, organize, import, export, and operate workflows from one operational workspace before opening the editor.
            </p>
          </div>

          <div class="flex flex-wrap gap-3 xl:justify-end">
            <button
              @click="openStarterModal"
              class="inline-flex items-center justify-center gap-2 rounded-xl bg-slate-950 px-4 py-2.5 text-sm font-black text-white transition hover:bg-slate-800"
            >
              <Plus class="h-4 w-4" />
              New Workflow
            </button>
            <button
              @click="triggerImportDialog"
              class="inline-flex items-center justify-center gap-2 rounded-xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-bold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
            >
              <Upload class="h-4 w-4" />
              Import JSON
            </button>
            <button
              @click="showStarterModal = true"
              class="inline-flex items-center justify-center gap-2 rounded-xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-bold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
            >
              <Sparkles class="h-4 w-4" />
              Templates
            </button>
          </div>
        </div>
      </section>

      <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Catalog</p>
          <p class="mt-3 text-3xl font-display font-black text-slate-950">{{ totalWorkflowCount }}</p>
          <p class="mt-1 text-sm font-medium text-slate-500">Workflows loaded in the current view.</p>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Active</p>
          <p class="mt-3 text-3xl font-display font-black text-emerald-600">{{ activeWorkflowCount }}</p>
          <p class="mt-1 text-sm font-medium text-slate-500">Triggers currently armed and live.</p>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Nodes</p>
          <p class="mt-3 text-3xl font-display font-black text-slate-950">{{ totalNodeCount }}</p>
          <p class="mt-1 text-sm font-medium text-slate-500">Total steps across the loaded workflow set.</p>
        </div>
        <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Templates</p>
          <p class="mt-3 text-3xl font-display font-black text-brand-600">{{ workflowStore.workflowTemplates.length }}</p>
          <p class="mt-1 text-sm font-medium text-slate-500">Starter and intermediate launch blueprints.</p>
        </div>
      </section>

      <section class="grid gap-6 xl:grid-cols-[1.45fr_0.95fr]">
        <div class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
          <div class="flex items-center justify-between gap-4">
            <div>
              <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Filters</p>
              <h2 class="mt-2 text-2xl font-display font-black text-slate-950">Search and segment the catalog</h2>
            </div>
            <div class="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1 text-[11px] font-black uppercase tracking-[0.2em] text-slate-500">
              <Filter class="h-3.5 w-3.5" />
              Server-backed
            </div>
          </div>

          <div class="mt-6 grid gap-4 lg:grid-cols-[1.4fr_0.8fr_0.8fr]">
            <label class="relative block">
              <Search class="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
              <input
                v-model="searchTerm"
                type="text"
                placeholder="Search workflows, tags, or themes"
                class="w-full rounded-2xl border border-slate-200 bg-slate-50/90 py-3 pl-11 pr-4 text-sm font-medium text-slate-900 outline-none transition focus:border-brand-400 focus:bg-white"
              />
            </label>

            <label class="block">
              <span class="mb-2 block text-[11px] font-black uppercase tracking-[0.2em] text-slate-400">Status</span>
              <select
                v-model="statusFilter"
                class="w-full rounded-2xl border border-slate-200 bg-slate-50/90 px-4 py-3 text-sm font-bold text-slate-700 outline-none transition focus:border-brand-400 focus:bg-white"
              >
                <option value="all">All workflows</option>
                <option value="active">Active only</option>
                <option value="inactive">Inactive only</option>
              </select>
            </label>

            <div class="grid gap-3 sm:grid-cols-2">
              <label class="block">
                <span class="mb-2 block text-[11px] font-black uppercase tracking-[0.2em] text-slate-400">Sort By</span>
                <select
                  v-model="sortBy"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50/90 px-4 py-3 text-sm font-bold text-slate-700 outline-none transition focus:border-brand-400 focus:bg-white"
                >
                  <option value="updatedAt">Recently updated</option>
                  <option value="createdAt">Recently created</option>
                  <option value="name">Name</option>
                </select>
              </label>
              <label class="block">
                <span class="mb-2 block text-[11px] font-black uppercase tracking-[0.2em] text-slate-400">Direction</span>
                <select
                  v-model="sortDirection"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50/90 px-4 py-3 text-sm font-bold text-slate-700 outline-none transition focus:border-brand-400 focus:bg-white"
                >
                  <option value="desc">Descending</option>
                  <option value="asc">Ascending</option>
                </select>
              </label>
            </div>
          </div>

          <div class="mt-6 flex flex-wrap items-center gap-3">
            <div class="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1.5 text-xs font-black uppercase tracking-[0.18em] text-slate-500">
              <Tag class="h-3.5 w-3.5" />
              Tag Filters
            </div>
            <button
              v-for="tag in workflowStore.workflowTags"
              :key="tag.id"
              type="button"
              @click="toggleTagFilter(tag.name)"
              :class="[
                'rounded-full border px-3 py-1.5 text-xs font-bold transition',
                selectedTags.includes(tag.name)
                  ? 'border-brand-500 bg-brand-500 text-white shadow-[0_8px_24px_rgba(14,165,233,0.22)]'
                  : 'border-slate-200 bg-white text-slate-600 hover:border-brand-200 hover:text-brand-600'
              ]"
            >
              {{ tag.name }}
              <span class="ml-1 opacity-70">{{ tag.workflowCount }}</span>
            </button>
            <button
              v-if="selectedTags.length > 0"
              type="button"
              @click="clearTagFilters"
              class="rounded-full border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-bold text-slate-500 transition hover:border-slate-300 hover:text-slate-700"
            >
              Clear tags
            </button>
          </div>
        </div>

        <div class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
          <div class="flex items-center justify-between gap-4">
            <div>
              <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Tag Studio</p>
              <h2 class="mt-2 text-2xl font-display font-black text-slate-950">Create and clean workspace tags</h2>
            </div>
            <div class="rounded-full bg-amber-50 px-3 py-1 text-[11px] font-black uppercase tracking-[0.18em] text-amber-700">
              Workspace scoped
            </div>
          </div>

          <div class="mt-6 flex gap-3">
            <input
              v-model="newTagName"
              type="text"
              placeholder="Add tag, for example ops or q2"
              class="w-full rounded-2xl border border-slate-200 bg-slate-50/90 px-4 py-3 text-sm font-medium text-slate-900 outline-none transition focus:border-brand-400 focus:bg-white"
              @keydown.enter.prevent="createTag"
            />
            <button
              @click="createTag"
              class="inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-black text-white transition hover:bg-slate-900"
            >
              <Plus class="h-4 w-4" />
              Add
            </button>
          </div>

          <div class="mt-6 flex max-h-56 flex-wrap gap-2 overflow-auto pr-1">
            <div
              v-for="tag in workflowStore.workflowTags"
              :key="tag.id"
              class="inline-flex items-center gap-2 rounded-full border border-slate-200 bg-slate-50 px-3 py-2 text-xs font-bold text-slate-700"
            >
              <span>{{ tag.name }}</span>
              <span class="rounded-full bg-white px-2 py-0.5 text-[10px] font-black uppercase tracking-[0.14em] text-slate-500">
                {{ tag.workflowCount }} flows
              </span>
              <button
                type="button"
                class="text-slate-400 transition hover:text-red-500"
                @click="removeTag(tag.id)"
              >
                <X class="h-3.5 w-3.5" />
              </button>
            </div>
            <p v-if="workflowStore.workflowTags.length === 0" class="text-sm font-medium text-slate-500">
              No tags yet. Create a few to power list filters and editor metadata.
            </p>
          </div>

          <p v-if="importError" class="mt-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm font-medium text-red-700">
            {{ importError }}
          </p>
        </div>
      </section>

      <section class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
        <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div>
            <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Templates</p>
            <h2 class="mt-2 text-2xl font-display font-black text-slate-950">Start from templates or blank workflows</h2>
          </div>
          <button
            @click="openStarterModal"
            class="inline-flex items-center gap-2 self-start rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-black text-slate-700 transition hover:border-brand-200 hover:text-brand-600 md:self-auto"
          >
            <Wand2 class="h-4 w-4" />
            Open Template Gallery
          </button>
        </div>

        <div class="mt-6 grid gap-4 xl:grid-cols-4">
          <button
            @click="createBlankWorkflow"
            class="group flex h-full flex-col justify-between rounded-[1.75rem] border border-dashed border-slate-300 bg-slate-50/80 p-6 text-left transition hover:-translate-y-1 hover:border-brand-400 hover:bg-brand-50/60"
          >
            <div class="inline-flex h-12 w-12 items-center justify-center rounded-2xl bg-white text-brand-600 shadow-sm transition group-hover:scale-105">
              <Rocket class="h-6 w-6" />
            </div>
            <div class="mt-8">
              <p class="text-lg font-display font-black text-slate-900">Blank canvas</p>
              <p class="mt-2 text-sm font-medium leading-6 text-slate-500">
                Start clean and build from scratch in the editor.
              </p>
            </div>
          </button>

          <button
            v-for="template in starterTemplates"
            :key="template.id"
            @click="instantiateTemplate(template)"
            class="group flex h-full flex-col justify-between rounded-[1.75rem] border border-slate-200 bg-white p-6 text-left shadow-sm transition hover:-translate-y-1 hover:border-brand-200 hover:shadow-[0_18px_40px_rgba(15,23,42,0.08)]"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="inline-flex h-12 w-12 items-center justify-center rounded-2xl bg-sky-50 text-sky-600 transition group-hover:bg-sky-600 group-hover:text-white">
                <Sparkles class="h-6 w-6" />
              </div>
              <span class="rounded-full bg-slate-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.18em] text-slate-500">
                {{ template.category }}
              </span>
            </div>
            <div class="mt-8">
              <p class="text-lg font-display font-black text-slate-950">{{ template.name }}</p>
              <p class="mt-2 text-sm font-medium leading-6 text-slate-500">
                {{ template.description }}
              </p>
              <div class="mt-4 flex flex-wrap gap-2">
                <span
                  v-for="tag in template.tags.slice(0, 3)"
                  :key="`${template.id}-${tag}`"
                  class="rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold text-slate-500"
                >
                  {{ tag }}
                </span>
              </div>
            </div>
          </button>
        </div>
      </section>

      <section>
        <div class="mb-6 flex items-center justify-between gap-4">
          <div>
            <p class="text-xs font-black uppercase tracking-[0.22em] text-slate-400">Workflow Inventory</p>
            <h2 class="mt-2 text-3xl font-display font-black text-slate-950">Browse and operate workflows</h2>
          </div>
          <div class="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1.5 text-[11px] font-black uppercase tracking-[0.2em] text-slate-500">
            <ArrowUpDown class="h-3.5 w-3.5" />
            {{ sortBy }} / {{ sortDirection }}
          </div>
        </div>

        <div v-if="workflowStore.loading" class="flex flex-col items-center justify-center py-24 opacity-60">
          <Loader2 class="mb-4 h-10 w-10 animate-spin text-brand-500" />
          <p class="font-bold text-slate-500">Loading workflows with the current filters…</p>
        </div>

        <div v-else-if="workflowStore.workflows.length === 0" class="rounded-[2rem] border border-dashed border-slate-300 bg-white/80 p-12 text-center shadow-sm">
          <div class="mx-auto inline-flex h-16 w-16 items-center justify-center rounded-[1.5rem] bg-slate-100 text-slate-500">
            <Workflow class="h-8 w-8" />
          </div>
          <h3 class="mt-6 text-2xl font-display font-black text-slate-950">No workflows match this slice.</h3>
          <p class="mx-auto mt-3 max-w-xl text-sm font-medium leading-6 text-slate-500">
            Change the filters, clear some tags, import a workflow JSON file, or launch a starter template to seed the workspace.
          </p>
          <div class="mt-8 flex flex-wrap items-center justify-center gap-3">
            <button
              @click="clearTagFilters"
              class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-bold text-slate-700 transition hover:border-slate-300"
            >
              Clear tag filters
            </button>
            <button
              @click="triggerImportDialog"
              class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-bold text-slate-700 transition hover:border-slate-300"
            >
              Import workflow
            </button>
            <button
              @click="openStarterModal"
              class="rounded-2xl bg-brand-500 px-4 py-3 text-sm font-black text-white transition hover:bg-brand-600"
            >
              Open starters
            </button>
          </div>
        </div>

        <div v-else class="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
          <article
            v-for="workflow in workflowStore.workflows"
            :key="workflow.id"
            class="group relative overflow-hidden rounded-[2rem] border border-slate-200/70 bg-white/95 p-7 shadow-sm transition hover:-translate-y-1 hover:shadow-[0_20px_50px_rgba(15,23,42,0.08)]"
          >
            <div class="pointer-events-none absolute inset-x-0 top-0 h-28 bg-[radial-gradient(circle_at_top_left,_rgba(14,165,233,0.12),_transparent_62%)] opacity-0 transition group-hover:opacity-100"></div>

            <div class="relative flex items-start justify-between gap-4">
              <button
                class="flex flex-1 items-start gap-4 text-left"
                @click="editWorkflow(workflow.id)"
              >
                <div class="inline-flex h-14 w-14 shrink-0 items-center justify-center rounded-[1.35rem] bg-slate-100 text-slate-600 transition group-hover:bg-brand-600 group-hover:text-white">
                  <Workflow class="h-7 w-7" />
                </div>
                <div class="min-w-0 flex-1">
                  <h3 class="truncate text-2xl font-display font-black text-slate-950 transition group-hover:text-brand-600">
                    {{ workflow.name }}
                  </h3>
                  <p class="mt-2 text-sm font-medium leading-6 text-slate-500">
                    {{ workflowNodeCount(workflow) }} nodes, {{ workflowTriggerCount(workflow) }} triggers, version {{ workflowVersion(workflow) }}.
                  </p>
                </div>
              </button>

              <div class="rounded-full px-3 py-1 text-[10px] font-black uppercase tracking-[0.2em] shadow-sm" :class="workflow.active ? 'bg-emerald-100 text-emerald-700' : 'bg-slate-100 text-slate-500'">
                {{ workflow.active ? 'Active' : 'Inactive' }}
              </div>
            </div>

            <div class="relative mt-6 flex flex-wrap gap-2">
              <span
                v-for="tag in workflow.tags"
                :key="tag.id"
                class="rounded-full border border-slate-200 bg-slate-50 px-3 py-1 text-[11px] font-bold text-slate-600"
              >
                {{ tag.name }}
              </span>
              <span v-if="workflow.tags.length === 0" class="rounded-full border border-dashed border-slate-200 px-3 py-1 text-[11px] font-bold text-slate-400">
                Untagged
              </span>
            </div>

            <div class="relative mt-6 grid grid-cols-3 gap-3">
              <div class="rounded-2xl bg-slate-50 p-3">
                <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Last Run</p>
                <p class="mt-2 text-sm font-bold text-slate-700" :title="workflowLastExecutionTitle(workflow)">
                  {{ workflowLastExecutionLabel(workflow) }}
                </p>
              </div>
              <div class="rounded-2xl bg-slate-50 p-3">
                <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Bindings</p>
                <p class="mt-2 text-sm font-bold text-slate-700">
                  {{ workflow.summary.credentialBindingCount }} linked
                </p>
              </div>
              <div class="rounded-2xl bg-slate-50 p-3">
                <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Updated</p>
                <p class="mt-2 text-sm font-bold text-slate-700" :title="workflowTimestampTitle(workflow)">
                  {{ workflowRelativeUpdatedAt(workflow) }}
                </p>
              </div>
            </div>

            <div class="relative mt-7 flex flex-wrap gap-2 border-t border-slate-100 pt-5">
              <button
                @click="duplicateWorkflow(workflow.id)"
                class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-600 transition hover:border-sky-200 hover:text-sky-600"
              >
                <Copy class="h-3.5 w-3.5" />
                Duplicate
              </button>
              <button
                @click="exportWorkflowJson(workflow)"
                class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-600 transition hover:border-violet-200 hover:text-violet-600"
              >
                <Download class="h-3.5 w-3.5" />
                Export
              </button>
              <button
                @click="toggleWorkflowActive(workflow.id, workflow.active)"
                class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-600 transition hover:border-amber-200 hover:text-amber-600"
              >
                <Power class="h-3.5 w-3.5" />
                {{ workflow.active ? 'Pause' : 'Activate' }}
              </button>
              <button
                @click="editWorkflow(workflow.id)"
                class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-600 transition hover:border-brand-200 hover:text-brand-600"
              >
                <Edit2 class="h-3.5 w-3.5" />
                Edit
              </button>
              <button
                @click="deleteWorkflow(workflow.id)"
                class="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-600 transition hover:border-red-200 hover:text-red-500"
              >
                <Trash2 class="h-3.5 w-3.5" />
                Delete
              </button>
            </div>

            <div v-if="busyCardId === workflow.id" class="absolute inset-0 flex items-center justify-center bg-white/80 backdrop-blur-sm">
              <div class="inline-flex items-center gap-2 rounded-full bg-slate-950 px-4 py-2 text-sm font-bold text-white">
                <Loader2 class="h-4 w-4 animate-spin" />
                Working…
              </div>
            </div>
          </article>
        </div>
      </section>
    </div>

    <div
      v-if="showStarterModal"
      class="fixed inset-0 z-50 flex items-start justify-center overflow-auto bg-slate-950/55 px-4 py-10 backdrop-blur-sm"
      @click.self="closeStarterModal"
    >
      <div class="w-full max-w-5xl rounded-[2rem] border border-white/20 bg-white p-6 shadow-[0_40px_120px_rgba(15,23,42,0.25)] md:p-8">
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="text-xs font-black uppercase tracking-[0.22em] text-brand-600">Workflow Starters</p>
            <h3 class="mt-2 text-3xl font-display font-black text-slate-950">Choose how the next workflow starts</h3>
            <p class="mt-3 text-sm font-medium leading-6 text-slate-500">
              Start blank, instantiate a template, or use the import button for existing JSON.
            </p>
          </div>
          <button
            @click="closeStarterModal"
            class="inline-flex h-11 w-11 items-center justify-center rounded-2xl border border-slate-200 text-slate-500 transition hover:border-slate-300 hover:text-slate-700"
          >
            <X class="h-5 w-5" />
          </button>
        </div>

        <div class="mt-6 rounded-[1.75rem] border border-dashed border-slate-300 bg-slate-50 p-5 md:flex md:items-center md:justify-between">
          <div>
            <p class="text-lg font-display font-black text-slate-950">Blank workflow</p>
            <p class="mt-2 text-sm font-medium text-slate-500">
              Open the editor with a clean canvas and build every node yourself.
            </p>
          </div>
          <button
            @click="createBlankWorkflow"
            class="mt-4 inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-4 py-3 text-sm font-black text-white transition hover:bg-slate-900 md:mt-0"
          >
            <Rocket class="h-4 w-4" />
            Start blank
          </button>
        </div>

        <div class="mt-6 relative">
          <Search class="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
          <input
            v-model="templateSearch"
            type="text"
            placeholder="Search templates by name, category, or tag"
            class="w-full rounded-2xl border border-slate-200 bg-slate-50 py-3 pl-11 pr-4 text-sm font-medium text-slate-900 outline-none transition focus:border-brand-400 focus:bg-white"
          />
        </div>

        <div class="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <button
            v-for="template in filteredTemplates"
            :key="template.id"
            @click="instantiateTemplate(template)"
            class="group flex h-full flex-col rounded-[1.75rem] border border-slate-200 bg-white p-5 text-left shadow-sm transition hover:-translate-y-1 hover:border-brand-200 hover:shadow-[0_18px_40px_rgba(15,23,42,0.08)]"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="inline-flex h-11 w-11 items-center justify-center rounded-2xl bg-slate-100 text-slate-600 transition group-hover:bg-brand-600 group-hover:text-white">
                <Sparkles class="h-5 w-5" />
              </div>
              <div class="flex flex-wrap justify-end gap-2">
                <span class="rounded-full bg-slate-100 px-3 py-1 text-[10px] font-black uppercase tracking-[0.18em] text-slate-500">
                  {{ template.category }}
                </span>
                <span class="rounded-full bg-brand-50 px-3 py-1 text-[10px] font-black uppercase tracking-[0.18em] text-brand-700">
                  {{ template.difficulty }}
                </span>
              </div>
            </div>

            <div class="mt-5 flex-1">
              <p class="text-xl font-display font-black text-slate-950">{{ template.name }}</p>
              <p class="mt-2 text-sm font-medium leading-6 text-slate-500">
                {{ template.description }}
              </p>
              <div class="mt-4 flex flex-wrap gap-2">
                <span
                  v-for="tag in template.tags"
                  :key="`${template.id}-${tag}`"
                  class="rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold text-slate-600"
                >
                  {{ tag }}
                </span>
              </div>
            </div>

            <div class="mt-5 grid grid-cols-3 gap-2 rounded-2xl bg-slate-50 p-3 text-center text-[11px] font-bold text-slate-600">
              <div>
                <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Nodes</p>
                <p class="mt-1 text-sm text-slate-800">{{ template.summary.nodeCount }}</p>
              </div>
              <div>
                <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Triggers</p>
                <p class="mt-1 text-sm text-slate-800">{{ template.summary.triggerCount }}</p>
              </div>
              <div>
                <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-400">Tags</p>
                <p class="mt-1 text-sm text-slate-800">{{ template.summary.tagCount }}</p>
              </div>
            </div>

            <div class="mt-5 text-xs font-medium leading-5 text-slate-500">
              <p v-for="highlight in template.highlights.slice(0, 2)" :key="highlight">
                {{ highlight }}
              </p>
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
