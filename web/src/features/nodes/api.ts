import http from '../../api'
import type {
  NodeDynamicOptionsRequest,
  NodeDynamicOptionsResponse,
  NodeSchemaContract,
} from '../../types/contracts'

export function listNodeSchemas() {
  return http.get<NodeSchemaContract[]>('/nodes')
}

export function resolveNodeDynamicOptions(payload: NodeDynamicOptionsRequest) {
  return http.post<NodeDynamicOptionsResponse>('/nodes/dynamic-options', payload)
}
