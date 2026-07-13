import { expect, test, type APIRequestContext, type Page } from '@playwright/test';

type User = {
  id: string;
  username: string;
  display_name: string;
  role: 'admin' | 'user';
  created_at: string;
  updated_at: string;
};

type AuthResponse = {
  token: string;
  user: User;
  participant?: { id: string } | null;
};

type Gathering = {
  id: string;
  invite_code: string;
  title: string;
};

type MenuItem = {
  id: string;
  revision: number;
  name: string;
};

async function loginAdmin(request: APIRequestContext) {
  const response = await request.post('/api/auth/login', {
    data: { username: 'suite-admin', password: 'Admin_1234' },
  });
  expect(response.ok()).toBeTruthy();
  return (await response.json()) as AuthResponse;
}

async function createGathering(request: APIRequestContext, adminToken: string) {
  const response = await request.post('/api/gatherings', {
    headers: { Authorization: `Bearer ${adminToken}` },
    data: {
      title: `E2E Conflict ${Date.now()}`,
      description: 'E2E gathering for browser conflict tests',
      host_name: 'E2E Host',
      expires_at: '2099-07-14T12:00:00Z',
    },
  });
  expect(response.ok()).toBeTruthy();
  const body = await response.json();
  return body.gathering as Gathering;
}

async function registerUser(
  request: APIRequestContext,
  displayName: string,
  gatheringId: string,
) {
  const response = await request.post('/api/auth/register', {
    data: { display_name: displayName, gathering_id: gatheringId },
  });
  expect(response.ok()).toBeTruthy();
  return (await response.json()) as AuthResponse;
}

async function createMenuItem(
  request: APIRequestContext,
  userToken: string,
  gatheringId: string,
  participantId: string,
) {
  const response = await request.post(`/api/gatherings/${gatheringId}/menu-items`, {
    headers: { Authorization: `Bearer ${userToken}` },
    data: {
      created_by: participantId,
      name: 'Conflict noodles',
      category: 'Main',
      quantity: 1,
      unit: 'plates',
      owner_name: 'E2E Chef',
      status: 'planned',
    },
  });
  expect(response.ok()).toBeTruthy();
  const body = await response.json();
  return body.menu_item as MenuItem;
}

async function setAuth(page: Page, auth: AuthResponse) {
  await page.addInitScript(({ token, user }) => {
    window.localStorage.setItem('letsorder:auth_token', token);
    window.localStorage.setItem('letsorder:auth_user', JSON.stringify(user));
  }, auth);
}

test('protected routes hide content when the user is not logged in', async ({ page }) => {
  await page.goto('/join');

  await expect(page.getByText('Login required')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Log in to LetsOrder' })).toBeVisible();
  await expect(page.getByText('Join an active gathering')).not.toBeVisible();
});

test('dish editor shows conflict dialog for stale edits', async ({ page, request }) => {
  const admin = await loginAdmin(request);
  const gathering = await createGathering(request, admin.token);
  const user = await registerUser(request, 'E2E Chef', gathering.id);
  const participantId = user.participant?.id;
  expect(participantId).toBeTruthy();
  const menuItem = await createMenuItem(request, user.token, gathering.id, participantId!);

  await setAuth(page, user);
  await page.goto(`/menu/${gathering.invite_code}`);
  await expect(page.locator('h1.menu-title')).toHaveText(gathering.title);
  await expect(page.getByRole('heading', { name: menuItem.name })).toBeVisible();

  await page.getByRole('button', { name: 'Edit item' }).click();
  await expect(page.getByText('Dish editor')).toBeVisible();

  const concurrentUpdate = await request.patch(`/api/menu-items/${menuItem.id}`, {
    headers: { Authorization: `Bearer ${user.token}` },
    data: {
      updated_by: participantId,
      quantity: 2,
      expected_revision: menuItem.revision,
    },
  });
  expect(concurrentUpdate.ok()).toBeTruthy();

  await page.getByLabel('Qty').fill('3');
  await page.getByRole('button', { name: 'Save changes' }).click();

  await expect(page.getByText('Conflict detected')).toBeVisible();
  await expect(page.getByText('This dish changed while you were editing')).toBeVisible();
  await expect(page.getByText('Latest', { exact: true })).toBeVisible();
  await expect(page.getByText('Your Change', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Use latest' }).click();
  await expect(page.getByText('Loaded the latest dish. Review it, then save again if needed.')).toBeVisible();
});
