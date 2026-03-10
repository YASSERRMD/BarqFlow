<script setup lang="ts">
const props = defineProps<{
  modelValue: string
  tabs: Array<{
    id: string
    label: string
    badge?: string | number | null
  }>
}>()

const emit = defineEmits<{
  (event: 'update:modelValue', value: string): void
}>()
</script>

<template>
  <div class="px-6 pt-4 bg-white border-b border-slate-200">
    <div class="flex gap-2 overflow-x-auto pb-3">
      <button
        v-for="tab in props.tabs"
        :key="tab.id"
        type="button"
        class="inline-flex items-center gap-2 whitespace-nowrap rounded-full border px-3 py-1.5 text-xs font-semibold transition-colors"
        :class="
          props.modelValue === tab.id
            ? 'border-brand-500 bg-brand-50 text-brand-700'
            : 'border-slate-200 bg-slate-50 text-slate-600 hover:bg-slate-100'
        "
        @click="emit('update:modelValue', tab.id)"
      >
        <span>{{ tab.label }}</span>
        <span
          v-if="tab.badge !== undefined && tab.badge !== null && `${tab.badge}`.length > 0"
          class="rounded-full bg-white/80 px-1.5 py-0.5 text-[10px] font-bold text-slate-500"
        >
          {{ tab.badge }}
        </span>
      </button>
    </div>
  </div>
</template>
