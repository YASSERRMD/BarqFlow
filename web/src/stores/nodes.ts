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
                this.nodeTypes = response.data.map((node: any) => ({
                    name: node.displayName,
                    type: node.name.includes('Trigger') ? 'trigger' : (node.name.includes('Set') ? 'manipulation' : 'action'),
                    description: node.description,
                    icon: node.name.includes('HTTP') ? 'Globe' : 'Settings2',
                    schema: node
                }));
            } catch (err: any) {
                this.error = err.response?.data?.message || 'Failed to load nodes';
            } finally {
                this.isLoading = false;
            }
        }
    }
});
