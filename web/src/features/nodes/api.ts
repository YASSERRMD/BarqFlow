import http from '../../api'
import type { NodeSchemaContract } from '../../types/contracts'

export function listNodeSchemas() {
  return http.get<NodeSchemaContract[]>('/nodes')
}
