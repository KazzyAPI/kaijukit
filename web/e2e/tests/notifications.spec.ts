import { expect, test } from '@playwright/test';
import { login, para, seedPage } from './helpers';

// The mock seeds notifications through __notify; the real stack gets them from a second user (see social.spec.ts).
test.skip(!!process.env.E2E_BASE_URL, 'mock-only: seeds notifications through the mock');

test('bell shows an unread badge, opens the page, marks read, and the page lists them', async ({ page }) => {
  const a = await seedPage(page, `Notify A ${Date.now()}`, [para('a')]);
  const b = await seedPage(page, `Notify B ${Date.now()}`, [para('b')]);
  await login(page, '/wiki/home');
  await expect(page.getByRole('button', { name: 'Notifications', exact: true })).toBeVisible();
  for (const p of [a, b]) await page.request.post('/wiki/api/v2/__notify', { data: { pageId: p.id } });
  await page.reload();
  const bell = page.getByRole('button', { name: 'Notifications, 2 unread' });
  await expect(bell).toBeVisible();
  await bell.click();
  const menu = page.getByRole('menu', { name: 'Notifications' });
  await expect(menu.getByRole('menuitem', { name: /Bob Reader updated Notify A/ })).toBeVisible();
  await menu.getByRole('menuitem', { name: /Bob Reader updated Notify A/ }).click();
  await expect(page).toHaveURL(new RegExp(`/pages/${a.id}`));
  await expect(page.getByRole('button', { name: 'Notifications, 1 unread' })).toBeVisible();

  await page.getByRole('button', { name: 'Notifications, 1 unread' }).click();
  await page.getByRole('menuitem', { name: 'View all notifications' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Notifications' })).toBeVisible();
  await expect(page.getByRole('list', { name: 'Notifications' }).getByRole('listitem')).toHaveCount(2);
  await page.getByRole('button', { name: 'Mark all as read' }).first().click();
  await expect(page.getByRole('button', { name: 'Notifications', exact: true })).toBeVisible();
  await expect(page.getByText('Unread')).toHaveCount(0);
});
