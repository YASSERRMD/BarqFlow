<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { Lock, Mail, Loader2, ArrowRight } from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')

async function handleLogin() {
  const success = await authStore.login({ email: email.value, password: password.value })
  if (success) {
    router.push('/workflows')
  }
}
</script>

<template>
  <div class="min-h-screen w-full flex items-center justify-center bg-slate-50 relative overflow-hidden p-6">
    <!-- Background Accents -->
    <div class="absolute top-0 right-0 w-[500px] h-[500px] bg-brand-100/50 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2"></div>
    <div class="absolute bottom-0 left-0 w-[500px] h-[500px] bg-purple-100/50 rounded-full blur-3xl translate-y-1/2 -translate-x-1/2"></div>

    <div class="w-full max-w-md bg-white/80 backdrop-blur-xl border border-white shadow-2xl shadow-slate-200/50 rounded-3xl p-8 md:p-12 relative z-10">
      <div class="text-center mb-10">
        <div class="w-16 h-16 bg-gradient-to-tr from-brand-500 to-brand-600 rounded-2xl flex items-center justify-center mx-auto mb-6 shadow-xl shadow-brand-500/20">
          <img src="/logo.png" alt="Logo" class="w-10 h-10 brightness-0 invert" />
        </div>
        <h1 class="text-3xl font-extrabold text-slate-900 tracking-tight">Welcome back</h1>
        <p class="text-slate-500 mt-2 font-medium">Log in to manage your workflows</p>
      </div>

      <form @submit.prevent="handleLogin" class="space-y-6">
        <div>
          <label class="block text-sm font-bold text-slate-700 mb-2 ml-1">Email address</label>
          <div class="relative group">
            <Mail class="w-5 h-5 absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-brand-500 transition-colors" />
            <input 
              v-model="email"
              type="email" 
              required
              placeholder="you@example.com"
              class="w-full pl-12 pr-4 py-4 bg-white border border-slate-200 rounded-2xl text-slate-900 focus:outline-none focus:ring-4 focus:ring-brand-500/10 focus:border-brand-500 transition-all font-medium"
            />
          </div>
        </div>

        <div>
          <label class="block text-sm font-bold text-slate-700 mb-2 ml-1">Password</label>
          <div class="relative group">
            <Lock class="w-5 h-5 absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-brand-500 transition-colors" />
            <input 
              v-model="password"
              type="password" 
              required
              placeholder="••••••••"
              class="w-full pl-12 pr-4 py-4 bg-white border border-slate-200 rounded-2xl text-slate-900 focus:outline-none focus:ring-4 focus:ring-brand-500/10 focus:border-brand-500 transition-all font-medium"
            />
          </div>
        </div>

        <div class="flex items-center justify-between ml-1">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" class="w-4 h-4 rounded border-slate-300 text-brand-600 focus:ring-brand-500" />
            <span class="text-sm text-slate-500 font-medium">Remember me</span>
          </label>
          <a href="#" class="text-sm font-bold text-brand-600 hover:text-brand-700">Forgot password?</a>
        </div>

        <button 
          type="submit"
          :disabled="authStore.loading"
          class="w-full bg-brand-500 hover:bg-brand-600 text-white py-4 rounded-2xl font-bold flex items-center justify-center gap-2 shadow-xl shadow-brand-500/20 transition-all hover:-translate-y-1 active:translate-y-0 disabled:opacity-70"
        >
          <Loader2 v-if="authStore.loading" class="w-5 h-5 animate-spin" />
          <template v-else>
            Sign in <ArrowRight class="w-5 h-5" />
          </template>
        </button>
      </form>

      <div class="mt-10 text-center">
        <p class="text-slate-500 text-sm font-medium">
          Don't have an account? 
          <a href="#" class="text-brand-600 font-bold hover:text-brand-700">Create one for free</a>
        </p>
      </div>
    </div>
  </div>
</template>
