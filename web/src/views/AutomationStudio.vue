<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  Activity,
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  Fingerprint,
  Loader2,
  Package2,
  Play,
  Rocket,
  ShieldCheck,
  ShieldX,
  Sparkles,
  TerminalSquare,
  Wand2,
  Workflow,
} from 'lucide-vue-next'
import { generateWorkflowDraft, invokeExtensionAction, listExtensionBundles } from '../features/studio/api'
import { saveWorkflow } from '../features/workflows/api'
import type {
  ExtensionActionInvocationRecord,
  ExtensionActionRecord,
  ExtensionBundleRecord,
  WorkflowDraftRecord,
} from '../types/contracts'

const router = useRouter()
const prompt = ref('')
const generating = ref(false)
const openingDraft = ref(false)
const loadingExtensions = ref(false)
const pageError = ref<string | null>(null)
const extensionError = ref<string | null>(null)
const draft = ref<WorkflowDraftRecord | null>(null)
const extensions = ref<ExtensionBundleRecord[]>([])
const actionContexts = ref<Record<string, string>>({})
const actionErrors = ref<Record<string, string | null>>({})
const actionResults = ref<Record<string, ExtensionActionInvocationRecord>>({})
const invokingActionKey = ref<string | null>(null)

const samplePrompts = [
  'Receive webhook leads, score high-intent submissions with AI, and notify Slack for urgent ones.',
  'Every morning fetch GitHub issues, summarize blockers with AI, and send the update to Telegram.',
  'Poll a status API every five minutes and page Slack only when severity is critical.',
]

const generatedNodeNames = computed(() =>
  (draft.value?.nodes || []).map((node) => node.name).filter(Boolean),
)

const recommendedBundles = computed(() => {
  const recommendedIds = new Set(draft.value?.recommendedExtensions || [])
  return extensions.value.filter((bundle) => recommendedIds.has(bundle.id))
})

const extensionCount = computed(() => extensions.value.length)
const validatedExtensionCount = computed(
  () => extensions.value.filter((bundle) => bundle.status === 'validated').length,
)
const verifiedSignatureCount = computed(
  () => extensions.value.filter((bundle) => bundle.signatureStatus === 'verified').length,
)
const runtimeActionCount = computed(() =>
  extensions.value.reduce((total, bundle) => total + bundle.actions.length, 0),
)

function applySamplePrompt(sample: string) {
  prompt.value = sample
}

function bundleStatusClass(status: string) {
  switch (status) {
    case 'validated':
      return 'bg-emerald-100 text-emerald-700'
    case 'validatedWithWarnings':
      return 'bg-amber-100 text-amber-700'
    default:
      return 'bg-red-100 text-red-700'
  }
}

function isRecommendedBundle(bundle: ExtensionBundleRecord) {
  return (draft.value?.recommendedExtensions || []).includes(bundle.id)
}

function actionKey(bundleId: string, actionId: string) {
  return `${bundleId}:${actionId}`
}

function actionContextValue(bundleId: string, actionId: string) {
  return actionContexts.value[actionKey(bundleId, actionId)] || defaultActionContext(actionId)
}

function actionError(bundleId: string, actionId: string) {
  return actionErrors.value[actionKey(bundleId, actionId)] || null
}

function actionResult(bundleId: string, actionId: string) {
  return actionResults.value[actionKey(bundleId, actionId)] || null
}

function ensureActionContexts(bundles: ExtensionBundleRecord[]) {
  for (const bundle of bundles) {
    for (const action of bundle.actions) {
      const key = actionKey(bundle.id, action.id)
      if (!actionContexts.value[key]) {
        actionContexts.value[key] = defaultActionContext(action.id)
      }
    }
  }
}

function defaultActionContext(actionId: string) {
  switch (actionId) {
    case 'runtime-health':
      return JSON.stringify({ scope: 'workspace', includeAdvice: true }, null, 2)
    case 'incident-brief':
      return JSON.stringify(
        {
          incidentTitle: 'Slack delivery latency spike',
          severity: 'high',
          currentError: 'Slack API requests exceeded the configured timeout window.',
        },
        null,
        2,
      )
    case 'prompt-planner':
      return JSON.stringify(
        {
          prompt: 'Summarize GitHub issue backlog with AI and notify Slack when priority is high.',
        },
        null,
        2,
      )
    case 'run-diagnosis':
      return JSON.stringify(
        {
          failingNode: 'OpenAI Summarizer',
          status: 'failed',
          error: 'Upstream provider rejected the model parameter.',
        },
        null,
        2,
      )
    default:
      return JSON.stringify({}, null, 2)
  }
}

