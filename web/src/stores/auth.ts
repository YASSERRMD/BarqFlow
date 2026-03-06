import { defineStore } from 'pinia';
import api from '../api';

export const useAuthStore = defineStore('auth', {
    state: () => ({
        user: null as any,
        token: localStorage.getItem('token') || null,
        loading: false,
        error: null as string | null,
    }),
    getters: {
        isAuthenticated: (state) => !!state.token,
    },
    actions: {
        async login(credentials: any) {
            this.loading = true;
            this.error = null;
            try {
                const response = await api.post('/login', credentials);
                this.token = response.data.token;
                this.user = response.data.user;
                localStorage.setItem('token', this.token as string);
                return true;
            } catch (err: any) {
                this.error = err.response?.data?.message || 'Login failed';
                return false;
            } finally {
                this.loading = false;
            }
        },
        logout() {
            this.token = null;
            this.user = null;
            localStorage.removeItem('token');
        },
        async fetchMe() {
            if (!this.token) return;
            try {
                const response = await api.get('/users/me');
                this.user = response.data;
            } catch (err) {
                this.logout();
            }
        }
    },
});
