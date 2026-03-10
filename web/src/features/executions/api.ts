import http from '../../api'
import type { ExecutionRecord } from '../../types/contracts'

export function listExecutions(params: {
  workflowId?: string
  status?: string
  limit?: number
} = {}) {
  return http.get<ExecutionRecord[]>('/executions', { params })
}

export function getExecution(id: string) {
  return http.get<ExecutionRecord>(`/executions/${id}`)
}

export function retryExecution(id: string) {
  return http.post<ExecutionRecord>(`/executions/${id}/retry`)
}

export function stopExecution(id: string) {
  return http.post<ExecutionRecord>(`/executions/${id}/stop`)
}

export function deleteExecution(id: string) {
  return http.delete(`/executions/${id}`)
}
