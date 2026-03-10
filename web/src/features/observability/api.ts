import http from '../../api'
import type { ObservabilityOverview } from '../../types/contracts'

export function getObservabilityOverview(hours = 72) {
  return http.get<ObservabilityOverview>('/observability/overview', {
    params: { hours },
  })
}
