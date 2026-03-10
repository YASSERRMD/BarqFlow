import { defineStore } from 'pinia'
import { listNodeSchemas } from './api'
import type { NodeCatalogEntry, NodeSchemaContract } from '../../types/contracts'

export const useNodeStore = defineStore('nodes', {
  state: () => ({
    nodeTypes: [] as NodeCatalogEntry[],
    isLoading: false,
    error: null as string | null,
  }),
  actions: {
    async fetchNodeTypes() {
      if (this.nodeTypes.length > 0) return
      this.isLoading = true
      this.error = null
      try {
        const response = await listNodeSchemas()
        this.nodeTypes = response.data.map((node: NodeSchemaContract) => {
          let category = 'Core'
          const name = node.name || ''
          if (
            node.isTrigger ||
            name.toLowerCase().includes('trigger') ||
            name.includes('webhook') ||
            name.includes('manual')
          ) {
            category = 'Triggers'
          } else if (
            name.startsWith('barqflow-nodes') &&
            !name.includes('trigger') &&
            !name.includes('webhook') &&
            !name.includes('wait') &&
            !name.includes('executeWorkflow')
          ) {
            category = 'Integrations'
          } else if (
            name.includes('set') ||
            name.includes('filter') ||
            name.includes('itemLists') ||
            name.includes('code') ||
            name.includes('merge') ||
            name.includes('switch') ||
            name.includes('if')
          ) {
            category = 'Data & Logic'
          }

          return {
            name: node.displayName || node.name,
            type: node.name,
            kind: node.isTrigger
              ? 'trigger'
              : category === 'Data & Logic'
                ? 'manipulation'
                : 'action',
            description: node.description,
            isTrigger: !!node.isTrigger,
            schema: node,
            category,
          }
        })
      } catch (err: any) {
        this.error = err.response?.data?.message || 'Failed to load nodes'
      } finally {
        this.isLoading = false
      }
    },
  },
})
