import http from '../../api'
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

export function listWorkflows(
  params: {
    active?: boolean
    search?: string
    tags?: string[]
    limit?: number
    sortBy?: 'updatedAt' | 'createdAt' | 'name'
    sortDirection?: 'asc' | 'desc'
  } = {},
) {
  const normalizedParams = {
    ...params,
    tags: params.tags && params.tags.length > 0 ? params.tags.join(',') : undefined,
  }
  return http.get<WorkflowRecord[]>('/workflows', { params: normalizedParams })
}

export function getWorkflow(id: string) {
  return http.get<WorkflowRecord>(`/workflows/${id}`)
}

export function saveWorkflow(workflow: WorkflowUpsertRequest & { id?: string }) {
  if (workflow.id) {
    return http.put<WorkflowRecord>(`/workflows/${workflow.id}`, workflow)
  }

  return http.post<WorkflowRecord>('/workflows', workflow)
}

export function importWorkflow(payload: WorkflowImportRequest) {
  return http.post<WorkflowRecord>('/workflows/import', payload)
}

export function exportWorkflow(id: string) {
  return http.get<WorkflowExportRecord>(`/workflows/${id}/export`)
}

export function deleteWorkflow(id: string) {
  return http.delete(`/workflows/${id}`)
}

export function setWorkflowActive(id: string, active: boolean) {
  return http.put<WorkflowRecord>(`/workflows/${id}/activate`, { active })
}

export function duplicateWorkflow(id: string) {
  return http.post<WorkflowRecord>(`/workflows/${id}/duplicate`)
}

export function listWorkflowHistory(id: string) {
  return http.get<WorkflowHistoryEntry[]>(`/workflows/${id}/history`)
}

export function getWorkflowHistoryDiff(id: string, fromVersion: number, toVersion: number) {
  return http.get<WorkflowHistoryDiff>(`/workflows/${id}/history/diff`, {
    params: {
      fromVersion,
      toVersion,
    },
  })
}

export function listWorkflowTemplates() {
  return http.get<WorkflowTemplateRecord[]>('/workflow-templates')
}

export function instantiateWorkflowTemplate(id: string, payload: { name?: string } = {}) {
  return http.post<WorkflowRecord>(`/workflow-templates/${encodeURIComponent(id)}/instantiate`, payload)
}

export function listWorkflowTags() {
  return http.get<TagRecord[]>('/tags')
}

export function createWorkflowTag(name: string) {
  return http.post<TagRecord>('/tags', { name })
}

export function deleteWorkflowTag(id: string) {
  return http.delete(`/tags/${id}`)
}

export function executeWorkflow(workflowId: string, payload: CreateExecutionRequest = {}) {
  return http.post<ExecutionRecord>(`/executions/workflow/${workflowId}`, payload)
}

export function executeWorkflowToNode(
  workflowId: string,
  nodeId: string,
  payload: CreateExecutionRequest = {},
) {
  return http.post<ExecutionRecord>(
    `/executions/workflow/${workflowId}/test-node/${encodeURIComponent(nodeId)}`,
    payload,
  )
}
