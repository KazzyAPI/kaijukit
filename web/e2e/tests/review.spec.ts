import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { ADMIN, login, uniqueKey } from './helpers';

const auth = { Authorization: `Basic ${Buffer.from(`${ADMIN.email}:${ADMIN.token}`).toString('base64')}` };
const V2 = '/wiki/api/v2';
const text = (t: string, marks?: { type: string; attrs?: object }[]) => ({ type: 'text', text: t, ...(marks ? { marks } : {}) });
const p = (...c: object[]) => ({ type: 'paragraph', content: c });
const rich = {
  version: 1,
  type: 'doc',
  content: [
    { type: 'heading', attrs: { level: 2 }, content: [text('Overview')] },
    p(text('Plain text with '), text('bold', [{ type: 'strong' }]), text(', '), text('italic', [{ type: 'em' }]), text(', '), text('code', [{ type: 'code' }]), text(' and a '), text('link', [{ type: 'link', attrs: { href: 'https://example.com' } }]), text('. '.repeat(1) + 'A long unbroken token follows: ' + 'x'.repeat(90))),
    { type: 'bulletList', content: [{ type: 'listItem', content: [p(text('First'))] }, { type: 'listItem', content: [p(text('Second'))] }] },
    { type: 'orderedList', content: [{ type: 'listItem', content: [p(text('Step one'))] }, { type: 'listItem', content: [p(text('Step two'))] }] },
    { type: 'blockquote', content: [p(text('A quoted sentence.'))] },
    { type: 'codeBlock', attrs: { language: 'ts' }, content: [text('const veryLongLine = "this line is intentionally long so the block has to scroll horizontally rather than overflow";')] },
    { type: 'rule' },
    { type: 'table', content: [{ type: 'tableRow', content: ['Name', 'Role', 'Notes'].map((h) => ({ type: 'tableHeader', content: [p(text(h))] })) }, { type: 'tableRow', content: ['Ada', 'Admin', 'Owns the space'].map((h) => ({ type: 'tableCell', content: [p(text(h))] })) }] },
  ],
};

async function post(page: Page, path: string, data: object) {
  const r = await page.request.post(`${V2}${path}`, { headers: auth, data });
  expect(r.ok(), await r.text()).toBeTruthy();
  return (await r.json()) as { id: string; key?: string };
}
const body = (d: object) => ({ representation: 'atlas_doc_format', value: JSON.stringify(d) });

const viewports = { wide: { width: 1280, height: 800 }, narrow: { width: 390, height: 844 } };
const violations: string[] = [];

