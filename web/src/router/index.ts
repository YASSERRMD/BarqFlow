import { createRouter, createWebHistory } from 'vue-router'
import WorkflowEditor from '../views/WorkflowEditor.vue'
import ExecutionViewer from '../views/ExecutionViewer.vue'

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
            component: () => import('../views/WorkflowList.vue')
        },
        {
            path: '/workflow/:id',
            name: 'WorkflowEditor',
            component: WorkflowEditor
        },
        {
            path: '/executions',
            name: 'Executions',
            component: ExecutionViewer
        },
        {
            path: '/credentials',
            name: 'Credentials',
            component: () => import('../views/Credentials.vue')
        },
        {
            path: '/settings',
            name: 'Settings',
            component: () => import('../views/Settings.vue')
        }
    ]
})

export default router
