import http from '../../api'
import type {
  CredentialOAuthConnectResult,
  CredentialSummary,
  CredentialTypeContract,
  CredentialValidationResult,
} from '../../types/contracts'

export function listCredentials(params?: { type?: string }) {
  return http.get<CredentialSummary[]>('/credentials', { params })
}

export function listCredentialTypes() {
  return http.get<CredentialTypeContract[]>('/credentials/types')
}

export function getCredentialById(id: string) {
  return http.get<CredentialSummary>(`/credentials/${id}`)
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

export function rotateCredential(
  id: string,
  payload: {
    name?: string
    data?: Record<string, unknown>
  },
) {
  return http.post<CredentialSummary>(`/credentials/${id}/rotate`, payload)
}

export function testCredentialType(payload: {
  credentialType: string
  data: Record<string, unknown>
}) {
  return http.post<CredentialValidationResult>('/credentials/test', payload)
}

export function testSavedCredentialById(id: string) {
  return http.post<CredentialValidationResult>(`/credentials/${id}/test`)
}

export function startCredentialOAuthConnect(id: string) {
  return http.post<CredentialOAuthConnectResult>(`/credentials/${id}/oauth2/connect`)
}

export function deleteCredentialById(id: string) {
  return http.delete(`/credentials/${id}`)
}
