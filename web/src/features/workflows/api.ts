import http from '../../api'
import type {
  CreateExecutionRequest,
  ExecutionRecord,
  WorkflowRecord,
  WorkflowUpsertRequest,
} from '../../types/contracts'

export function listWorkflows(params: {
  active?: boolean
  search?: string
  limit?: number
} = {}) {
  return http.get<WorkflowRecord[]>('/workflows', { params })
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

export function deleteWorkflow(id: string) {
  return http.delete(`/workflows/${id}`)
}

export function setWorkflowActive(id: string, active: boolean) {
  return http.put<WorkflowRecord>(`/workflows/${id}/activate`, { active })
}

export function duplicateWorkflow(id: string) {
  return http.post<WorkflowRecord>(`/workflows/${id}/duplicate`)
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
