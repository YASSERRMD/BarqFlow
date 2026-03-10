import { defineStore } from 'pinia'
import { fetchProfile, login, register, type AuthCredentials } from './api'
import type { AuthResponse, UserProfile } from '../../types/contracts'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as UserProfile | null,
    token: localStorage.getItem('token') || null,
    loading: false,
    error: null as string | null,
  }),
  getters: {
    isAuthenticated: (state) => !!state.token,
    activeWorkspace: (state) => state.user?.activeWorkspace ?? null,
    userName: (state) => {
      const firstName = state.user?.firstName?.trim()
      const lastName = state.user?.lastName?.trim()
      const fullName = [firstName, lastName].filter(Boolean).join(' ')
      if (fullName) return fullName
      const fallback = state.user?.email?.split('@')[0]
      return fallback ? fallback.charAt(0).toUpperCase() + fallback.slice(1) : 'Workspace User'
    },
  },
  actions: {
    applyAuthResponse(response: AuthResponse) {
      this.token = response.token
      this.user = response.user
      localStorage.setItem('token', response.token)
    },
    async login(credentials: AuthCredentials) {
      this.loading = true
      this.error = null
      try {
        const response = await login(credentials)
        this.applyAuthResponse(response.data)
        return true
      } catch (err: any) {
        this.error = err.response?.data?.message || 'Login failed'
        return false
      } finally {
        this.loading = false
      }
    },
    async register(credentials: AuthCredentials) {
      this.loading = true
      this.error = null
      try {
        const response = await register(credentials)
        this.applyAuthResponse(response.data)
        return true
      } catch (err: any) {
        this.error = err.response?.data?.message || 'Registration failed'
        return false
      } finally {
        this.loading = false
      }
    },
    logout() {
      this.token = null
      this.user = null
      localStorage.removeItem('token')
    },
    async fetchMe() {
      if (!this.token) return
      try {
        const response = await fetchProfile()
        this.user = response.data
      } catch {
        this.logout()
      }
    },
  },
})
