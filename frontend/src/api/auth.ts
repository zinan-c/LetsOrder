import { apiRequest } from './client';
import type { AuthResponse, User } from '../types/auth';

export interface MeResponse {
  user: User;
}

export function login(username: string, password: string) {
  return apiRequest<AuthResponse>('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
}

export function register(displayName: string, gatheringId?: string) {
  return apiRequest<AuthResponse>('/api/auth/register', {
    method: 'POST',
    body: JSON.stringify({
      display_name: displayName,
      gathering_id: gatheringId,
    }),
  });
}

export function getMe() {
  return apiRequest<MeResponse>('/api/auth/me');
}

export function updateAccount(payload: {
  display_name?: string;
  password?: string;
}) {
  return apiRequest<MeResponse>('/api/auth/account', {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}
