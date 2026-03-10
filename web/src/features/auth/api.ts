import http from '../../api'
import type { AuthResponse, UserProfile } from '../../types/contracts'

interface AuthCredentials {
  email: string
  password: string
  firstName?: string
  lastName?: string
}

export function login(credentials: AuthCredentials) {
  return http.post<AuthResponse>('/login', credentials)
}

export function register(credentials: AuthCredentials) {
  return http.post<AuthResponse>('/users', credentials)
}

export function fetchProfile() {
  return http.get<UserProfile>('/users/me')
}

export type { AuthCredentials }
