import { defineStore } from 'pinia';
import api from '../api';
import type { AuthResponse, UserProfile } from '../types/contracts';

interface AuthCredentials {
    email: string;
    password: string;
    firstName?: string;
    lastName?: string;
}

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
            this.loading = true;
            this.error = null;
            try {
                const response = await api.post<AuthResponse>('/login', credentials);
                this.token = response.data.token;
                this.user = response.data.user ?? (response.data.userId ? { id: response.data.userId, email: credentials.email, role: 'user' } : null);
                localStorage.setItem('token', this.token as string);
                return true;
            } catch (err: any) {
                this.error = err.response?.data?.message || 'Login failed';
                return false;
            } finally {
                this.loading = false;
            }
        },
        async register(credentials: AuthCredentials) {
            this.loading = true;
            this.error = null;
            try {
                const response = await api.post<AuthResponse>('/users', credentials);
                this.token = response.data.token;
                this.user = response.data.user ?? (response.data.userId ? { id: response.data.userId, email: credentials.email, role: 'user' } : null);
                localStorage.setItem('token', this.token as string);
                return true;
            } catch (err: any) {
                this.error = err.response?.data?.message || 'Registration failed';
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
                const response = await api.get<UserProfile>('/users/me');
                this.user = response.data;
            } catch (err) {
                this.logout();
            }
        }
    },
});
