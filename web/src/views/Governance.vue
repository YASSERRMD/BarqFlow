<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ClipboardList, RefreshCw, ShieldCheck } from 'lucide-vue-next'
import { getWorkspacePolicy, listAuditLogs, listPromotionRequests, listPromotionTargets, listSecretProviders } from '../features/governance/api'

const loading = ref(false)
const error = ref<string | null>(null)
const counts = ref({
  providers: 0,
  targets: 0,
  requests: 0,
  approvals: 0,
  logs: 0,
})

async function loadSurface() {
  loading.value = true
  error.value = null

  try {
    const [providersResponse, targetsResponse, requestsResponse, logsResponse, policyResponse] = await Promise.all([
      listSecretProviders(),
      listPromotionTargets(),
      listPromotionRequests(25),
      listAuditLogs(25),
      getWorkspacePolicy(),
    ])

    counts.value = {
      providers: providersResponse.data.length,
      targets: targetsResponse.data.length,
      requests: requestsResponse.data.length,
      approvals: policyResponse.data.approvalRequiredNodeTypes.length,
      logs: logsResponse.data.length,
    }
  } catch (err: any) {
    error.value = err?.response?.data?.message || err?.response?.data || err?.message || 'Failed to load governance surface.'
  } finally {
    loading.value = false
  }
}

onMounted(loadSurface)
</script>

<template>
  <div class="h-full overflow-y-auto bg-slate-50">
    <div class="mx-auto flex max-w-7xl flex-col gap-6 px-4 py-6 md:px-6 lg:px-8">
      <section class="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
        <div class="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div class="max-w-3xl">
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Enterprise controls</p>
            <h2 class="mt-2 text-3xl font-display font-bold tracking-tight text-slate-950">Governance Control Center</h2>
            <p class="mt-3 text-sm leading-6 text-slate-600 sm:text-base">
              Manage policy controls, promotion approvals, secret-provider posture, and audit evidence from one operational surface.
            </p>
          </div>

          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm font-semibold text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
            @click="loadSurface"
          >
            <RefreshCw class="h-4 w-4" />
            Refresh
          </button>
        </div>
      </section>

      <div v-if="error" class="rounded-2xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
        {{ error }}
      </div>

      <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Secret Providers</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ counts.providers }}</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Promotion Targets</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ counts.targets }}</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Promotion Requests</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ counts.requests }}</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Approval Gates</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ counts.approvals }}</p>
        </article>
        <article class="rounded-[24px] border border-slate-200 bg-white p-5 shadow-sm">
          <p class="text-[11px] font-bold uppercase tracking-[0.18em] text-slate-400">Audit Events</p>
          <p class="mt-3 text-3xl font-black text-slate-950">{{ counts.logs }}</p>
        </article>
      </section>

      <section class="rounded-[28px] border border-dashed border-slate-300 bg-white px-6 py-16 text-center shadow-sm">
        <div class="mx-auto max-w-2xl">
          <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-700">
            <ShieldCheck class="h-7 w-7" />
          </div>
          <h3 class="mt-5 text-2xl font-black text-slate-950">Governance surface is wired and ready for the full control plane build-out.</h3>
          <p class="mt-3 text-sm leading-6 text-slate-500 sm:text-base">
            This route now loads real policy, provider, promotion, and audit data. The next commit replaces this shell with the full operating surface and approval workflows.
          </p>
          <div class="mt-6 inline-flex items-center gap-2 rounded-full bg-slate-100 px-4 py-2 text-xs font-bold uppercase tracking-[0.2em] text-slate-600">
            <ClipboardList class="h-4 w-4" />
            Buildable phase checkpoint
          </div>
          <p v-if="loading" class="mt-4 text-sm text-slate-500">Loading governance data...</p>
        </div>
      </section>
    </div>
  </div>
</template>
