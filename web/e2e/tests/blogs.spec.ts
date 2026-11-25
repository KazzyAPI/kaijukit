import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { adfBody, api, auth, doc, login, para, seedPage, selectText } from './helpers';

test.skip(!process.env.E2E_BASE_URL, 'these specs need the real server (E2E_BASE_URL)');

const V2 = '/wiki/api/v2';
const viewports = { wide: { width: 1280, height: 800 }, narrow: { width: 390, height: 844 } };
const violations: string[] = [];

async function seedBlog(page: import('@playwright/test').Page, tag: string, text: string) {
  const s = await seedPage(page, `Home ${tag}`, [para('x')]);
  const b = await api<{ id: string }>(page, 'post', '/blogposts', { spaceId: s.spaceId, status: 'current', title: `Blog ${tag}`, body: adfBody(doc(para(text))) });
  return { ...s, blog: b.id, url: `/wiki/spaces/${s.key}/blog/${b.id}` };
}

test('blog post: edit over a published post, publish, comment, label, like, history, restore, trash', async ({ page }) => {
  test.setTimeout(120_000);
  const tag = `b${Date.now().toString(36)}`;
  const s = await seedBlog(page, tag, 'Original text.');
  await login(page, s.url);
  await expect(page.getByRole('heading', { level: 1, name: `Blog ${tag}` })).toBeVisible();

  // like, label, footer comment
  await page.getByRole('button', { name: /^Like/ }).click();
  await expect(page.getByRole('button', { name: /^Liked/ })).toBeVisible();
  await page.getByRole('button', { name: 'Add label' }).click();
  await page.getByLabel('New label').fill('news');
  await page.getByLabel('New label').press('Enter');
  await expect(page.getByRole('link', { name: 'news' })).toBeVisible();
  await page.getByRole('button', { name: 'Write a comment...' }).click();
  await page.getByLabel('Add a comment').fill('Nice post');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByRole('list', { name: 'Comments' }).getByText('Nice post')).toBeVisible();

  // inline comment on selected text
  await selectText(page, 'Original');
  await page.getByRole('button', { name: 'Comment', exact: true }).click();
  await page.getByLabel('New inline comment').fill('Inline remark');
  await page.getByRole('button', { name: 'Comment', exact: true }).last().click();
  await expect(page.getByRole('list', { name: 'Open inline comments' }).getByText('Inline remark')).toBeVisible();
  await page.getByRole('button', { name: /Inline comments, 1 open/ }).click();

  // edit: the draft autosaves over the published post, the reader still sees the old text
  await page.getByRole('button', { name: 'Edit', exact: true }).first().click();
  await page.getByRole('textbox', { name: 'Page content' }).click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' More words.');
  await expect(page.getByText('Saved', { exact: true })).toBeVisible();
  const live = await api<{ body: { atlas_doc_format: { value: string } } }>(page, 'get', `/blogposts/${s.blog}?body-format=atlas_doc_format`);
  expect(live.body.atlas_doc_format.value).not.toContain('More words');
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await page.getByRole('dialog').getByLabel('Version comment (optional)').fill('added words');
  await page.getByRole('dialog').getByRole('button', { name: 'Publish' }).click();
  await expect(page.getByText('Original text. More words.')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Version 2' })).toBeVisible();

  // history and restore
  await page.getByRole('link', { name: 'Version 2' }).click();
  await expect(page.getByRole('heading', { name: 'Blog post history' })).toBeVisible();
  await page.getByRole('button', { name: /Version 1/ }).click();
  await page.getByRole('button', { name: 'Restore this version' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Restore' }).click();
  await expect(page.getByRole('heading', { level: 1, name: `Blog ${tag}` })).toBeVisible();
  await expect(page.getByText('Original text.', { exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Version 3' })).toBeVisible();

  // trash
  await page.getByRole('button', { name: 'More actions', exact: true }).click();
  await page.getByRole('menuitem', { name: 'Move to trash' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Move to trash' }).click();
  await expect(page).toHaveURL(/\/overview$/);
  const gone = await page.request.get(`${V2}/blogposts/${s.blog}`, { headers: auth });
  expect(gone.status()).toBe(404);
});

test('blog post: unpublished draft resumes, discard removes it', async ({ page }) => {
  const tag = `d${Date.now().toString(36)}`;
  const s = await seedPage(page, `Home ${tag}`, [para('x')]);
  await login(page, s.url);
  await page.goto(`/wiki/spaces/${s.key}/blog/create`);
  await page.getByLabel('Title', { exact: true }).fill(`Draft ${tag}`);
  await page.getByLabel('Title', { exact: true }).press('Enter');
  await page.keyboard.type('Half written');
  await expect(page.getByText('Saved', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Page options' }).click();
  await page.getByRole('menuitem', { name: 'Discard draft' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Discard' }).click();
  await expect(page).toHaveURL(/\/overview$/);
});

test('folders: menu offers trash but not move, trash removes the folder', async ({ page }) => {
  test.setTimeout(90_000);
  const tag = `f${Date.now().toString(36)}`;
  const s = await seedPage(page, `Home ${tag}`, [para('x')]);
  const home = (await api<{ homepageId: string }>(page, 'get', `/spaces/${s.spaceId}`)).homepageId;
  await api(page, 'post', '/folders', { spaceId: s.spaceId, title: `Alpha ${tag}`, parentId: home });
  await api(page, 'post', '/folders', { spaceId: s.spaceId, title: `Beta ${tag}`, parentId: home });
  await login(page, s.url);
  const tree = page.getByRole('tree', { name: 'Pages' });
  await expect(tree.getByRole('treeitem', { name: `Beta ${tag}` })).toBeVisible();

  // The server moves pages only, so a folder's menu offers no Move.
  await page.getByRole('button', { name: `More actions for Beta ${tag}` }).click({ force: true });
  await expect(page.getByRole('menuitem', { name: 'Add page' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Move', exact: true })).toHaveCount(0);
  await page.keyboard.press('Escape');

  await page.getByRole('button', { name: `More actions for Alpha ${tag}` }).click({ force: true });
  await page.getByRole('menuitem', { name: 'Move to trash' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Move to trash' }).click();
  await expect(tree.getByRole('treeitem', { name: `Alpha ${tag}` })).toHaveCount(0);
});

for (const scheme of ['light', 'dark'] as const) {
  for (const [vpName, vp] of Object.entries(viewports)) {
    test(`blog features ${scheme} ${vpName}`, async ({ page }) => {
      test.setTimeout(90_000);
      await page.setViewportSize(vp);
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' });
      const shot = async (name: string) => {
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(200);
        await page.screenshot({ path: `screenshots/bg-${scheme}-${vpName}-${name}.png`, fullPage: true });
        const r = await new AxeBuilder({ page: page as never }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
        for (const v of r.violations) violations.push(`${scheme}-${vpName}-${name}: ${v.id} (${v.impact}) ${v.nodes.slice(0, 2).map((n) => n.target.join(' ')).join(' | ')}`);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        if (overflow > 0) violations.push(`${scheme}-${vpName}-${name}: horizontal page overflow ${overflow}px`);
      };
      const tag = `${scheme}${vpName}${Date.now().toString(36)}`;
      const s = await seedBlog(page, tag, 'Some blog text for review.');
      await api(page, 'post', '/footer-comments', { blogPostId: s.blog, body: adfBody(doc(para('A comment')))});
      await login(page, s.url);
      await page.getByRole('button', { name: /^Like/ }).click();
      await expect(page.getByRole('button', { name: /^Liked/ })).toBeVisible();
      await shot('view');
      await page.goto(`/wiki/spaces/${s.key}/overview`);
      await expect(page.getByRole('list', { name: 'Recent activity' }).getByRole('link', { name: `Blog ${tag}` })).toBeVisible();
      await shot('overview');
      await page.goto(`/wiki/spaces/${s.key}/blog/edit/${s.blog}`);
      await expect(page.getByRole('textbox', { name: 'Page content' })).toBeVisible();
      await shot('editor');
      await page.goto(`/wiki/spaces/${s.key}/blog/history/${s.blog}`);
      await expect(page.getByRole('heading', { name: 'Blog post history' })).toBeVisible();
      await shot('history');
      expect(violations, violations.join('\n')).toEqual([]);
    });
  }
}
