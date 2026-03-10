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
          const category = node.category || 'Core'

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
            supportTier: node.supportTier,
            supportNote: node.supportNote,
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
