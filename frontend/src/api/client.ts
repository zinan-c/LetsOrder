const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '';

function getCookieUser() {
  const rawValue = document.cookie
    .split(';')
    .map((part) => part.trim())
    .find((part) => part.startsWith('user='))
    ?.slice('user='.length);

  return rawValue ? decodeURIComponent(rawValue) : '';
}

export async function apiRequest<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const { headers, ...requestOptions } = options;
  const currentUser = getCookieUser();
  const isFormData = options.body instanceof FormData;
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...requestOptions,
    headers: {
      ...(isFormData ? {} : { 'Content-Type': 'application/json' }),
      ...(currentUser ? { 'X-LetsOrder-User': currentUser } : {}),
      ...headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API request failed: ${response.status}`);
  }

  return response.json() as Promise<T>;
}
