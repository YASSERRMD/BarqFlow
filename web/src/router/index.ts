import { createRouter, createWebHistory } from 'vue-router'
import api from '../api'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/',
            redirect: '/workflows'
        },
        {
            path: '/login',
            name: 'Login',
            component: () => import('../views/Login.vue')
        },
        {
            path: '/workflows',
            name: 'Workflows',
            component: () => import('../views/WorkflowList.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/workflow/:id',
            name: 'WorkflowEditor',
            component: () => import('../views/WorkflowEditor.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/executions',
            name: 'Executions',
            component: () => import('../views/ExecutionViewer.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/observability',
            name: 'Observability',
            component: () => import('../views/Observability.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/governance',
            name: 'Governance',
            component: () => import('../views/Governance.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/studio',
            name: 'AutomationStudio',
            component: () => import('../views/AutomationStudio.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/credentials',
            name: 'Credentials',
            component: () => import('../views/Credentials.vue'),
            meta: { requiresAuth: true },
        },
        {
            path: '/settings',
            name: 'Settings',
            component: () => import('../views/Settings.vue'),
            meta: { requiresAuth: true },
        }
    ]
})

// Track whether the session has been validated this page-load so we only
// hit the server once, not on every navigation.
let sessionValidated = false

router.beforeEach(async (to) => {
    const requiresAuth = to.matched.some((record) => record.meta.requiresAuth)
    const hasToken = !!localStorage.getItem('token')

    // Already logged in and navigating to /login → redirect to app.
    if (to.path === '/login' && hasToken) {
        return '/workflows'
    }

    // No token at all → go to /login.
    if (requiresAuth && !hasToken) {
        return '/login'
    }

    // Token present on an auth-protected route.
    if (requiresAuth && hasToken && !sessionValidated) {
        try {
            // Validate the token server-side on the first protected navigation.
            await api.get('/users/me')
            sessionValidated = true
        } catch {
            // Server rejected the token (expired, revoked, etc.) — clean up and redirect.
            localStorage.removeItem('token')
            sessionValidated = false
            return '/login'
        }
    }

    return true
})

export default router
