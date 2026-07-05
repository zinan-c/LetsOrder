const USER_COOKIE_NAME = 'user';
export const USER_CHANGED_EVENT = 'letsorder:user-changed';

export function getCookieUser() {
  const rawValue = document.cookie
    .split(';')
    .map((part) => part.trim())
    .find((part) => part.startsWith(`${USER_COOKIE_NAME}=`))
    ?.slice(USER_COOKIE_NAME.length + 1);

  return rawValue ? decodeURIComponent(rawValue) : '';
}

export function setCookieUser(user: string) {
  document.cookie = `${USER_COOKIE_NAME}=${encodeURIComponent(user)}; path=/; max-age=2592000; SameSite=Lax`;
}

export function notifyUserChanged(user: string) {
  window.dispatchEvent(new CustomEvent(USER_CHANGED_EVENT, { detail: user }));
}

export function syncUserFromQuery(search: string) {
  const queryUser = new URLSearchParams(search).get(USER_COOKIE_NAME);

  if (queryUser === null) {
    return getCookieUser();
  }

  setCookieUser(queryUser.trim());
  return queryUser.trim();
}
