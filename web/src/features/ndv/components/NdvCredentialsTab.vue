<script setup lang="ts">
const props = defineProps<{
  node: any
  nodeCredentialRefs: Array<{ credentialType: string; displayName: string; required: boolean }>
  credentialOptions: Record<string, any[]>
  credentialsLoading: boolean
  credentialsError: string | null
  missingRequiredCredentials: string[]
  credentialTypesWithoutSavedOptions: string[]
  openCredentialsPage: (credentialType?: string, displayName?: string) => void
}>()
</script>

<template>
  <section class="space-y-5 bg-white border border-slate-200 rounded-lg p-5">
    <div class="flex items-start justify-between gap-3">
      <div>
        <h3 class="font-semibold text-slate-800">Credentials</h3>
        <p class="mt-1 text-xs text-slate-500">
          Bind saved credentials to this node without embedding secrets in the workflow.
        </p>
      </div>
      <span class="rounded-full bg-slate-100 px-2 py-1 text-[10px] font-bold uppercase tracking-wide text-slate-500">
        {{ props.nodeCredentialRefs.length }} refs
      </span>
    </div>

    <div
      v-if="props.credentialsError"
      class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-600"
    >
      {{ props.credentialsError }}
    </div>

    <div
      v-if="props.missingRequiredCredentials.length > 0"
      class="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-700"
    >
      Missing required credentials: {{ props.missingRequiredCredentials.join(', ') }}
    </div>

    <div
      v-if="props.credentialTypesWithoutSavedOptions.length > 0"
      class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700"
    >
      No saved credentials available for {{ props.credentialTypesWithoutSavedOptions.join(', ') }}.
      <button
        type="button"
        class="ml-1 underline font-semibold"
        @click="props.openCredentialsPage(props.nodeCredentialRefs[0]?.credentialType)"
      >
        Open Credentials
      </button>
    </div>

    <div v-if="props.nodeCredentialRefs.length > 0" class="space-y-4">
      <div v-for="ref in props.nodeCredentialRefs" :key="ref.credentialType">
        <div class="mb-1.5 flex items-center justify-between gap-3">
          <label class="block text-sm font-medium text-slate-700">
            {{ ref.displayName }}
          </label>
          <button
            type="button"
            class="text-xs font-semibold text-brand-700 underline underline-offset-2"
            @click="props.openCredentialsPage(ref.credentialType, ref.displayName)"
          >
            Set up
          </button>
        </div>
        <select
          v-model="props.node.data.credentials[ref.credentialType]"
          :disabled="props.credentialsLoading"
          class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-brand-500 focus:ring-1 focus:ring-brand-500 disabled:opacity-70"
        >
          <option value="">Select credential</option>
          <option
            v-for="cred in props.credentialOptions[ref.credentialType] || []"
            :key="cred.id"
            :value="cred.id"
          >
            {{ cred.name }}
          </option>
        </select>
        <p
          v-if="ref.required && !props.node.data.credentials[ref.credentialType]"
          class="mt-1 text-xs text-amber-600"
        >
          This credential is required.
        </p>
        <div
          v-if="(props.credentialOptions[ref.credentialType] || []).length === 0"
          class="mt-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600"
        >
          No saved {{ ref.displayName }} credential exists yet.
          <button
            type="button"
            class="ml-1 font-semibold text-brand-700 underline underline-offset-2"
            @click="props.openCredentialsPage(ref.credentialType, ref.displayName)"
          >
            Create one now
          </button>
        </div>
      </div>
    </div>

    <div
      v-else
      class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-4 text-sm text-slate-500"
    >
      This node does not require a credential binding.
    </div>
  </section>
</template>
