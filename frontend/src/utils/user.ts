import type { User } from '../types/auth';

const AUTH_TOKEN_KEY = 'letsorder:auth_token';
const AUTH_USER_KEY = 'letsorder:auth_user';
export const USER_CHANGED_EVENT = 'letsorder:user-changed';

export function getAuthToken() {
  return localStorage.getItem(AUTH_TOKEN_KEY) ?? '';
}

export function setAuthSession(token: string, user: User) {
  localStorage.setItem(AUTH_TOKEN_KEY, token);
  localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user));
  notifyUserChanged(user);
}

export function getCurrentUser(): User | null {
  const rawValue = localStorage.getItem(AUTH_USER_KEY);
  if (!rawValue) {
    return null;
  }

  try {
    return JSON.parse(rawValue) as User;
  } catch {
    return null;
  }
}

export function updateStoredUser(user: User) {
  localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user));
  notifyUserChanged(user);
}

export function clearAuthSession() {
  localStorage.removeItem(AUTH_TOKEN_KEY);
  localStorage.removeItem(AUTH_USER_KEY);
  notifyUserChanged(null);
}

export function notifyUserChanged(user: User | null) {
  window.dispatchEvent(new CustomEvent(USER_CHANGED_EVENT, { detail: user }));
}

export function getCookieUser() {
  return getCurrentUser()?.display_name ?? '';
}

export function setCookieUser(user: string) {
  const currentUser = getCurrentUser();
  if (!currentUser) {
    return;
  }

  updateStoredUser({ ...currentUser, display_name: user });
}

export function syncUserFromQuery() {
  return getCookieUser();
}
