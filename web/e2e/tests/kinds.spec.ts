import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { api, login, para, seedPage } from './helpers';

test.skip(!process.env.E2E_BASE_URL, 'these specs need the real server (E2E_BASE_URL)');

const viewports = { wide: { width: 1280, height: 800 }, narrow: { width: 390, height: 844 } };
const violations: string[] = [];

for (const scheme of ['light', 'dark'] as const) {
  for (const [vpName, vp] of Object.entries(viewports)) {
    test(`blog posts and folders ${scheme} ${vpName}`, async ({ page }) => {
      test.setTimeout(90_000);
      await page.setViewportSize(vp);
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' });
      const shot = async (name: string) => {
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(200);
        await page.screenshot({ path: `screenshots/kn-${scheme}-${vpName}-${name}.png`, fullPage: true });
        const r = await new AxeBuilder({ page: page as never }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
        for (const v of r.violations) violations.push(`${scheme}-${vpName}-${name}: ${v.id} (${v.impact}) ${v.nodes.slice(0, 2).map((n) => n.target.join(' ')).join(' | ')}`);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        if (overflow > 0) violations.push(`${scheme}-${vpName}-${name}: horizontal page overflow ${overflow}px`);
      };
      const tag = `${scheme}${vpName}${Date.now().toString(36)}`;
      const s = await seedPage(page, `Home ${tag}`, [para('x')]);
      const home = (await api<{ homepageId: string }>(page, 'get', `/spaces/${s.spaceId}`)).homepageId;
      const folder = await api<{ id: string }>(page, 'post', '/folders', { spaceId: s.spaceId, title: `Folder ${tag}`, parentId: home });
      await api(page, 'post', '/pages', { spaceId: s.spaceId, status: 'current', title: `Inside ${tag}`, parentId: folder.id });
      await login(page, s.url);
      if (vpName === 'narrow') await page.getByRole('button', { name: /sidebar/i }).first().click();

      // The folder is a tree row; selecting it opens it instead of navigating, and its page shows up.
      const tree = page.getByRole('tree', { name: 'Pages' });
      await tree.getByRole('treeitem', { name: `Folder ${tag}` }).locator('.kk-tree__row').click();
      await expect(tree.getByRole('treeitem', { name: `Inside ${tag}` })).toBeVisible();
      await expect(page).toHaveURL(new RegExp(`/pages/${s.id}`));
      await shot('folder-tree');

      // A new folder through the menu.
      await page.getByRole('button', { name: 'Content actions' }).click();
      await page.getByRole('menuitem', { name: 'New folder' }).click();
      await page.getByRole('dialog').getByLabel('Title').fill(`Made ${tag}`);
      await page.getByRole('dialog').getByRole('button', { name: 'Create' }).click();
      await expect(tree.getByRole('treeitem', { name: `Made ${tag}` })).toBeVisible();

      // A blog post: created from the nav, listed there, opened on its own route.
      await page.getByRole('button', { name: 'Create blog post' }).click();
      await page.getByRole('dialog').getByLabel('Title').fill(`Post ${tag}`);
      await page.getByRole('dialog').getByLabel('Text').fill('First paragraph.\n\nSecond paragraph.');
      await shot('blog-dialog');
      await page.getByRole('dialog').getByRole('button', { name: 'Create' }).click();
      await expect(page.getByRole('heading', { level: 1, name: `Post ${tag}` })).toBeVisible();
      await expect(page.getByText('Second paragraph.')).toBeVisible();
      if (vpName === 'wide') await expect(page.getByRole('link', { name: `Post ${tag}` })).toBeVisible();
      await shot('blog-view');
      expect(violations, violations.join('\n')).toEqual([]);
    });
  }
}
