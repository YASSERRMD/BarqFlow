import http from '../../api'
import type {
  AddWorkspaceMemberRequest,
  ApiKeyCreateResult,
  ApiKeyRecord,
  ChangePasswordRequest,
  CreateApiKeyRequest,
  OperationsOverview,
  PruneExecutionsResult,
  RuntimeSettings,
  UserProfile,
  WorkspaceMember,
  WorkspaceSummary,
} from '../../types/contracts'

export interface CreateWorkspacePayload {
  name: string
}

export function getRuntimeSettings() {
  return http.get<RuntimeSettings>('/settings/runtime')
}

export function getOperationsOverview() {
  return http.get<OperationsOverview>('/settings/operations')
}

export function pruneExecutions() {
  return http.post<PruneExecutionsResult>('/settings/operations/prune')
}

export function listWorkspaces() {
  return http.get<WorkspaceSummary[]>('/workspaces')
}

export function getCurrentWorkspace() {
  return http.get<WorkspaceSummary>('/workspaces/current')
}

export function createWorkspace(payload: CreateWorkspacePayload) {
  return http.post<WorkspaceSummary>('/workspaces', payload)
}

export function selectWorkspace(workspaceId: string) {
  return http.post<WorkspaceSummary>(`/workspaces/${workspaceId}/select`)
}

export function listWorkspaceMembers() {
  return http.get<WorkspaceMember[]>('/workspaces/current/members')
}

export function addWorkspaceMember(payload: AddWorkspaceMemberRequest) {
  return http.post<WorkspaceMember>('/workspaces/current/members', payload)
}

export function listApiKeys() {
  return http.get<ApiKeyRecord[]>('/api-keys')
}

export function createApiKey(payload: CreateApiKeyRequest) {
  return http.post<ApiKeyCreateResult>('/api-keys', payload)
}

export function revokeApiKey(apiKeyId: string) {
  return http.delete(`/api-keys/${apiKeyId}`)
}

export function changePassword(payload: ChangePasswordRequest) {
  return http.post<UserProfile>('/users/change-password', payload)
}
