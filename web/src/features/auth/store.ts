import { defineStore } from 'pinia'
import { fetchProfile, login, register, type AuthCredentials } from './api'
import type { UserProfile } from '../../types/contracts'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as UserProfile | null,
    token: localStorage.getItem('token') || null,
    loading: false,
    error: null as string | null,
  }),
  getters: {
    isAuthenticated: (state) => !!state.token,
  },
  actions: {
    async login(credentials: AuthCredentials) {
      this.loading = true
      this.error = null
      try {
        const response = await login(credentials)
        this.token = response.data.token
        this.user =
          response.data.user ??
          (response.data.userId
            ? { id: response.data.userId, email: credentials.email, role: 'user' }
            : null)
        localStorage.setItem('token', this.token as string)
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
        this.token = response.data.token
        this.user =
          response.data.user ??
          (response.data.userId
            ? { id: response.data.userId, email: credentials.email, role: 'user' }
            : null)
        localStorage.setItem('token', this.token as string)
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
