import { expect, type Page } from '@playwright/test';

export const ADMIN = { email: 'admin@kaijukit.local', token: 'kk-dev-token' };

export async function login(page: Page, next = '/wiki/home') {
  await page.goto(`/wiki/login?next=${encodeURIComponent(next)}`);
  await page.getByLabel('Email').fill(ADMIN.email);
  await page.getByLabel('API token').fill(ADMIN.token);
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page).toHaveURL(new RegExp(next.replace(/[?]/g, '\\?')));
}

export const uniqueKey = () => `E${Date.now().toString(36).toUpperCase()}${Math.floor(Math.random() * 1000)}`;

export async function createSpace(page: Page, name: string) {
  await page.goto('/wiki/spaces');
  await page.getByRole('button', { name: 'Create space' }).first().click();
  await page.getByLabel('Space name').fill(name);
  await page.getByLabel('Space key').fill(uniqueKey());
  await page.getByRole('dialog').getByRole('button', { name: 'Create' }).click();
  await expect(page).toHaveURL(/\/overview$/);
  return page.url().match(/spaces\/([^/]+)\//)![1]!;
}

export async function publishNewPage(page: Page, title: string, body: string[]) {
  await page.getByLabel('Title', { exact: true }).fill(title);
  await page.getByLabel('Title', { exact: true }).press('Enter');
  const editor = page.getByRole('textbox', { name: 'Page content' });
  await expect(editor).toBeFocused();
  await page.keyboard.type(body.join('\n'));
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Publish' }).click();
  await expect(page.getByRole('heading', { level: 1, name: title })).toBeVisible();
}
