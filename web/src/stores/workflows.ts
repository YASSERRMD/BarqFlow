import { defineStore } from 'pinia';
import api from '../api';

export const useWorkflowStore = defineStore('workflows', {
    state: () => ({
        workflows: [] as any[],
        activeWorkflow: null as any,
        executions: [] as any[],
        loading: false,
        error: null as string | null,
    }),
    actions: {
        async fetchWorkflows() {
            this.loading = true;
            try {
                const response = await api.get('/workflows');
                this.workflows = response.data;
            } catch (err: any) {
                this.error = err.message;
            } finally {
                this.loading = false;
            }
        },
        async fetchWorkflow(id: string) {
            this.loading = true;
            try {
                const response = await api.get(`/workflows/${id}`);
                this.activeWorkflow = response.data;
            } catch (err: any) {
                this.error = err.message;
            } finally {
                this.loading = false;
            }
        },
        async saveWorkflow(workflow: any) {
            try {
                if (workflow.id) {
                    await api.put(`/workflows/${workflow.id}`, workflow);
                } else {
                    const response = await api.post('/workflows', workflow);
                    this.workflows.push(response.data);
                }
            } catch (err: any) {
                this.error = err.message;
            }
        },
        async executeWorkflow(workflowId: string, payload: any = {}) {
            this.loading = true;
            try {
                const response = await api.post(`/executions/workflow/${workflowId}`, payload);
                return response.data;
            } catch (err: any) {
                this.error = err.message;
                throw err;
            } finally {
                this.loading = false;
            }
        }
    },
});
