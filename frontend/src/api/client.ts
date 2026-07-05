import { clearAuthSession, getAuthToken } from '../utils/user';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '';

export async function apiRequest<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const { headers, ...requestOptions } = options;
  const authToken = getAuthToken();
  const isFormData = options.body instanceof FormData;
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...requestOptions,
    headers: {
      ...(isFormData ? {} : { 'Content-Type': 'application/json' }),
      ...(authToken ? { Authorization: `Bearer ${authToken}` } : {}),
      ...headers,
    },
  });

  if (!response.ok) {
    if (response.status === 401) {
      clearAuthSession();
    }

    throw new Error(`API request failed: ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}
