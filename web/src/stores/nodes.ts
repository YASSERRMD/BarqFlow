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
                    let category = 'Core';
                    const name = node.name || '';
                    if (node.is_trigger || name.toLowerCase().includes('trigger') || name.includes('webhook') || name.includes('manual')) {
                        category = 'Triggers';
                    } else if (name.startsWith('barqflow-nodes') && !name.includes('trigger') && !name.includes('webhook') && !name.includes('wait') && !name.includes('executeWorkflow')) {
                        category = 'Integrations';
                    } else if (name.includes('set') || name.includes('filter') || name.includes('itemLists') || name.includes('code') || name.includes('merge') || name.includes('switch') || name.includes('if')) {
                        category = 'Data & Logic';
                    }

                    return {
                        name: node.display_name || node.name,
                        type: node.name,
                        kind: node.is_trigger ? 'trigger' : (category === 'Data & Logic' ? 'manipulation' : 'action'),
                        description: node.description,
                        isTrigger: !!node.is_trigger,
                        schema: node,
                        category
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
