import http from '../../api'
import type { ExecutionEvent, ExecutionRecord } from '../../types/contracts'

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

export function getExecutionEvents(id: string) {
  return http.get<ExecutionEvent[]>(`/executions/${id}/events`)
}

export function createExecutionEventSource(id: string) {
  const token = globalThis.localStorage?.getItem('token')
  const query = token ? `?token=${encodeURIComponent(token)}` : ''
  return new EventSource(`/rest/executions/${encodeURIComponent(id)}/events/stream${query}`)
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
