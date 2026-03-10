import http from '../../api'
import type {
  ApprovePromotionRequestPayload,
  AuditLogRecord,
  CreatePromotionRequestPayload,
  CreatePromotionTargetRequest,
  CreateSecretProviderRequest,
  PromotionRequestRecord,
  PromotionTargetRecord,
  SecretProviderRecord,
  UpdateWorkspacePolicyRequest,
  WorkspacePolicyRecord,
} from '../../types/contracts'

export function listAuditLogs(limit = 50) {
  return http.get<AuditLogRecord[]>('/governance/audit-logs', { params: { limit } })
}

export function listSecretProviders() {
  return http.get<SecretProviderRecord[]>('/governance/secret-providers')
}

export function createSecretProvider(payload: CreateSecretProviderRequest) {
  return http.post<SecretProviderRecord>('/governance/secret-providers', payload)
}

export function validateSecretProvider(id: string) {
  return http.post<SecretProviderRecord>(`/governance/secret-providers/${id}/validate`)
}

export function getWorkspacePolicy() {
  return http.get<WorkspacePolicyRecord>('/governance/workspace-policy')
}

export function updateWorkspacePolicy(payload: UpdateWorkspacePolicyRequest) {
  return http.put<WorkspacePolicyRecord>('/governance/workspace-policy', payload)
}

export function listPromotionTargets() {
  return http.get<PromotionTargetRecord[]>('/governance/promotion-targets')
}

export function createPromotionTarget(payload: CreatePromotionTargetRequest) {
  return http.post<PromotionTargetRecord>('/governance/promotion-targets', payload)
}

export function listPromotionRequests(limit = 50) {
  return http.get<PromotionRequestRecord[]>('/governance/promotion-requests', {
    params: { limit },
  })
}

export function createPromotionRequest(payload: CreatePromotionRequestPayload) {
  return http.post<PromotionRequestRecord>('/governance/promotion-requests', payload)
}

export function approvePromotionRequest(id: string, payload: ApprovePromotionRequestPayload = {}) {
  return http.post<PromotionRequestRecord>(`/governance/promotion-requests/${id}/approve`, payload)
}
