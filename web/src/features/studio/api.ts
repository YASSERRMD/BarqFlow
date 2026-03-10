import http from '../../api'
import type { ExtensionBundleRecord, WorkflowDraftRecord, WorkflowDraftRequest } from '../../types/contracts'

export function listExtensionBundles() {
  return http.get<ExtensionBundleRecord[]>('/studio/extensions')
}

export function generateWorkflowDraft(payload: WorkflowDraftRequest) {
  return http.post<WorkflowDraftRecord>('/studio/workflow-drafts', payload)
}
