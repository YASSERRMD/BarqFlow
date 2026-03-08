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
        async fetchWorkflows(params: {
            active?: boolean;
            search?: string;
            limit?: number;
        } = {}) {
            this.loading = true;
            try {
                const response = await api.get('/workflows', { params });
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
                    const response = await api.put(`/workflows/${workflow.id}`, workflow);
                    this.workflows = this.workflows.map((wf) =>
                        wf.id === workflow.id ? response.data : wf
                    );
                    this.activeWorkflow = response.data;
                } else {
                    const response = await api.post('/workflows', workflow);
                    this.workflows.push(response.data);
                    this.activeWorkflow = response.data;
                }
            } catch (err: any) {
                this.error = err.message;
            }
        },
        async deleteWorkflow(id: string) {
            try {
                await api.delete(`/workflows/${id}`);
                this.workflows = this.workflows.filter((wf) => wf.id !== id);
                if (this.activeWorkflow?.id === id) {
                    this.activeWorkflow = null;
                }
            } catch (err: any) {
                this.error = err.message;
                throw err;
            }
        },
        async toggleWorkflowActive(id: string, active: boolean) {
            try {
                const response = await api.put(`/workflows/${id}/activate`, { active });
                this.workflows = this.workflows.map((wf) =>
                    wf.id === id ? response.data : wf
                );
                if (this.activeWorkflow?.id === id) {
                    this.activeWorkflow = response.data;
                }
                return response.data;
            } catch (err: any) {
                this.error = err.message;
                throw err;
            }
        },
        async duplicateWorkflow(id: string) {
            try {
                const response = await api.post(`/workflows/${id}/duplicate`);
                this.workflows.unshift(response.data);
                return response.data;
            } catch (err: any) {
                this.error = err.message;
                throw err;
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
