import type {
  CredentialSummary,
  CredentialTypeContract,
  NodeProperty,
} from '../../types/contracts'

export interface CredentialQuickStart {
  credentialType: string
  title: string
  summary: string
  highlight: string
}

export interface ExternalSecretReference {
  providerId: string
  path: string
  key: string
}

export interface ExternalSecretReferenceEnvelope {
  __secretRef: ExternalSecretReference
}

const SECRET_FIELD_HINTS = ['token', 'secret', 'password', 'key', 'clientsecret', 'apikey']
const DATABASE_TYPE_HINTS = ['postgres', 'mysql', 'redis', 'mongo', 'database']
const DATABASE_FIELD_HINTS = ['host', 'port', 'database', 'user']

export const CREDENTIAL_QUICK_STARTS: CredentialQuickStart[] = [
  {
    credentialType: 'openAiApi',
    title: 'OpenAI',
    summary: 'Create a production AI credential with API key and optional base URL.',
    highlight: 'Best for LLM, embeddings, and assistant workflows.',
  },
  {
    credentialType: 'slackApi',
    title: 'Slack',
    summary: 'Store a Slack token once and bind it across alerts and chat automations.',
    highlight: 'Pairs well with message, incident, and approval flows.',
  },
  {
    credentialType: 'githubApi',
    title: 'GitHub',
    summary: 'Set up GitHub access for repo automation, issues, and release workflows.',
    highlight: 'Use PATs or OAuth-style credentials when you add them.',
  },
  {
    credentialType: 'notionApi',
    title: 'Notion',
    summary: 'Connect Notion workspaces for sync, indexing, and publishing flows.',
    highlight: 'Good for docs, internal wikis, and content operations.',
  },
  {
    credentialType: 'postgresApi',
    title: 'PostgreSQL',
    summary: 'Save a reusable database connection with host, port, and user details.',
    highlight: 'Supports ETL, reporting, and workflow persistence patterns.',
  },
]

export function credentialSupportsOAuthConnect(
  type?: CredentialTypeContract | null,
): boolean {
  if (!type) return false
  const propertyNames = new Set(
    (type.properties || []).map((property) => String(property.name || '').trim()),
  )

  return (
    propertyNames.has('authUrl') &&
    (propertyNames.has('accessTokenUrl') || propertyNames.has('tokenUrl')) &&
    propertyNames.has('clientId')
  )
}

export function isSecretCredentialField(property: NodeProperty): boolean {
  const fieldName = `${String(property.name || '')}${String(property.displayName || '')}`
    .replace(/\s+/g, '')
    .toLowerCase()

  return SECRET_FIELD_HINTS.some((hint) => fieldName.includes(hint))
}

export function credentialAuthKind(
  type?: CredentialTypeContract | null,
): 'oauth' | 'token' | 'database' | 'custom' {
  if (!type) return 'custom'
  if (credentialSupportsOAuthConnect(type)) return 'oauth'

  const typeName = String(type.name || '').toLowerCase()
  const propertyNames = (type.properties || []).map((property) =>
    String(property.name || '').toLowerCase(),
  )

  if (
    DATABASE_TYPE_HINTS.some((hint) => typeName.includes(hint)) ||
    DATABASE_FIELD_HINTS.every((hint) => propertyNames.includes(hint))
  ) {
    return 'database'
  }

  if (type.authenticate || propertyNames.some((name) => SECRET_FIELD_HINTS.some((hint) => name.includes(hint)))) {
    return 'token'
  }

  return 'custom'
}

export function credentialHasOAuthToken(credential: CredentialSummary): boolean {
  return credential?.data?.oauthTokenData !== undefined
}

export function extractExternalSecretReference(
  value: unknown,
): ExternalSecretReference | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null

  const maybeEnvelope = value as Record<string, unknown>
  const secretRef = maybeEnvelope.__secretRef
  if (!secretRef || typeof secretRef !== 'object' || Array.isArray(secretRef)) return null

  const ref = secretRef as Record<string, unknown>
  return {
    providerId: String(ref.providerId || ''),
    path: String(ref.path || ''),
    key: String(ref.key || ''),
  }
}

