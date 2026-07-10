import { apiRequest } from './client';
import type { AuthResponse, User } from '../types/auth';

export interface MeResponse {
  user: User;
}

export interface MembersResponse {
  members: User[];
}

export interface MemberResponse {
  member: User;
}

export function login(username: string, password: string) {
  return apiRequest<AuthResponse>('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
}

export function register(displayName: string, gatheringId?: string, inviteCode?: string) {
  return apiRequest<AuthResponse>('/api/auth/register', {
    method: 'POST',
    body: JSON.stringify({
      display_name: displayName,
      gathering_id: gatheringId,
      invite_code: inviteCode,
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

export function listMembers() {
  return apiRequest<MembersResponse>('/api/auth/members');
}

export function updateMember(
  userId: string,
  payload: {
    display_name?: string;
    password?: string;
  },
) {
  return apiRequest<MemberResponse>(`/api/auth/members/${userId}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export function logout() {
  return apiRequest<void>('/api/auth/logout', {
    method: 'POST',
  });
}
