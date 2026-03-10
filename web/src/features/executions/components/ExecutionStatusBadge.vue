<script setup lang="ts">
const props = defineProps<{
  status: string
}>()

function badgeClass(status: string) {
  switch (status.toLowerCase()) {
    case 'success':
      return 'border-green-200 bg-green-50 text-green-700'
    case 'running':
      return 'border-blue-200 bg-blue-50 text-blue-700'
    case 'waiting':
      return 'border-amber-200 bg-amber-50 text-amber-700'
    case 'queued':
      return 'border-slate-200 bg-slate-100 text-slate-600'
    case 'stopped':
    case 'cancelled':
      return 'border-slate-300 bg-slate-100 text-slate-700'
    default:
      return 'border-red-200 bg-red-50 text-red-700'
  }
}
</script>

<template>
  <span
    class="inline-flex rounded-full border px-2.5 py-1 text-[11px] font-semibold uppercase tracking-wide"
    :class="badgeClass(props.status)"
  >
    {{ props.status }}
  </span>
</template>
