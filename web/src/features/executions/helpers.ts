import type { ExecutionEvent, ExecutionRecord } from '../../types/contracts'

export const EXECUTION_META_KEY = '__barqflow'

export interface ExecutionNodeResult {
  nodeName: string
  success: boolean
  error?: string | null
  outputsCount: number
}

export interface ExecutionWaitDetails {
  nodeName?: string | null
  waitType?: string | null
  durationMs?: number | null
  resumeToken?: string | null
  resumeUrl?: string | null
  expiresAt?: string | null
}

export function mergeExecutionEvents(
  existing: ExecutionEvent[] = [],
  incoming: ExecutionEvent[] = [],
): ExecutionEvent[] {
  const bySequence = new Map<number, ExecutionEvent>()
  for (const event of [...existing, ...incoming]) {
    bySequence.set(Number(event.sequence), event)
  }
  return Array.from(bySequence.values()).sort((left, right) => left.sequence - right.sequence)
}

export function extractExecutionEvents(record?: ExecutionRecord | null): ExecutionEvent[] {
  const meta = extractExecutionMeta(record)
  const events = Array.isArray(meta?.events) ? (meta.events as ExecutionEvent[]) : []
  return mergeExecutionEvents([], events)
}

export function extractExecutionMeta(record?: ExecutionRecord | null): Record<string, unknown> | null {
  const raw = record?.data?.[EXECUTION_META_KEY]
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null
  return raw as Record<string, unknown>
}

export function extractExecutionNodeResults(record?: ExecutionRecord | null): ExecutionNodeResult[] {
  const rawData = record?.data
  if (!rawData || typeof rawData !== 'object' || Array.isArray(rawData)) return []

  return Object.entries(rawData)
    .filter(([key, value]) => {
      if (key === EXECUTION_META_KEY) return false
      if (!value || typeof value !== 'object' || Array.isArray(value)) return false
      return 'success' in value || 'outputs' in value || 'error' in value
    })
    .map(([nodeName, value]) => {
      const nodeResult = value as { success?: boolean; error?: string | null; outputs?: unknown[] }
      const outputs = Array.isArray(nodeResult.outputs) ? nodeResult.outputs : []
      const outputsCount = outputs.reduce<number>((count, branch) => {
        return count + (Array.isArray(branch) ? branch.length : 0)
      }, 0)

      return {
        nodeName,
        success: nodeResult.success !== false,
        error: nodeResult.error || null,
        outputsCount,
      }
    })
}

export function extractExecutionWaitDetails(record?: ExecutionRecord | null): ExecutionWaitDetails | null {
  const rawData = record?.data as Record<string, unknown> | undefined
  if (!rawData || rawData.waiting !== true) return null

  return {
    nodeName: typeof rawData.nodeName === 'string' ? rawData.nodeName : null,
    waitType: typeof rawData.waitType === 'string' ? rawData.waitType : null,
    durationMs: typeof rawData.durationMs === 'number' ? rawData.durationMs : null,
    resumeToken: typeof rawData.resumeToken === 'string' ? rawData.resumeToken : null,
    resumeUrl: typeof rawData.resumeUrl === 'string' ? rawData.resumeUrl : null,
    expiresAt: typeof rawData.expiresAt === 'string' ? rawData.expiresAt : null,
  }
}

export function resolveExecutionStatusFromEvent(event: ExecutionEvent): string {
  if (event.eventType === 'completed') return 'success'
  if (event.eventType === 'failed') return 'failed'
  if (event.eventType === 'stopped') return 'stopped'
  if (event.eventType === 'waiting') return 'waiting'
  if (event.eventType === 'queued') return 'queued'
  return event.status
}

export function isTerminalExecutionEvent(event: ExecutionEvent): boolean {
  return event.eventType === 'waiting' || ['completed', 'failed', 'stopped'].includes(event.eventType)
}
