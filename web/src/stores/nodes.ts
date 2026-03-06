import { defineStore } from 'pinia';

export const useNodeStore = defineStore('nodes', {
    state: () => ({
        nodeTypes: [
            {
                name: 'Manual Trigger',
                type: 'trigger',
                description: 'Triggers the workflow manually.',
                icon: 'MousePointerClick'
            },
            {
                name: 'HTTP Request',
                type: 'action',
                description: 'Sends an HTTP request.',
                icon: 'Globe'
            },
            {
                name: 'IF',
                type: 'logic',
                description: 'Branches the flow based on a condition.',
                icon: 'Split'
            },
            {
                name: 'Set',
                type: 'manipulation',
                description: 'Sets data values in the flow.',
                icon: 'Settings2'
            },
            {
                name: 'Code',
                type: 'action',
                description: 'Executes custom Rhai code.',
                icon: 'Code'
            },
            {
                name: 'Merge',
                type: 'logic',
                description: 'Merges multi-branch results into one.',
                icon: 'GitMerge'
            }
        ] as any[],
    }),
});