function signatureBadgeClass(status: string) {
  switch (status) {
    case 'verified':
      return 'bg-emerald-100 text-emerald-700'
    case 'unsigned':
      return 'bg-slate-200 text-slate-700'
    default:
      return 'bg-red-100 text-red-700'
  }
}

function digestPreview(digest: string) {
  if (!digest) return 'n/a'
  return `${digest.slice(0, 12)}...`
}

function stringifyOutput(output: Record<string, unknown>) {
  return JSON.stringify(output, null, 2)
}

async function loadExtensions() {
  loadingExtensions.value = true
  extensionError.value = null
  try {
    const response = await listExtensionBundles()
    extensions.value = Array.isArray(response.data) ? response.data : []
    ensureActionContexts(extensions.value)
  } catch (error: any) {
    extensionError.value = error?.response?.data || error?.message || 'Failed to load extension catalog.'
  } finally {
    loadingExtensions.value = false
  }
}

async function createDraft() {
  if (!prompt.value.trim()) return

  generating.value = true
  pageError.value = null
  try {
    const response = await generateWorkflowDraft({ prompt: prompt.value.trim() })
    draft.value = response.data
  } catch (error: any) {
    pageError.value = error?.response?.data || error?.message || 'Failed to generate workflow draft.'
  } finally {
    generating.value = false
  }
}

async function openDraftInEditor() {
  if (!draft.value) return

  openingDraft.value = true
  pageError.value = null
  try {
    const response = await saveWorkflow({
      name: draft.value.name,
      nodes: draft.value.nodes,
      connections: draft.value.connections,
      settings: draft.value.settings,
      tags: draft.value.suggestedTags,
    })
    await router.push(`/workflow/${response.data.id}`)
  } catch (error: any) {
    pageError.value = error?.response?.data || error?.message || 'Failed to create workflow from draft.'
  } finally {
    openingDraft.value = false
  }
}

async function runBundleAction(bundle: ExtensionBundleRecord, action: ExtensionActionRecord) {
  const key = actionKey(bundle.id, action.id)
  invokingActionKey.value = key
  actionErrors.value[key] = null

  let context = {}
  try {
    context = JSON.parse(actionContexts.value[key] || '{}')
  } catch (error: any) {
    actionErrors.value[key] = error?.message || 'Context must be valid JSON.'
    invokingActionKey.value = null
    return
  }

  try {
    const response = await invokeExtensionAction(bundle.id, {
      actionId: action.id,
      context,
    })
    actionResults.value[key] = response.data
  } catch (error: any) {
    actionErrors.value[key] = error?.response?.data || error?.message || 'Failed to invoke extension action.'
  } finally {
    invokingActionKey.value = null
  }
}

onMounted(() => {
  loadExtensions()
})
</script>

