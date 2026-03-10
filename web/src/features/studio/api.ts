import http from '../../api'
import type {
  ExtensionActionInvocationRecord,
  ExtensionBundleRecord,
  InvokeExtensionActionRequest,
  WorkflowDraftRecord,
  WorkflowDraftRequest,
} from '../../types/contracts'

export function listExtensionBundles() {
  return http.get<ExtensionBundleRecord[]>('/studio/extensions')
}

export function invokeExtensionAction(bundleId: string, payload: InvokeExtensionActionRequest) {
  return http.post<ExtensionActionInvocationRecord>(`/studio/extensions/${encodeURIComponent(bundleId)}/invoke`, payload)
}

export function generateWorkflowDraft(payload: WorkflowDraftRequest) {
  return http.post<WorkflowDraftRecord>('/studio/workflow-drafts', payload)
}
