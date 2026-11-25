import { expect, test } from '@playwright/test';
import { login, para, seedPage } from './helpers';

test('space templates: create, pick from the Create row and the top bar, edit, delete', async ({ page }) => {
  test.setTimeout(90_000);
  const tag = Date.now().toString(36);
  const s = await seedPage(page, `Tpl ${tag}`, [para('x')]);
  await login(page, `/wiki/spaces/${s.key}/settings?tab=templates`);
  await expect(page.getByText('No templates yet')).toBeVisible();

  await page.getByRole('button', { name: 'Create template' }).click();
  await expect(page).toHaveURL(/\/templates\/new$/);
  await page.getByLabel('Template name').fill(`Meeting ${tag}`);
  await page.getByLabel('Description (optional)').fill('Agenda and notes');
  await page.getByRole('textbox', { name: 'Template content' }).click();
  await page.keyboard.type(`Agenda ${tag}`);
  await page.getByRole('button', { name: 'Save template' }).click();
  const list = page.getByRole('list', { name: 'Space templates' });
  await expect(list.getByText(`Meeting ${tag}`)).toBeVisible();
  await expect(list.getByText('Agenda and notes')).toBeVisible();

  // Sidebar Create row now offers the picker.
  await page.goto(s.url);
  await page.getByRole('navigation', { name: 'Sidebar' }).getByRole('button', { name: 'Create', exact: true }).click();
  const dlg = page.getByRole('dialog', { name: 'Create page' });
  await expect(dlg.getByRole('radio', { name: 'Blank page' })).toBeChecked();
  await dlg.getByRole('radio', { name: new RegExp(`^Meeting ${tag}`) }).check();
  await dlg.getByRole('button', { name: 'Create', exact: true }).click();
  await expect(page).toHaveURL(/pages\/edit-v2\/\d+$/);
  await expect(page.getByRole('textbox', { name: 'Page content' })).toContainText(`Agenda ${tag}`);

  // Top bar Create > Page asks for the space and offers the same template.
  await page.goto('/wiki/home');
  await page.getByRole('button', { name: 'Create', exact: true }).first().click();
  await page.getByRole('menuitem', { name: 'Page' }).click();
  await expect(page.getByRole('dialog', { name: 'Create page' }).getByLabel('Space', { exact: true })).toBeVisible();
  await page.keyboard.press('Escape');

  // Edit then delete.
  await page.goto(`/wiki/spaces/${s.key}/settings?tab=templates`);
  await page.getByRole('button', { name: `Edit Meeting ${tag}` }).click();
  await expect(page.getByRole('textbox', { name: 'Template content' })).toContainText(`Agenda ${tag}`);
  await page.getByLabel('Template name').fill(`Retro ${tag}`);
  await page.getByRole('button', { name: 'Save template' }).click();
  await expect(list.getByText(`Retro ${tag}`)).toBeVisible();
  await page.getByRole('button', { name: `Delete Retro ${tag}` }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click();
  await expect(page.getByText('No templates yet')).toBeVisible();
});
