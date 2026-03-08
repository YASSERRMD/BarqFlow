import { defineStore } from 'pinia';

import api from '../api';

export const useNodeStore = defineStore('nodes', {
    state: () => ({
        nodeTypes: [] as any[],
        isLoading: false,
        error: null as string | null,
    }),
    actions: {
        async fetchNodeTypes() {
            if (this.nodeTypes.length > 0) return;
            this.isLoading = true;
            this.error = null;
            try {
                const response = await api.get('/nodes');
                this.nodeTypes = response.data.map((node: any) => {
                    let category = 'Integrations';
                    if (node.name.includes('Trigger') || node.name.includes('webhook')) category = 'Triggers';
                    else if (node.name.includes('n8n-nodes-base') || node.name === 'barqflow-nodes.executeWorkflow') category = 'Core Logic';

                    return {
                        name: node.display_name || node.name,
                        type: node.name.includes('Trigger') ? 'trigger' : 'action',
                        category,
                        description: node.description,
                        schema: node
                    };
                });
            } catch (err: any) {
                this.error = err.response?.data?.message || 'Failed to load nodes';
            } finally {
                this.isLoading = false;
            }
        }
    }
});
