import http from '../../api'
import type { CredentialSummary, CredentialTypeContract } from '../../types/contracts'

export function listCredentials(params?: { type?: string }) {
  return http.get<CredentialSummary[]>('/credentials', { params })
}

export function listCredentialTypes() {
  return http.get<CredentialTypeContract[]>('/credentials/types')
}

export function createCredential(payload: {
  name: string
  credentialType: string
  data: Record<string, unknown>
}) {
  return http.post<CredentialSummary>('/credentials', payload)
}

export function updateCredential(
  id: string,
  payload: {
    name?: string
    data?: Record<string, unknown>
  },
) {
  return http.put<CredentialSummary>(`/credentials/${id}`, payload)
}

export function testCredentialType(payload: {
  credentialType: string
  data: Record<string, unknown>
}) {
  return http.post<{ valid: boolean }>('/credentials/test', payload)
}

export function testSavedCredentialById(id: string) {
  return http.post<{ valid: boolean; credentialId?: string; credentialType?: string }>(
    `/credentials/${id}/test`,
  )
}

export function deleteCredentialById(id: string) {
  return http.delete(`/credentials/${id}`)
}
