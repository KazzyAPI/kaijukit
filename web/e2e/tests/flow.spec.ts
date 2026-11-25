import { expect, test } from '@playwright/test';
import { ADMIN, createSpace, login, publishNewPage } from './helpers';

test('login guards routes and rejects bad credentials', async ({ page }) => {
  await page.goto('/wiki/spaces');
  await expect(page).toHaveURL(/\/wiki\/login\?next=%2Fwiki%2Fspaces/);
  await page.getByLabel('Email').fill(ADMIN.email);
  await page.getByLabel('API token').fill('wrong');
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page.getByRole('alert')).toHaveText('Email or API token is incorrect.');
  await page.getByLabel('API token').fill(ADMIN.token);
  await page.getByRole('button', { name: 'Log in' }).click();
  await expect(page).toHaveURL(/\/wiki\/spaces$/);
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Spaces', level: 1 })).toBeVisible();
});

test('create space, create page, edit, publish, view', async ({ page }) => {
  await login(page);
  const name = `Docs ${Date.now()}`;
  const key = await createSpace(page, name);
  await expect(page.getByRole('heading', { level: 1, name })).toBeVisible();

  await page.getByRole('button', { name: 'Create page', exact: true }).click();
  await expect(page).toHaveURL(/pages\/edit-v2\/\d+$/);
  await publishNewPage(page, 'Getting started', ['# Welcome', 'Some **bold** text', '- one', 'two']);
  await expect(page).toHaveURL(new RegExp(`/wiki/spaces/${key}/pages/\\d+/getting-started$`));
  await expect(page.getByRole('heading', { level: 1, name: 'Getting started' })).toBeVisible();
  await expect(page.getByRole('heading', { level: 1, name: 'Welcome' })).toBeVisible();
  await expect(page.locator('.kk-doc strong')).toHaveText('bold');
  await expect(page.locator('.kk-doc li')).toHaveCount(2);
  await expect(page.getByText(/Version \d+/)).toBeVisible();
  const first = Number((await page.getByText(/Version \d+/).textContent())!.match(/Version (\d+)/)![1]);
  await expect(page.getByRole('treeitem', { name: 'Getting started' })).toBeVisible();

  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await expect(page).toHaveURL(/pages\/edit-v2\/\d+$/);
  const editor = page.getByRole('textbox', { name: 'Page content' });
  await editor.click();
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+ArrowDown' : 'Control+End');
  await page.keyboard.type('Added later');
  await expect(page.getByRole('status')).toContainText('Unpublished changes');
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await page.getByLabel('Version comment (optional)').fill('second pass');
  await page.getByRole('dialog').getByRole('button', { name: 'Publish' }).click();
  await expect(page.getByText(/Version \d+/)).not.toHaveText(`Version ${first}`);
  await expect(page.getByText('second pass').or(page.getByText(/Version \d+/))).toBeVisible();
  await expect(page.getByText('Added later')).toBeVisible();
});

test('create child page and navigate the tree by keyboard', async ({ page }) => {
  await login(page);
  await createSpace(page, `Tree ${Date.now()}`);
  await page.getByRole('button', { name: 'Create page', exact: true }).click();
  await publishNewPage(page, 'Parent', ['hello']);
  await page.getByRole('button', { name: 'More actions', exact: true }).click();
  await page.getByRole('menuitem', { name: 'Add child page' }).click();
  await publishNewPage(page, 'Child', ['kid']);

  const tree = page.getByRole('tree', { name: 'Pages' });
  await expect(tree.getByRole('treeitem', { name: 'Child' })).toBeVisible();
  const parent = tree.getByRole('treeitem', { name: 'Parent' });
  await expect(parent).toHaveAttribute('aria-expanded', 'true');
  await parent.focus();
  await page.keyboard.press('ArrowLeft');
  await expect(parent).toHaveAttribute('aria-expanded', 'false');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  await expect(tree.getByRole('treeitem', { name: 'Child' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { level: 1, name: 'Child' })).toBeVisible();
});

test('unknown routes show not found', async ({ page }) => {
  await login(page);
  await page.goto('/wiki/spaces/NOPE/overview');
  await expect(page.getByText("We can't find that page")).toBeVisible();
  await page.goto('/wiki/nothing-here');
  await expect(page.getByText("We can't find that page")).toBeVisible();
});

test('shell: byline, quick search, flyouts, share, toolbar', async ({ page }) => {
  await login(page);
  await createSpace(page, `Shell ${Date.now()}`);
  const title = `Searchable ${Date.now()}`;
  await page.getByRole('button', { name: 'Create page', exact: true }).click();
  await page.getByLabel('Title', { exact: true }).fill(title);
  await page.getByLabel('Title', { exact: true }).press('Enter');
  await page.keyboard.type('Intro');
  await page.getByRole('button', { name: 'Text style' }).click();
  await page.getByRole('menuitem', { name: 'Heading 2' }).click();
  await expect(page.locator('.kk-doc h2')).toHaveText('Intro');
  await page.getByRole('button', { name: 'Image', exact: true }).hover();
  await expect(page.getByRole('tooltip')).toHaveText('Image');
  await page.getByRole('button', { name: 'Pin toolbar' }).click();
  await expect(page.getByRole('button', { name: 'Pin toolbar' })).toHaveAttribute('aria-pressed', 'false');
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Publish' }).click();

  await expect(page.getByText(/By \S+/)).toBeVisible();
  await expect(page.getByText('1 min read')).toBeVisible();
  await expect(page.getByRole('button', { name: 'See views' })).toHaveAttribute('aria-disabled', 'true');

  await page.getByRole('button', { name: 'Share', exact: true }).click();
  await expect(page.getByRole('dialog', { name: /Share/ }).getByLabel('Page link')).toHaveValue(/\/pages\/\d+\//);
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: 'Recent' }).click();
  await expect(page.getByRole('menuitem', { name: title })).toBeVisible();
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Starred' }).click();
  await expect(page.getByRole('menu', { name: 'Starred' })).toContainText("isn't built yet");
  await page.keyboard.press('Escape');

  await page.goto('/wiki/home');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByLabel('Search pages', { exact: true }).fill(title);
  await page.getByRole('dialog').getByRole('link', { name: title, exact: true }).click();
  await expect(page.getByRole('heading', { level: 1, name: title })).toBeVisible();

  await page.getByLabel('Search by title').fill('no-such-page-xyz');
  await expect(page.getByText('No page is titled "no-such-page-xyz".')).toBeVisible();
  await page.getByLabel('Search by title').fill(title);
  await expect(page.getByRole('link', { name: title })).toBeVisible();
});
