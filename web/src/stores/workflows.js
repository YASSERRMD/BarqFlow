import { defineStore } from 'pinia';
import api from '../api';
export const useWorkflowStore = defineStore('workflows', {
    state: () => ({
        workflows: [],
        activeWorkflow: null,
        executions: [],
        loading: false,
        error: null,
    }),
    actions: {
        async fetchWorkflows(params = {}) {
            this.loading = true;
            try {
                const response = await api.get('/workflows', { params });
                this.workflows = response.data;
            }
            catch (err) {
                this.error = err.message;
            }
            finally {
                this.loading = false;
            }
        },
        async fetchWorkflow(id) {
            this.loading = true;
            try {
                const response = await api.get(`/workflows/${id}`);
                this.activeWorkflow = response.data;
            }
            catch (err) {
                this.error = err.message;
            }
            finally {
                this.loading = false;
            }
        },
        async saveWorkflow(workflow) {
            try {
                if (workflow.id) {
                    const response = await api.put(`/workflows/${workflow.id}`, workflow);
                    this.workflows = this.workflows.map((wf) => wf.id === workflow.id ? response.data : wf);
                    this.activeWorkflow = response.data;
                    return response.data;
                }
                const response = await api.post('/workflows', workflow);
                this.workflows.push(response.data);
                this.activeWorkflow = response.data;
                return response.data;
            }
            catch (err) {
                this.error = err.message;
                throw err;
            }
        },
        async deleteWorkflow(id) {
            try {
                await api.delete(`/workflows/${id}`);
                this.workflows = this.workflows.filter((wf) => wf.id !== id);
                if (this.activeWorkflow?.id === id) {
                    this.activeWorkflow = null;
                }
            }
            catch (err) {
                this.error = err.message;
                throw err;
            }
        },
        async toggleWorkflowActive(id, active) {
            try {
                const response = await api.put(`/workflows/${id}/activate`, { active });
                this.workflows = this.workflows.map((wf) => wf.id === id ? response.data : wf);
                if (this.activeWorkflow?.id === id) {
                    this.activeWorkflow = response.data;
                }
                return response.data;
            }
            catch (err) {
                this.error = err.message;
                throw err;
            }
        },
        async duplicateWorkflow(id) {
            try {
                const response = await api.post(`/workflows/${id}/duplicate`);
                this.workflows.unshift(response.data);
                return response.data;
            }
            catch (err) {
                this.error = err.message;
                throw err;
            }
        },
        async executeWorkflow(workflowId, payload = {}) {
            this.loading = true;
            try {
                const response = await api.post(`/executions/workflow/${workflowId}`, payload);
                return response.data;
            }
            catch (err) {
                this.error = err.message;
                throw err;
            }
            finally {
                this.loading = false;
            }
        },
        async executeWorkflowToNode(workflowId, nodeId, payload = {}) {
            this.loading = true;
            try {
                const response = await api.post(`/executions/workflow/${workflowId}/test-node/${encodeURIComponent(nodeId)}`, payload);
                return response.data;
            }
            catch (err) {
                this.error = err.message;
                throw err;
            }
            finally {
                this.loading = false;
            }
        },
    },
});
//# sourceMappingURL=workflows.js.map