<template>
  <div class="min-h-full bg-transparent p-4 text-slate-900 md:p-8">
    <div class="mx-auto max-w-7xl space-y-6">
      <section class="rounded-[2rem] border border-slate-200/80 bg-white px-6 py-6 shadow-panel md:px-8">
        <div class="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div class="max-w-3xl">
            <div class="inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-600">
              <Sparkles class="h-3.5 w-3.5" />
              Advanced Track
            </div>
            <h1 class="mt-3 text-3xl font-black tracking-tight text-slate-950 md:text-4xl">
              Automation Studio
            </h1>
            <p class="mt-2 text-sm leading-6 text-slate-600 md:text-base">
              Generate workflow drafts from natural-language intent and inspect the built-in extension packs that package BarqFlow’s advanced operational and AI surfaces.
            </p>
          </div>

          <div class="grid gap-3 sm:grid-cols-2 lg:min-w-[320px]">
            <div class="rounded-[1.4rem] border border-slate-200 bg-slate-50 px-4 py-4 shadow-sm">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Extension Packs</p>
              <p class="mt-2 text-3xl font-black text-slate-950">{{ extensionCount }}</p>
              <p class="mt-1 text-xs text-slate-500">Catalogs discovered from the local extension registry.</p>
            </div>
            <div class="rounded-[1.4rem] border border-slate-200 bg-slate-50 px-4 py-4 shadow-sm">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Verified Signatures</p>
              <p class="mt-2 text-3xl font-black text-emerald-600">{{ verifiedSignatureCount }}</p>
              <p class="mt-1 text-xs text-slate-500">Bundles trusted for runtime invocation in this build.</p>
            </div>
            <div class="rounded-[1.4rem] border border-slate-200 bg-slate-50 px-4 py-4 shadow-sm">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Validated</p>
              <p class="mt-2 text-3xl font-black text-slate-950">{{ validatedExtensionCount }}</p>
              <p class="mt-1 text-xs text-slate-500">Bundles with assets aligned to the current platform surface.</p>
            </div>
            <div class="rounded-[1.4rem] border border-slate-200 bg-slate-50 px-4 py-4 shadow-sm">
              <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Runtime Actions</p>
              <p class="mt-2 text-3xl font-black text-slate-950">{{ runtimeActionCount }}</p>
              <p class="mt-1 text-xs text-slate-500">Capability-scoped actions exposed by the built-in packs.</p>
            </div>
          </div>
        </div>
      </section>

      <div v-if="pageError" class="rounded-[1.5rem] border border-red-200 bg-red-50 px-5 py-4 text-sm text-red-700 shadow-panel">
        {{ pageError }}
      </div>

      <section class="grid gap-6 xl:grid-cols-[minmax(0,1.55fr)_380px]">
        <div class="space-y-6">
          <div class="rounded-[1.8rem] border border-slate-200/80 bg-white p-6 shadow-panel">
            <div class="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
              <div class="max-w-2xl">
                <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-400">Prompt Builder</p>
                <h2 class="mt-2 text-2xl font-black text-slate-950">Describe the workflow you want</h2>
                <p class="mt-2 text-sm leading-6 text-slate-500">
                  The studio maps your intent onto the current BarqFlow node catalog, extension packs, and workflow schema. It produces a draft you can open directly in the designer.
                </p>
              </div>
              <div class="inline-flex items-center gap-2 rounded-full bg-blue-50 px-3 py-2 text-xs font-semibold text-blue-700">
                <ShieldCheck class="h-4 w-4" />
                Deterministic planner
              </div>
            </div>

            <div class="mt-6 space-y-4">
              <label class="block">
                <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Workflow Intent</span>
                <textarea
                  v-model="prompt"
                  rows="6"
                  class="w-full rounded-[1.5rem] border border-slate-200 bg-slate-50 px-4 py-4 text-sm leading-6 text-slate-800 outline-none transition focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20"
                  placeholder="Example: Every morning fetch GitHub issues, summarize blockers with AI, and notify Slack only when severity is high."
                ></textarea>
              </label>

              <div>
                <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Starter Prompts</p>
                <div class="mt-3 flex flex-wrap gap-2">
                  <button
                    v-for="sample in samplePrompts"
                    :key="sample"
                    type="button"
                    class="rounded-full border border-slate-200 bg-white px-4 py-2 text-left text-xs font-semibold text-slate-600 transition hover:border-slate-300 hover:bg-slate-50"
                    @click="applySamplePrompt(sample)"
                  >
                    {{ sample }}
                  </button>
                </div>
              </div>

              <div class="flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  class="inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-5 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:opacity-60"
                  :disabled="generating || !prompt.trim()"
                  @click="createDraft"
                >
                  <Loader2 v-if="generating" class="h-4 w-4 animate-spin" />
                  <Wand2 v-else class="h-4 w-4" />
                  Generate Draft
                </button>
                <p class="text-sm text-slate-500">The studio uses the current built-in node catalog and extension manifests. No remote model call is required to draft the flow.</p>
              </div>
            </div>
          </div>

          <div class="rounded-[1.8rem] border border-slate-200/80 bg-white p-6 shadow-panel">
            <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
              <div>
                <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-400">Draft Output</p>
                <h2 class="mt-2 text-2xl font-black text-slate-950">Workflow preview</h2>
                <p class="mt-2 text-sm text-slate-500">
                  Review the generated node chain, required credentials, and extension recommendations before opening the draft in the editor.
                </p>
              </div>
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:opacity-60"
                :disabled="!draft || openingDraft"
                @click="openDraftInEditor"
              >
                <Loader2 v-if="openingDraft" class="h-4 w-4 animate-spin" />
                <Rocket v-else class="h-4 w-4" />
                Open In Editor
              </button>
            </div>

            <div v-if="draft" class="mt-6 space-y-6">
              <div class="rounded-[1.5rem] border border-slate-200 bg-slate-50 px-5 py-5">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                  <div>
                    <div class="inline-flex items-center gap-2 rounded-full bg-white px-3 py-1 text-[11px] font-bold uppercase tracking-[0.18em] text-slate-500 ring-1 ring-slate-200">
                      <Workflow class="h-3.5 w-3.5" />
                      {{ draft.generator }}
                    </div>
                    <h3 class="mt-3 text-2xl font-black text-slate-950">{{ draft.name }}</h3>
                    <p class="mt-2 text-sm leading-6 text-slate-600">{{ draft.summary }}</p>
                  </div>
                  <div class="flex flex-wrap gap-2 lg:max-w-[260px] lg:justify-end">
                    <span
                      v-for="tag in draft.suggestedTags"
                      :key="tag"
                      class="rounded-full bg-slate-900 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.16em] text-white"
                    >
                      {{ tag }}
                    </span>
                  </div>
                </div>
              </div>

              <div class="grid gap-4 md:grid-cols-2">
                <div class="rounded-[1.5rem] border border-slate-200 bg-white p-5 shadow-sm">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Node Chain</p>
                  <div class="mt-4 flex flex-wrap items-center gap-2">
                    <template v-for="(nodeName, index) in generatedNodeNames" :key="`${nodeName}-${index}`">
                      <span class="rounded-full bg-slate-100 px-3 py-2 text-sm font-semibold text-slate-700">
                        {{ nodeName }}
                      </span>
                      <ArrowRight v-if="index < generatedNodeNames.length - 1" class="h-4 w-4 text-slate-300" />
                    </template>
                  </div>
                </div>

                <div class="rounded-[1.5rem] border border-slate-200 bg-white p-5 shadow-sm">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Credential Coverage</p>
                  <div v-if="draft.requiredCredentials.length > 0" class="mt-4 flex flex-wrap gap-2">
                    <span
                      v-for="credential in draft.requiredCredentials"
                      :key="credential"
                      class="rounded-full bg-amber-100 px-3 py-2 text-sm font-semibold text-amber-700"
                    >
                      {{ credential }}
                    </span>
                  </div>
                  <div v-else class="mt-4 rounded-2xl bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                    This draft does not require an external credential binding yet.
                  </div>
                </div>
              </div>

              <div class="grid gap-4 lg:grid-cols-2">
                <div class="rounded-[1.5rem] border border-slate-200 bg-white p-5 shadow-sm">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Planning Rationale</p>
                  <ul class="mt-4 space-y-3 text-sm text-slate-600">
                    <li v-for="item in draft.rationale" :key="item" class="flex items-start gap-3">
                      <CheckCircle2 class="mt-0.5 h-4 w-4 shrink-0 text-emerald-500" />
                      <span>{{ item }}</span>
                    </li>
                  </ul>
                </div>

                <div class="rounded-[1.5rem] border border-slate-200 bg-white p-5 shadow-sm">
                  <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Draft Warnings</p>
                  <div v-if="draft.warnings.length > 0" class="mt-4 space-y-3">
                    <div
                      v-for="warning in draft.warnings"
                      :key="warning"
                      class="flex items-start gap-3 rounded-2xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800"
                    >
                      <AlertTriangle class="mt-0.5 h-4 w-4 shrink-0" />
                      <span>{{ warning }}</span>
                    </div>
                  </div>
                  <div v-else class="mt-4 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
                    The draft does not currently have planner warnings.
                  </div>
                </div>
              </div>

              <div class="rounded-[1.5rem] border border-slate-200 bg-white p-5 shadow-sm">
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Recommended Packs</p>
                    <p class="mt-1 text-sm text-slate-500">These extension bundles cover one or more of the generated nodes.</p>
                  </div>
                  <Package2 class="h-5 w-5 text-slate-400" />
                </div>
                <div v-if="recommendedBundles.length > 0" class="mt-4 grid gap-3 md:grid-cols-2">
                  <div
                    v-for="bundle in recommendedBundles"
                    :key="bundle.id"
                    class="rounded-2xl border border-brand-200 bg-brand-50 px-4 py-4"
                  >
                    <div class="flex items-center justify-between gap-3">
                      <p class="text-sm font-bold text-slate-900">{{ bundle.name }}</p>
                      <span :class="['rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.16em]', bundleStatusClass(bundle.status)]">
                        {{ bundle.status }}
                      </span>
                    </div>
                    <p class="mt-2 text-sm text-slate-600">{{ bundle.description }}</p>
                  </div>
                </div>
                <div v-else class="mt-4 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-600">
                  No extension pack overlap was detected for this draft.
                </div>
              </div>
            </div>

            <div v-else class="mt-6 rounded-[1.5rem] border border-dashed border-slate-300 bg-slate-50 px-6 py-16 text-center">
              <div class="mx-auto max-w-md">
                <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-900 text-white shadow-sm">
                  <Sparkles class="h-6 w-6" />
                </div>
                <h3 class="mt-5 text-xl font-black text-slate-950">No draft generated yet</h3>
                <p class="mt-2 text-sm leading-6 text-slate-500">
                  Submit a prompt and the studio will return a workflow name, node chain, required credentials, and a ready-to-save draft structure.
                </p>
              </div>
            </div>
          </div>
        </div>

        <aside class="space-y-6">
          <div class="rounded-[1.8rem] border border-slate-200/80 bg-white p-6 shadow-panel">
            <div class="flex items-center justify-between gap-3">
              <div>
                <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-400">Extension Registry</p>
                <h2 class="mt-2 text-2xl font-black text-slate-950">Built-in packs</h2>
              </div>
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-2 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 disabled:opacity-60"
                :disabled="loadingExtensions"
                @click="loadExtensions"
              >
                <Loader2 v-if="loadingExtensions" class="h-4 w-4 animate-spin" />
                <Package2 v-else class="h-4 w-4" />
                Refresh
              </button>
            </div>

            <p class="mt-2 text-sm leading-6 text-slate-500">
              Each pack is discovered from the local file-backed extension catalog and validated against the current BarqFlow node and template surface.
            </p>

            <div v-if="extensionError" class="mt-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {{ extensionError }}
            </div>

            <div v-if="loadingExtensions" class="mt-6 flex items-center gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4 text-sm text-slate-500">
              <Loader2 class="h-4 w-4 animate-spin" />
              Loading extension catalog...
            </div>

            <div v-else class="mt-6 space-y-4">
              <div
                v-for="bundle in extensions"
                :key="bundle.id"
                :class="[
                  'rounded-[1.5rem] border px-4 py-4 transition',
                  isRecommendedBundle(bundle)
                    ? 'border-brand-200 bg-brand-50 shadow-sm'
                    : 'border-slate-200 bg-slate-50',
                ]"
              >
                <div class="flex items-start justify-between gap-3">
                  <div>
                    <div class="flex flex-wrap items-center gap-2">
                      <h3 class="text-sm font-bold text-slate-950">{{ bundle.name }}</h3>
                      <span :class="['rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.16em]', bundleStatusClass(bundle.status)]">
                        {{ bundle.status }}
                      </span>
                      <span v-if="isRecommendedBundle(bundle)" class="rounded-full bg-slate-950 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.16em] text-white">
                        Recommended
                      </span>
                    </div>
                    <p class="mt-1 text-xs text-slate-500">{{ bundle.vendor }} · {{ bundle.version }} · {{ bundle.runtime }}</p>
                  </div>
                  <Sparkles class="h-4 w-4 text-slate-400" />
                </div>

                <p class="mt-3 text-sm leading-6 text-slate-600">{{ bundle.description }}</p>

                <div class="mt-4 grid gap-3 sm:grid-cols-2">
                  <div class="rounded-2xl border border-white/70 bg-white/80 px-3 py-3">
                    <p class="text-[11px] font-bold uppercase tracking-[0.16em] text-slate-400">Provides</p>
                    <p class="mt-2 text-sm font-semibold text-slate-900">{{ bundle.providedAssets.nodes.length }} nodes · {{ bundle.providedAssets.templates.length }} templates</p>
                    <p class="mt-1 text-xs text-slate-500">{{ bundle.providedAssets.panels.length }} panels</p>
                  </div>
                  <div class="rounded-2xl border border-white/70 bg-white/80 px-3 py-3">
                    <p class="text-[11px] font-bold uppercase tracking-[0.16em] text-slate-400">Digest</p>
                    <p class="mt-2 font-mono text-sm font-semibold text-slate-900">{{ digestPreview(bundle.digest) }}</p>
                    <p class="mt-1 text-xs text-slate-500">{{ bundle.sourcePath }}</p>
                  </div>
                </div>

                <div class="mt-4 grid gap-3 sm:grid-cols-2">
                  <div class="rounded-2xl border border-white/70 bg-white/80 px-3 py-3">
                    <div class="flex items-center justify-between gap-3">
                      <p class="text-[11px] font-bold uppercase tracking-[0.16em] text-slate-400">Trust</p>
                      <Fingerprint class="h-4 w-4 text-slate-400" />
                    </div>
                    <div class="mt-2 flex flex-wrap items-center gap-2">
                      <span :class="['rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.16em]', signatureBadgeClass(bundle.signatureStatus)]">
                        {{ bundle.signatureStatus }}
                      </span>
                      <span class="text-xs text-slate-500">
                        {{ bundle.signatureKeyId || 'No key id recorded' }}
                      </span>
                    </div>
                  </div>
                  <div class="rounded-2xl border border-white/70 bg-white/80 px-3 py-3">
                    <div class="flex items-center justify-between gap-3">
                      <p class="text-[11px] font-bold uppercase tracking-[0.16em] text-slate-400">Runtime Actions</p>
                      <Activity class="h-4 w-4 text-slate-400" />
                    </div>
                    <p class="mt-2 text-sm font-semibold text-slate-900">{{ bundle.actions.length }} exposed actions</p>
                    <p class="mt-1 text-xs text-slate-500">Only verified built-in packs can execute these actions.</p>
                  </div>
                </div>

                <div class="mt-4 flex flex-wrap gap-2">
                  <span
                    v-for="capability in bundle.capabilities"
                    :key="capability"
                    class="rounded-full bg-white px-3 py-1.5 text-[11px] font-bold uppercase tracking-[0.14em] text-slate-600 ring-1 ring-slate-200"
                  >
                    {{ capability }}
                  </span>
                </div>

                <div v-if="bundle.actions.length > 0" class="mt-5 space-y-4">
                  <div
                    v-for="action in bundle.actions"
                    :key="action.id"
                    class="rounded-[1.35rem] border border-white/70 bg-white/90 px-4 py-4"
                  >
                    <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                      <div class="max-w-xl">
                        <div class="flex flex-wrap items-center gap-2">
                          <p class="text-sm font-bold text-slate-950">{{ action.name }}</p>
                          <span class="rounded-full bg-slate-950 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.16em] text-white">
                            {{ action.id }}
                          </span>
                        </div>
                        <p class="mt-2 text-sm leading-6 text-slate-600">{{ action.description }}</p>
                      </div>
                      <button
                        type="button"
                        class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-slate-950 px-4 py-3 text-sm font-semibold text-white transition hover:bg-slate-800 disabled:opacity-60"
                        :disabled="invokingActionKey === actionKey(bundle.id, action.id) || bundle.signatureStatus !== 'verified'"
                        @click="runBundleAction(bundle, action)"
                      >
                        <Loader2 v-if="invokingActionKey === actionKey(bundle.id, action.id)" class="h-4 w-4 animate-spin" />
                        <Play v-else class="h-4 w-4" />
                        Invoke Action
                      </button>
                    </div>

                    <div class="mt-4 flex flex-wrap gap-2">
                      <span
                        v-for="capability in action.requiredCapabilities"
                        :key="capability"
                        class="rounded-full bg-slate-100 px-3 py-1.5 text-[11px] font-bold uppercase tracking-[0.14em] text-slate-600"
                      >
                        {{ capability }}
                      </span>
                    </div>

                    <label class="mt-4 block">
                      <span class="mb-2 block text-[11px] font-bold uppercase tracking-[0.16em] text-slate-400">Invocation Context (JSON)</span>
                      <textarea
                        v-model="actionContexts[actionKey(bundle.id, action.id)]"
                        rows="7"
                        class="w-full rounded-[1.2rem] border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs leading-6 text-slate-700 outline-none transition focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20"
                      ></textarea>
                    </label>

                    <div
                      v-if="bundle.signatureStatus !== 'verified'"
                      class="mt-4 flex items-start gap-3 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
                    >
                      <ShieldX class="mt-0.5 h-4 w-4 shrink-0" />
                      <span>This bundle cannot execute runtime actions until its signature is verified.</span>
                    </div>

                    <div
                      v-if="actionError(bundle.id, action.id)"
                      class="mt-4 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
                    >
                      {{ actionError(bundle.id, action.id) }}
                    </div>

                    <div
                      v-if="actionResult(bundle.id, action.id)"
                      class="mt-4 rounded-[1.3rem] border border-emerald-200 bg-emerald-50/80 px-4 py-4"
                    >
                      <div class="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                        <div>
                          <p class="text-[11px] font-bold uppercase tracking-[0.16em] text-emerald-700">Latest Invocation</p>
                          <p class="mt-2 text-sm font-semibold text-slate-900">{{ actionResult(bundle.id, action.id)?.summary }}</p>
                        </div>
                        <div class="inline-flex items-center gap-2 rounded-full bg-white px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.16em] text-emerald-700 ring-1 ring-emerald-200">
                          <ShieldCheck class="h-3.5 w-3.5" />
                          {{ actionResult(bundle.id, action.id)?.signatureStatus }}
                        </div>
                      </div>

                      <div class="mt-4 flex flex-wrap gap-2">
                        <span
                          v-for="capability in actionResult(bundle.id, action.id)?.capabilityTrace || []"
                          :key="capability"
                          class="rounded-full bg-white px-3 py-1 text-[11px] font-bold uppercase tracking-[0.14em] text-slate-600 ring-1 ring-emerald-200"
                        >
                          {{ capability }}
                        </span>
                      </div>

                      <div class="mt-4 rounded-[1.15rem] border border-emerald-200 bg-slate-950 px-4 py-4 text-xs text-slate-100">
                        <div class="mb-2 flex items-center gap-2 text-emerald-300">
                          <TerminalSquare class="h-4 w-4" />
                          Structured output
                        </div>
                        <pre class="overflow-x-auto whitespace-pre-wrap leading-6">{{ stringifyOutput(actionResult(bundle.id, action.id)?.output || {}) }}</pre>
                      </div>
                    </div>
                  </div>
                </div>

                <div v-if="bundle.warnings.length > 0" class="mt-4 space-y-2">
                  <div
                    v-for="warning in bundle.warnings"
                    :key="warning"
                    class="rounded-2xl border border-amber-200 bg-amber-50 px-3 py-3 text-xs text-amber-800"
                  >
                    {{ warning }}
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="rounded-[1.8rem] border border-slate-200/80 bg-white p-6 shadow-panel">
            <p class="text-[11px] font-bold uppercase tracking-[0.2em] text-slate-400">How To Use It</p>
            <div class="mt-4 space-y-3 text-sm text-slate-600">
              <div class="flex items-start gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
                <div class="mt-0.5 flex h-6 w-6 items-center justify-center rounded-full bg-slate-900 text-[11px] font-bold text-white">1</div>
                <p>Name the trigger, the source system, any AI step, and the delivery channel in one sentence.</p>
              </div>
              <div class="flex items-start gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
                <div class="mt-0.5 flex h-6 w-6 items-center justify-center rounded-full bg-slate-900 text-[11px] font-bold text-white">2</div>
                <p>Review the required credentials and warnings before opening the draft in the full-screen workflow designer.</p>
              </div>
              <div class="flex items-start gap-3 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
                <div class="mt-0.5 flex h-6 w-6 items-center justify-center rounded-full bg-slate-900 text-[11px] font-bold text-white">3</div>
                <p>Use the extension registry to understand which built-in packs cover the generated workflow surface.</p>
              </div>
            </div>

            <div class="mt-5 rounded-[1.5rem] border border-blue-200 bg-blue-50 px-4 py-4 text-sm text-blue-800">
              <div class="flex items-start gap-3">
                <Rocket class="mt-0.5 h-4 w-4 shrink-0" />
                <p>
                  Opening a draft creates a real workflow record immediately. The studio does not store drafts server-side before that step.
                </p>
              </div>
            </div>

            <div class="mt-4 rounded-[1.5rem] border border-slate-200 bg-slate-50 px-4 py-4 text-sm text-slate-700">
              <div class="flex items-start gap-3">
                <ShieldCheck class="mt-0.5 h-4 w-4 shrink-0 text-emerald-600" />
                <p>
                  Runtime actions are restricted to verified built-in packs and are evaluated against the bundle capability manifest before execution.
                </p>
              </div>
            </div>
          </div>
        </aside>
      </section>
    </div>
  </div>
</template>
