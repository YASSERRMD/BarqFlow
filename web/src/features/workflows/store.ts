import { defineStore } from 'pinia'
import {
  deleteWorkflow,
  duplicateWorkflow,
  executeWorkflow,
  executeWorkflowToNode,
  getWorkflow,
  listWorkflows,
  saveWorkflow,
  setWorkflowActive,
} from './api'
import type {
  CreateExecutionRequest,
  ExecutionRecord,
  WorkflowRecord,
  WorkflowUpsertRequest,
} from '../../types/contracts'

export const useWorkflowStore = defineStore('workflows', {
  state: () => ({
    workflows: [] as WorkflowRecord[],
    activeWorkflow: null as WorkflowRecord | null,
    executions: [] as ExecutionRecord[],
    loading: false,
    error: null as string | null,
  }),
  actions: {
    async fetchWorkflows(params: {
      active?: boolean
      search?: string
      limit?: number
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
          this.workflows.push(response.data)
        }

        this.activeWorkflow = response.data
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
