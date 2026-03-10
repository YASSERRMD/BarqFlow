<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, Loader2, Lock, Mail, ShieldCheck, Workflow } from 'lucide-vue-next'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')
const firstName = ref('')
const lastName = ref('')
const isLogin = ref(true)

async function handleSubmit() {
  const payload = isLogin.value
    ? { email: email.value, password: password.value }
    : {
        email: email.value,
        password: password.value,
        firstName: firstName.value,
        lastName: lastName.value,
      }

  const success = isLogin.value
    ? await authStore.login(payload)
    : await authStore.register(payload)

  if (success) {
    router.push('/workflows')
  }
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-mesh-gradient px-4 py-8 sm:px-6 lg:px-8">
    <div class="grid w-full max-w-6xl overflow-hidden rounded-[2rem] border border-slate-200/80 bg-white shadow-[0_30px_80px_rgba(15,23,42,0.12)] lg:grid-cols-[1.08fr_0.92fr]">
      <section class="hidden border-r border-slate-200/80 bg-slate-950 px-10 py-12 text-white lg:flex lg:flex-col lg:justify-between">
        <div>
          <div class="flex items-center gap-4">
            <div class="flex h-14 w-14 items-center justify-center rounded-3xl bg-white/10 ring-1 ring-white/15">
              <img src="/logo.png" alt="BarqFlow" class="h-8 w-8 brightness-0 invert" />
            </div>
            <div>
              <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-400">Automation Platform</p>
              <h1 class="mt-1 text-3xl font-display font-bold text-white">BarqFlow Control Plane</h1>
            </div>
          </div>

          <div class="mt-16 max-w-xl space-y-6">
            <div>
              <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-sky-300">Production workflow operations</p>
              <h2 class="mt-3 text-4xl font-display font-bold leading-tight text-white">
                Build, secure, and operate automation with a product-grade control surface.
              </h2>
              <p class="mt-4 text-base leading-7 text-slate-300">
                Govern workflow design, credential lifecycle, and execution visibility from a single environment built for operational teams.
              </p>
            </div>

            <div class="grid gap-4 md:grid-cols-3">
              <div class="rounded-3xl border border-white/10 bg-white/5 p-5">
                <Workflow class="h-5 w-5 text-sky-300" />
                <p class="mt-4 text-lg font-semibold text-white">Workflow catalog</p>
                <p class="mt-2 text-sm leading-6 text-slate-300">Structured authoring, templates, and version-aware workflow inventory.</p>
              </div>
              <div class="rounded-3xl border border-white/10 bg-white/5 p-5">
                <ShieldCheck class="h-5 w-5 text-sky-300" />
                <p class="mt-4 text-lg font-semibold text-white">Credential governance</p>
                <p class="mt-2 text-sm leading-6 text-slate-300">Managed secrets, OAuth handoffs, and validation telemetry for integrations.</p>
              </div>
              <div class="rounded-3xl border border-white/10 bg-white/5 p-5">
                <ArrowRight class="h-5 w-5 text-sky-300" />
                <p class="mt-4 text-lg font-semibold text-white">Execution insight</p>
                <p class="mt-2 text-sm leading-6 text-slate-300">Operational event streams, retries, and wait-state inspection.</p>
              </div>
            </div>
          </div>
        </div>

        <div class="rounded-3xl border border-white/10 bg-white/5 px-6 py-5">
          <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-400">Platform posture</p>
          <div class="mt-3 flex items-center justify-between gap-4">
            <div>
              <p class="text-lg font-semibold text-white">Enterprise workflow foundations</p>
              <p class="mt-1 text-sm text-slate-300">Operationally focused UI, credential lifecycle, and execution telemetry.</p>
            </div>
            <div class="rounded-2xl bg-sky-400/10 px-3 py-2 text-xs font-bold uppercase tracking-[0.18em] text-sky-200">
              Secure by design
            </div>
          </div>
        </div>
      </section>

      <section class="flex items-center justify-center px-5 py-8 sm:px-8 lg:px-12">
        <div class="w-full max-w-md">
          <div class="mb-8 lg:hidden">
            <div class="flex items-center gap-3">
              <div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-slate-950 text-white">
                <img src="/logo.png" alt="BarqFlow" class="h-7 w-7 brightness-0 invert" />
              </div>
              <div>
                <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-slate-500">Automation Platform</p>
                <h1 class="text-2xl font-display font-bold text-slate-950">BarqFlow Control Plane</h1>
              </div>
            </div>
          </div>

          <div>
            <p class="text-[11px] font-extrabold uppercase tracking-[0.24em] text-sky-700">Access workspace</p>
            <h2 class="mt-3 text-3xl font-display font-bold text-slate-950">
              {{ isLogin ? 'Sign in to the platform' : 'Create a workspace account' }}
            </h2>
            <p class="mt-3 text-sm leading-6 text-slate-600">
              {{
                isLogin
                  ? 'Authenticate to manage workflows, credentials, and execution operations.'
                  : 'Provision a local account to start operating the automation platform.'
              }}
            </p>
          </div>

          <div
            v-if="authStore.error"
            class="mt-6 rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm font-medium text-red-700"
          >
            {{ authStore.error }}
          </div>

          <form class="mt-8 space-y-5" @submit.prevent="handleSubmit">
            <div v-if="!isLogin" class="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <label class="block">
                <span class="mb-2 block text-sm font-semibold text-slate-700">First name</span>
                <input
                  v-model="firstName"
                  type="text"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3.5 text-slate-900 outline-none transition focus:border-brand-500 focus:bg-white"
                />
              </label>
              <label class="block">
                <span class="mb-2 block text-sm font-semibold text-slate-700">Last name</span>
                <input
                  v-model="lastName"
                  type="text"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3.5 text-slate-900 outline-none transition focus:border-brand-500 focus:bg-white"
                />
              </label>
            </div>

            <label class="block">
              <span class="mb-2 block text-sm font-semibold text-slate-700">Email address</span>
              <div class="relative">
                <Mail class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400" />
                <input
                  v-model="email"
                  type="email"
                  required
                  placeholder="you@example.com"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 py-3.5 pl-12 pr-4 text-slate-900 outline-none transition focus:border-brand-500 focus:bg-white"
                />
              </div>
            </label>

            <label class="block">
              <span class="mb-2 block text-sm font-semibold text-slate-700">Password</span>
              <div class="relative">
                <Lock class="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400" />
                <input
                  v-model="password"
                  type="password"
                  required
                  placeholder="••••••••"
                  class="w-full rounded-2xl border border-slate-200 bg-slate-50 py-3.5 pl-12 pr-4 text-slate-900 outline-none transition focus:border-brand-500 focus:bg-white"
                />
              </div>
            </label>

            <div v-if="isLogin" class="flex items-center justify-between gap-4 text-sm">
              <label class="flex items-center gap-2 text-slate-600">
                <input type="checkbox" class="h-4 w-4 rounded border-slate-300 text-brand-600 focus:ring-brand-500" />
                <span>Remember this workspace</span>
              </label>
              <a href="#" class="font-semibold text-brand-700 hover:text-brand-800">Forgot password?</a>
            </div>

            <button
              type="submit"
              :disabled="authStore.loading"
              class="inline-flex w-full items-center justify-center gap-3 rounded-2xl bg-slate-950 px-5 py-3.5 text-base font-semibold text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-70"
            >
              <Loader2 v-if="authStore.loading" class="h-5 w-5 animate-spin" />
              <template v-else>
                {{ isLogin ? 'Sign In' : 'Create Account' }}
                <ArrowRight class="h-5 w-5" />
              </template>
            </button>
          </form>

          <div class="mt-8 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-4 text-sm text-slate-600">
            <span>{{ isLogin ? "Don't have an account?" : 'Already have an account?' }}</span>
            <button
              class="ml-1 font-semibold text-brand-700 hover:text-brand-800"
              @click="isLogin = !isLogin; authStore.error = null"
            >
              {{ isLogin ? 'Create one' : 'Sign in instead' }}
            </button>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
