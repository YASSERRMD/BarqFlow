import http from '../../api'

export interface RuntimeSettings {
  serverTime: string
  environment: string
  nodeTypesCount: number
  credentialTypesCount: number
  encryptionKeyConfigured: boolean
}

export function getRuntimeSettings() {
  return http.get<RuntimeSettings>('/settings/runtime')
}
