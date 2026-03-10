import { defineStore } from 'pinia'
import {
  createWorkflowTag,
  deleteWorkflow,
  deleteWorkflowTag,
  duplicateWorkflow,
  executeWorkflow,
  executeWorkflowToNode,
  exportWorkflow,
  getWorkflow,
  getWorkflowHistoryDiff,
  importWorkflow,
  instantiateWorkflowTemplate,
  listWorkflowHistory,
  listWorkflowTags,
  listWorkflowTemplates,
  listWorkflows,
  saveWorkflow,
  setWorkflowActive,
} from './api'
import type {
  CreateExecutionRequest,
  ExecutionRecord,
  TagRecord,
  WorkflowExportRecord,
  WorkflowHistoryDiff,
  WorkflowHistoryEntry,
  WorkflowImportRequest,
  WorkflowRecord,
  WorkflowTemplateRecord,
  WorkflowUpsertRequest,
} from '../../types/contracts'

export const useWorkflowStore = defineStore('workflows', {
  state: () => ({
    workflows: [] as WorkflowRecord[],
    activeWorkflow: null as WorkflowRecord | null,
    workflowTags: [] as TagRecord[],
    workflowTemplates: [] as WorkflowTemplateRecord[],
    workflowHistory: {} as Record<string, WorkflowHistoryEntry[]>,
    workflowDiffs: {} as Record<string, WorkflowHistoryDiff>,
    executions: [] as ExecutionRecord[],
    loading: false,
    error: null as string | null,
  }),
  actions: {
    async fetchWorkflows(params: {
      active?: boolean
      search?: string
      tags?: string[]
      limit?: number
      sortBy?: 'updatedAt' | 'createdAt' | 'name'
      sortDirection?: 'asc' | 'desc'
    } = {}) {
      this.loading = true
      try {
        const response = await listWorkflows(params)
        this.workflows = response.data
      } catch (err: any) {
        this.error = err.message
      } finally {
        this.loading = false
      }
    },
    async fetchWorkflow(id: string) {
      this.loading = true
      try {
        const response = await getWorkflow(id)
        this.activeWorkflow = response.data
      } catch (err: any) {
        this.error = err.message
      } finally {
        this.loading = false
      }
    },
    async saveWorkflow(workflow: WorkflowUpsertRequest & { id?: string }) {
      try {
        const response = await saveWorkflow(workflow)

        if (workflow.id) {
          this.workflows = this.workflows.map((wf) =>
            wf.id === workflow.id ? response.data : wf,
          )
        } else {
          this.workflows.unshift(response.data)
        }

        this.activeWorkflow = response.data
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async importWorkflow(payload: WorkflowImportRequest) {
      try {
        const response = await importWorkflow(payload)
        this.workflows.unshift(response.data)
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async exportWorkflow(id: string): Promise<WorkflowExportRecord> {
      try {
        const response = await exportWorkflow(id)
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async deleteWorkflow(id: string) {
      try {
        await deleteWorkflow(id)
        this.workflows = this.workflows.filter((wf) => wf.id !== id)
        if (this.activeWorkflow?.id === id) {
          this.activeWorkflow = null
        }
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async toggleWorkflowActive(id: string, active: boolean) {
      try {
        const response = await setWorkflowActive(id, active)
        this.workflows = this.workflows.map((wf) =>
          wf.id === id ? response.data : wf,
        )
        if (this.activeWorkflow?.id === id) {
          this.activeWorkflow = response.data
        }
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async duplicateWorkflow(id: string) {
      try {
        const response = await duplicateWorkflow(id)
        this.workflows.unshift(response.data)
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async fetchWorkflowHistory(id: string) {
      try {
        const response = await listWorkflowHistory(id)
        this.workflowHistory[id] = response.data
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async fetchWorkflowHistoryDiff(id: string, fromVersion: number, toVersion: number) {
      try {
        const response = await getWorkflowHistoryDiff(id, fromVersion, toVersion)
        this.workflowDiffs[`${id}:${fromVersion}:${toVersion}`] = response.data
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async fetchWorkflowTemplates() {
      try {
        const response = await listWorkflowTemplates()
        this.workflowTemplates = response.data
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async instantiateWorkflowTemplate(id: string, name?: string) {
      try {
        const response = await instantiateWorkflowTemplate(id, { name })
        this.workflows.unshift(response.data)
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async fetchWorkflowTags() {
      try {
        const response = await listWorkflowTags()
        this.workflowTags = response.data
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async createWorkflowTag(name: string) {
      try {
        const response = await createWorkflowTag(name)
        const existing = this.workflowTags.find((tag) => tag.id === response.data.id)
        if (existing) {
          this.workflowTags = this.workflowTags.map((tag) =>
            tag.id === response.data.id ? response.data : tag,
          )
        } else {
          this.workflowTags.push(response.data)
          this.workflowTags.sort((left, right) => left.name.localeCompare(right.name))
        }
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async deleteWorkflowTag(id: string) {
      try {
        await deleteWorkflowTag(id)
        this.workflowTags = this.workflowTags.filter((tag) => tag.id !== id)
      } catch (err: any) {
        this.error = err.message
        throw err
      }
    },
    async executeWorkflow(workflowId: string, payload: CreateExecutionRequest = {}) {
      this.loading = true
      try {
        const response = await executeWorkflow(workflowId, payload)
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      } finally {
        this.loading = false
      }
    },
    async executeWorkflowToNode(
      workflowId: string,
      nodeId: string,
      payload: CreateExecutionRequest = {},
    ) {
      this.loading = true
      try {
        const response = await executeWorkflowToNode(workflowId, nodeId, payload)
        return response.data
      } catch (err: any) {
        this.error = err.message
        throw err
      } finally {
        this.loading = false
      }
    },
  },
})
