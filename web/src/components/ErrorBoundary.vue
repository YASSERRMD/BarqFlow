<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import { AlertTriangle, RefreshCw } from 'lucide-vue-next'

const error = ref<Error | null>(null)

onErrorCaptured((err: Error) => {
    error.value = err
    return false
})

function reset() {
    error.value = null
}
</script>

<template>
    <div
        v-if="error"
        class="flex min-h-[320px] flex-col items-center justify-center gap-6 rounded-2xl border border-red-200 bg-red-50 p-10 text-center"
    >
        <div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-red-100">
            <AlertTriangle class="h-7 w-7 text-red-600" />
        </div>
        <div class="max-w-sm">
            <p class="text-base font-semibold text-slate-900">Something went wrong</p>
            <p class="mt-2 text-sm text-slate-600">
                An unexpected error occurred in this view. You can try again or navigate to a
                different page.
            </p>
            <p v-if="error.message" class="mt-3 rounded-xl bg-red-100 px-4 py-2 font-mono text-xs text-red-700">
                {{ error.message }}
            </p>
        </div>
        <button
            class="inline-flex items-center gap-2 rounded-2xl bg-slate-950 px-5 py-2.5 text-sm font-semibold text-white transition hover:bg-slate-800"
            @click="reset"
        >
            <RefreshCw class="h-4 w-4" />
            Try again
        </button>
    </div>
    <slot v-else />
</template>