export function buildExternalSecretReferenceEnvelope(
  value: Partial<ExternalSecretReference> = {},
): ExternalSecretReferenceEnvelope {
  return {
    __secretRef: {
      providerId: String(value.providerId || ''),
      path: String(value.path || ''),
      key: String(value.key || ''),
    },
  }
}

export function credentialUsesExternalSecrets(
  credential: CredentialSummary,
): boolean {
  return Object.values(credential.data || {}).some((value) => !!extractExternalSecretReference(value))
}

export function formatRelativeTime(
  iso?: string | null,
  emptyLabel = 'Never',
): string {
  if (!iso) return emptyLabel

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return emptyLabel

  const diffSeconds = Math.floor((date.getTime() - Date.now()) / 1000)
  const absSeconds = Math.abs(diffSeconds)
  const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })

  if (absSeconds < 60) return rtf.format(diffSeconds, 'second')

  const diffMinutes = Math.floor(diffSeconds / 60)
  if (Math.abs(diffMinutes) < 60) return rtf.format(diffMinutes, 'minute')

  const diffHours = Math.floor(diffMinutes / 60)
  if (Math.abs(diffHours) < 24) return rtf.format(diffHours, 'hour')

  const diffDays = Math.floor(diffHours / 24)
  if (Math.abs(diffDays) < 30) return rtf.format(diffDays, 'day')

  return date.toLocaleDateString()
}

export function formatDateTime(iso?: string | null, emptyLabel = 'Never'): string {
  if (!iso) return emptyLabel

  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return emptyLabel
  return date.toLocaleString()
}

export function credentialStatusPresentation(
  credential: CredentialSummary,
  type?: CredentialTypeContract | null,
): {
  label: string
  detail: string
  badgeClass: string
} {
  const rotatedAt = credential.rotatedAt ? new Date(credential.rotatedAt) : null
  const lastTestedAt = credential.lastTestedAt ? new Date(credential.lastTestedAt) : null
  const rotationNeedsRetest =
    rotatedAt &&
    !Number.isNaN(rotatedAt.getTime()) &&
    (!lastTestedAt || Number.isNaN(lastTestedAt.getTime()) || rotatedAt > lastTestedAt)

  if (rotationNeedsRetest) {
    return {
      label: 'Retest Required',
      detail: 'Credential changed after the last successful validation.',
      badgeClass: 'bg-amber-100 text-amber-700',
    }
  }

  if (credential.lastTestStatus === 'valid') {
    return {
      label: credentialHasOAuthToken(credential) && credentialSupportsOAuthConnect(type)
        ? 'Connected'
        : 'Validated',
      detail: credential.lastTestMessage || 'Credential validated successfully.',
      badgeClass: 'bg-emerald-100 text-emerald-700',
    }
  }

  if (credential.lastTestStatus === 'invalid') {
    return {
      label: 'Needs Fix',
      detail: credential.lastTestMessage || 'Credential validation failed.',
      badgeClass: 'bg-red-100 text-red-700',
    }
  }

  if (credential.lastTestStatus === 'error') {
    return {
      label: 'Test Error',
      detail: credential.lastTestMessage || 'Credential validation hit a runtime error.',
      badgeClass: 'bg-orange-100 text-orange-700',
    }
  }

  if (credentialHasOAuthToken(credential) && credentialSupportsOAuthConnect(type)) {
    return {
      label: 'Connected',
      detail: 'OAuth token data is present and ready to bind.',
      badgeClass: 'bg-blue-100 text-blue-700',
    }
  }

  return {
    label: 'Saved',
    detail: 'Credential is stored but has not been validated yet.',
    badgeClass: 'bg-slate-100 text-slate-700',
  }
}