for (const scheme of ['light', 'dark'] as const) {
  for (const [vpName, vp] of Object.entries(viewports)) {
    test(`review ${scheme} ${vpName}`, async ({ page }) => {
      test.setTimeout(120_000);
      await page.setViewportSize(vp);
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' });
      const shot = async (name: string) => {
        await page.waitForTimeout(150);
        await page.screenshot({ path: `screenshots/${scheme}-${vpName}-${name}.png`, fullPage: false });
        const r = await new AxeBuilder({ page: page as never }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
        for (const v of r.violations) violations.push(`${scheme}-${vpName}-${name}: ${v.id} (${v.impact}) ${v.nodes.slice(0, 2).map((n) => n.target.join(' ')).join(' | ')}`);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        if (overflow > 0) violations.push(`${scheme}-${vpName}-${name}: horizontal page overflow ${overflow}px`);
      };

      await page.request.post(`${V2}/__reset`);
      await page.goto('/wiki/login');
      await shot('login');
      await page.getByLabel('Email').fill(ADMIN.email);
      await page.getByLabel('API token').fill('nope');
      await page.getByRole('button', { name: 'Log in' }).click();
      await expect(page.getByRole('alert')).toBeVisible();
      await shot('login-error');
      await login(page);
      await expect(page.getByText('Nothing here yet')).toBeVisible();
      await shot('home-empty');
      await page.goto('/wiki/spaces');
      await expect(page.getByText('No spaces yet').first()).toBeVisible();
      await shot('spaces-empty');
      await page.getByRole('button', { name: 'Create space' }).first().click();
      await page.getByLabel('Space name').fill('Engineering Handbook');
      await shot('create-space-dialog');
      await page.getByLabel('Space key').fill('bad key!');
      await shot('create-space-dialog-invalid');
      await page.keyboard.press('Escape');

      const key = uniqueKey();
      const space = await post(page, '/spaces', { name: 'Engineering Handbook', key });
      await post(page, '/spaces', { name: 'Design', key: `D${key}` });
      const home = (await (await page.request.get(`${V2}/spaces/${space.id}`, { headers: auth })).json()).homepageId as string;
      const guide = await post(page, '/pages', { spaceId: space.id, parentId: home, status: 'current', title: 'Onboarding guide with a deliberately very long title that must wrap cleanly', body: body(rich) });
      const child = await post(page, '/pages', { spaceId: space.id, parentId: guide.id, status: 'current', title: 'Day one checklist', body: body({ version: 1, type: 'doc', content: [p(text('Short page.'))] }) });
      await post(page, '/pages', { spaceId: space.id, parentId: guide.id, status: 'current', title: 'Tooling', body: body({ version: 1, type: 'doc', content: [p(text('Tools.'))] }) });
      await post(page, '/pages', { spaceId: space.id, parentId: home, status: 'current', title: 'Runbooks', body: body({ version: 1, type: 'doc', content: [p(text('Ops.'))] }) });

      await page.goto('/wiki/home');
      await expect(page.getByRole('link', { name: 'Runbooks' })).toBeVisible();
      await shot('home');
      await page.goto('/wiki/spaces');
      await expect(page.getByRole('link', { name: 'Design' })).toBeVisible();
      await shot('spaces');
      await page.getByLabel('Filter by name').fill('zzz');
      await shot('spaces-no-match');
      await page.getByRole('tab', { name: 'Archived' }).click();
      await expect(page.getByText('No archived spaces')).toBeVisible();
      await shot('spaces-archived-empty');

      await page.goto(`/wiki/spaces/${key}/overview`);
      await expect(page.getByRole('heading', { level: 1, name: 'Engineering Handbook' })).toBeVisible();
      await shot('overview');

      await page.goto(`/wiki/spaces/${key}/pages/${child.id}/day-one-checklist`);
      if (vpName === 'narrow') await shot('page-sidebar-closed');
      if (vpName === 'narrow') await page.getByRole('button', { name: 'Show sidebar' }).click();
      await expect(page.getByRole('treeitem', { name: 'Tooling' })).toBeVisible();
      await shot('page-with-tree');
      if (vpName === 'narrow') await page.getByRole('button', { name: 'Collapse sidebar' }).click();
      await page.goto(`/wiki/spaces/${key}/pages/${guide.id}`);
      await expect(page.getByText('Plain text with')).toBeVisible();
      await shot('page-view');
      await page.getByRole('button', { name: 'More actions', exact: true }).click();
      await shot('page-menu');
      await page.getByRole('menuitem', { name: 'Move to trash' }).click();
      await shot('delete-dialog');
      await page.keyboard.press('Escape');

      await page.route(`**${V2}/pages/${guide.id}?*`, async (r) => {
        await new Promise((res) => setTimeout(res, 700));
        await r.continue();
      });
      await page.goto(`/wiki/spaces/${key}/pages/${guide.id}`);
      await page.waitForSelector('.kk-skeleton');
      await shot('page-loading');
      await page.unroute(`**${V2}/pages/${guide.id}?*`);
      await page.route(`**${V2}/pages/${guide.id}?*`, (r) => r.fulfill({ status: 500, contentType: 'application/json', body: '{"errors":[{"title":"Boom","detail":"The server had a problem."}]}' }));
      await page.goto(`/wiki/spaces/${key}/pages/${guide.id}`);
      await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible({ timeout: 15_000 });
      await shot('page-error');
      await page.unroute(`**${V2}/pages/${guide.id}?*`);
      await page.goto(`/wiki/spaces/${key}/pages/999999`);
      await expect(page.getByText("We can't find that page")).toBeVisible();
      await shot('page-not-found');

      await page.goto(`/wiki/spaces/${key}/overview`);
      if (vpName === 'wide') {
        await page.getByRole('button', { name: 'Collapse sidebar' }).click();
        await shot('sidebar-collapsed');
        await page.getByRole('button', { name: 'Show sidebar' }).click();
      }
      await page.getByRole('button', { name: 'Account' }).click();
      await shot('account-menu');
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: /^Notifications/ }).click();
      await shot('notifications-menu');
      await page.keyboard.press('Escape');
      {
        await page.getByRole('button', { name: 'Help' }).click();
        await page.getByRole('menuitem', { name: 'Keyboard shortcuts' }).click();
        await shot('shortcuts-dialog');
        await page.keyboard.press('Escape');
        await page.getByRole('button', { name: 'Switch app' }).click();
        await shot('app-switcher');
        await page.keyboard.press('Escape');
      }
      await page.getByRole('button', { name: 'Search', exact: true }).click();
      await shot('quick-search-empty');
      await page.getByLabel('Search pages', { exact: true }).fill('Runbooks');
      await expect(page.getByRole('dialog').getByRole('link', { name: 'Runbooks', exact: true })).toBeVisible();
      await shot('quick-search-results');
      await page.getByLabel('Search pages', { exact: true }).fill('zzzz');
      await expect(page.getByText('No pages match "zzzz".')).toBeVisible();
      await shot('quick-search-none');
      await page.keyboard.press('Escape');
      if (vpName === 'narrow') await page.getByRole('button', { name: 'Show sidebar' }).click();
      await page.getByRole('button', { name: 'Recent' }).click();
      await expect(page.getByRole('menuitem', { name: 'Runbooks' })).toBeVisible();
      await shot('flyout-recent');
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: 'Starred' }).click();
      await shot('flyout-starred');
      await page.keyboard.press('Escape');
      await page.getByLabel('Search by title').fill('Tooling');
      await expect(page.getByRole('link', { name: 'Tooling' })).toBeVisible();
      await shot('tree-search-hit');
      await page.getByLabel('Search by title').fill('nope');
      await expect(page.getByText('No page is titled "nope".')).toBeVisible();
      await shot('tree-search-none');
      await page.getByLabel('Search by title').fill('');
      await page.getByRole('button', { name: 'More actions for Runbooks' }).click();
      await shot('tree-row-menu');
      await page.keyboard.press('Escape');
      if (vpName === 'narrow') await page.getByRole('button', { name: 'Collapse sidebar' }).click();
      await page.goto(`/wiki/spaces/${key}/pages/${guide.id}`);
      await page.getByRole('button', { name: 'Share', exact: true }).click();
      await shot('share-dialog');
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: 'Collapse page header' }).click();
      await shot('page-header-collapsed');
      await page.getByRole('button', { name: 'Expand page header' }).click();
      await page.getByRole('button', { name: 'Create' }).first().click();
      await shot('create-menu');
      await page.keyboard.press('Escape');

      if (vpName === 'narrow') await page.getByRole('button', { name: 'Show sidebar' }).click();
      await page.getByRole('button', { name: 'Create page', exact: true }).click();
      await expect(page).toHaveURL(/edit-v2/);
      await shot('editor-new');
      await page.getByLabel('Title', { exact: true }).fill('Release notes');
      await page.getByLabel('Title', { exact: true }).press('Enter');
      await page.keyboard.type('Intro paragraph with some text\n/');
      await expect(page.getByRole('listbox', { name: 'Insert' })).toBeVisible();
      await shot('editor-slash-menu');
      await page.keyboard.type('tab');
      await page.keyboard.press('Enter');
      await shot('editor-table');
      await page.getByRole('button', { name: 'Text style' }).click();
      await shot('editor-text-style-menu');
      await page.getByRole('menuitem', { name: 'Heading 2' }).click();
      await page.getByRole('button', { name: 'Insert', exact: true }).click();
      await shot('editor-insert-menu');
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: 'Image' }).hover();
      await shot('editor-image-tooltip');
      await page.getByRole('button', { name: 'Link', exact: true }).click();
      await page.getByLabel('Address').fill('javascript:alert(1)');
      await page.getByRole('dialog').getByRole('button', { name: 'Apply' }).click();
      await shot('editor-link-dialog-error');
      await page.keyboard.press('Escape');
      await expect(page.getByRole('status')).toContainText('Saved', { timeout: 6000 });
      await shot('editor-saved');
      await page.getByRole('button', { name: 'Publish', exact: true }).click();
      await shot('publish-dialog');
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: 'Page options' }).click();
      await page.getByRole('menuitem', { name: 'Discard draft' }).click();
      await shot('discard-dialog');
      await page.keyboard.press('Escape');

      await page.goto(`/wiki/spaces/${key}/pages/edit-v2/${guide.id}`);
      await expect(page.getByRole('textbox', { name: 'Page content' })).toBeVisible();
      await shot('editor-rich');

      await page.keyboard.press('Escape');
      await page.getByLabel('Title', { exact: true }).fill('Runbooks');
      await page.getByRole('button', { name: 'Publish', exact: true }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Publish' }).click();
      await expect(page.getByRole('alert').filter({ hasText: 'already exists' })).toBeVisible();
      await shot('publish-duplicate-error');
    });
  }
}

test.afterAll(() => {
  console.log(violations.length ? `REVIEW FINDINGS (${violations.length}):\n${[...new Set(violations)].join('\n')}` : 'REVIEW FINDINGS: none');
});
