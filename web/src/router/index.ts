import { createRouter, createWebHistory } from 'vue-router'
import WorkflowEditor from '../views/WorkflowEditor.vue'
import ExecutionViewer from '../views/ExecutionViewer.vue'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            redirect: '/workflows'
        },
        {
            path: '/workflows',
            name: 'workflows',
            component: WorkflowEditor
        },
        {
            path: '/executions',
            name: 'executions',
            component: ExecutionViewer
        }
    ]
})

export default router
