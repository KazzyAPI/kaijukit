import AxeBuilder from '@axe-core/playwright';
import { execSync } from 'node:child_process';
import { expect, test, type Page } from '@playwright/test';
import { api, auth, BOB, login, para, seedPage, uniqueKey } from './helpers';

test.skip(!process.env.E2E_BASE_URL, 'these specs need the real server (E2E_BASE_URL)');

const V1 = '/wiki/rest/api';
const viewports = { wide: { width: 1280, height: 800 }, narrow: { width: 390, height: 844 } };
const violations: string[] = [];

// There is no user-creation endpoint, so the second (non-admin) account is inserted straight into the stack's database.
test.beforeAll(() => {
  const project = process.env.COMPOSE_PROJECT_NAME;
  test.skip(!project, 'COMPOSE_PROJECT_NAME is needed to create the second account');
  const sql = `INSERT INTO users (email, display_name) VALUES ('${BOB.email}', 'Bob Reader') ON CONFLICT DO NOTHING; INSERT INTO api_tokens (user_id, token_hash, label) SELECT id, encode(sha256('${BOB.token}'::bytea), 'hex'), 'e2e' FROM users WHERE email = '${BOB.email}' ON CONFLICT DO NOTHING;`;
  execSync(`docker compose -p ${project} -f ../../deploy/docker-compose.yml exec -T postgres psql -U kk -d kk -c "${sql}"`, { stdio: 'pipe' });
});

const bobHeaders = { Authorization: `Basic ${Buffer.from(`${BOB.email}:${BOB.token}`).toString('base64')}` };
async function bobId(page: Page) {
  return ((await (await page.request.get(`${V1}/user/current`, { headers: bobHeaders })).json()) as { accountId: string }).accountId;
}

test('restrictions: restricting hides the page from a non-admin (not found), granting shows it, editing is refused with a clear message', async ({ page, browser }) => {
  test.setTimeout(120_000);
  const p = await seedPage(page, `Restricted ${Date.now()}`, [para('Secret plan')]);
  const bob = await bobId(page);
  await login(page, p.url);

  // Admin restricts viewing through the dialog and adds Bob to the viewers.
  await page.getByRole('button', { name: 'Share' }).click();
  const dlg = page.getByRole('dialog');
  await dlg.getByRole('radio', { name: /Viewing and editing restricted/ }).click();
  await expect(page.getByRole('button', { name: 'Share, restricted' })).toBeVisible();
  await expect(dlg.getByRole('list', { name: 'Who can view' }).getByText('Admin')).toBeVisible();

  // A second account cannot see the page: it looks like any missing page.
  const ctx = await browser.newContext({ baseURL: process.env.E2E_BASE_URL });
  const other = await ctx.newPage();
  await login(other, '/wiki/home', BOB);
  await other.goto(p.url);
  await expect(other.getByRole('heading', { name: "We can't find that page" })).toBeVisible();

  // Grant view only to Bob; he can read but not edit.
  await dlg.getByLabel('Add to who can view', { exact: true }).fill('Bob');
  await dlg.getByRole('button', { name: /Bob Reader/ }).click();
  await expect(dlg.getByRole('list', { name: 'Who can view' }).getByText('Bob Reader')).toBeVisible();
  await other.reload();
  await expect(other.getByRole('heading', { level: 1, name: /Restricted/ })).toBeVisible();
  await other.getByRole('button', { name: 'Share, restricted' }).click();
  await other.getByRole('dialog').getByRole('radio', { name: 'No restrictions' }).click();
  await expect(other.getByText("You don't have permission to do that")).toBeVisible();
  await ctx.close();

  // Removing Bob again, then clearing everything.
  await dlg.getByRole('button', { name: 'Remove Bob Reader from who can view' }).click();
  await expect(dlg.getByText('Bob Reader')).toHaveCount(0);
  await dlg.getByRole('radio', { name: 'No restrictions' }).click();
  await expect(page.getByRole('button', { name: 'Share', exact: true })).toBeVisible();
  expect(bob).toBeTruthy();
});

test('groups and space permissions: create a group, add a member, grant it a permission, revoke it', async ({ page }) => {
  const s = await seedPage(page, `Perms ${Date.now()}`, [para('x')]);
  const name = `team-${uniqueKey()}`;
  await login(page, '/wiki/admin/groups');
  await page.getByLabel('New group name').fill(name);
  await page.getByRole('button', { name: 'Create group' }).click();
  await expect(page.getByRole('heading', { level: 2, name })).toBeVisible();
  await page.getByLabel('Add a member', { exact: true }).fill('Bob');
  await page.getByRole('button', { name: /Bob Reader/ }).click();
  await expect(page.getByRole('list', { name: 'Members' }).getByText('Bob Reader')).toBeVisible();

  await page.goto(`/wiki/spaces/${s.key}/settings`);
  await page.getByRole('tab', { name: 'Permissions' }).click();
  await page.getByLabel('Add a user or group', { exact: true }).fill(name);
  await page.getByRole('button', { name: new RegExp(name) }).click();
  const box = page.getByRole('checkbox', { name: `${name}: archive page` });
  await box.click();
  await expect(box).toBeChecked();
  await page.reload();
  await page.getByRole('tab', { name: 'Permissions' }).click();
  await expect(page.getByRole('checkbox', { name: `${name}: archive page` })).toBeChecked();
  await page.getByRole('checkbox', { name: `${name}: archive page` }).click();
  await expect(page.getByRole('checkbox', { name: `${name}: archive page` })).toHaveCount(0);

  // A non-admin sees the settings but cannot change them.
  const res = await page.request.post(`${V1}/space/${s.key}/permission`, { headers: bobHeaders, data: { subject: { type: 'group', identifier: '1' }, operation: { key: 'archive', target: 'page' } } });
  expect(res.status()).toBe(403);
});

test('watch a page and a space', async ({ page }) => {
  const s = await seedPage(page, `Watch ${Date.now()}`, [para('x')]);
  await login(page, s.url);
  await page.getByRole('button', { name: 'Watch', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Watching' })).toBeVisible();
  await page.reload();
  await expect(page.getByRole('button', { name: 'Watching' })).toBeVisible();
  await page.goto(`/wiki/spaces/${s.key}/settings`);
  await page.getByRole('button', { name: 'Watch this space' }).click();
  await expect(page.getByRole('button', { name: 'Stop watching this space' })).toBeVisible();
});

test('move a page with the dialog and by dragging in the tree', async ({ page }) => {
  const tag = Date.now().toString(36);
  const s = await seedPage(page, `Parent ${tag}`, [para('p')]);
  const mk = async (title: string) => (await api<{ id: string }>(page, 'post', '/pages', { spaceId: s.spaceId, status: 'current', title, parentId: s.id })).id;
  const a = await mk(`Alpha${tag}`);
  const b = await mk(`Beta${tag}`);
  const c = await mk(`Gamma${tag}`);
  await login(page, `/wiki/spaces/${s.key}/pages/${a}`);
  // Dialog: move Alpha inside Beta.
  await page.getByRole('button', { name: 'More actions', exact: true }).click();
  await page.getByRole('menuitem', { name: 'Move', exact: true }).click();
  const dlg = page.getByRole('dialog');
  await dlg.getByLabel('Find the destination page').fill(`Beta${tag}`);
  await dlg.getByRole('radio', { name: new RegExp(`^Beta${tag}`) }).check();
  await dlg.getByRole('button', { name: 'Move', exact: true }).click();
  await expect(page.getByText('Page moved')).toBeVisible();
  expect((await api<{ parentId: string }>(page, 'get', `/pages/${a}`)).parentId).toBe(b);
  // Drag: Gamma before Beta, through the tree.
  const tree = page.getByRole('tree', { name: 'Pages' });
  await expect(tree.getByRole('treeitem', { name: `Gamma${tag}` })).toBeVisible();
  const from = tree.getByRole('treeitem', { name: `Gamma${tag}` }).locator('.kk-tree__row');
  const to = tree.getByRole('treeitem', { name: `Beta${tag}` }).locator('.kk-tree__row');
  const fb = (await from.boundingBox())!;
  const tb = (await to.boundingBox())!;
  await page.mouse.move(fb.x + 60, fb.y + fb.height / 2);
  await page.mouse.down();
  await page.mouse.move(tb.x + 60, tb.y + tb.height / 2, { steps: 8 });
  await expect(to).toHaveAttribute('data-drop', 'append');
  await page.screenshot({ path: 'screenshots/ac-light-wide-drag-hint.png' });
  await page.mouse.move(900, 600);
  await page.mouse.up();
  await expect(to).not.toHaveAttribute('data-drop', /.+/);
  await from.dragTo(to, { targetPosition: { x: 40, y: 2 } });
  await expect
    .poll(async () => (await api<{ results: { id: string }[] }>(page, 'get', `/pages/${s.id}/direct-children`)).results.map((k) => k.id).join())
    .toBe([c, b].join());
});

for (const scheme of ['light', 'dark'] as const) {
  for (const [vpName, vp] of Object.entries(viewports)) {
    test(`access screens ${scheme} ${vpName}`, async ({ page }) => {
      test.setTimeout(120_000);
      await page.setViewportSize(vp);
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' });
      const shot = async (name: string) => {
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(200);
        await page.screenshot({ path: `screenshots/ac-${scheme}-${vpName}-${name}.png`, fullPage: name !== 'restrictions' });
        const r = await new AxeBuilder({ page: page as never }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
        for (const v of r.violations) violations.push(`${scheme}-${vpName}-${name}: ${v.id} (${v.impact}) ${v.nodes.slice(0, 2).map((n) => n.target.join(' ')).join(' | ')}`);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        if (overflow > 0) violations.push(`${scheme}-${vpName}-${name}: horizontal page overflow ${overflow}px`);
      };
      const s = await seedPage(page, `Look ${scheme}${vpName}${Date.now()}`, [para('Body text')]);
      const grp = await page.request.post(`${V1}/group`, { headers: auth, data: { name: `ops-${uniqueKey()}` } });
      const g = (await grp.json()) as { id: string; name: string };
      const bob = await bobId(page);
      await page.request.post(`${V1}/group/userByGroupId?groupId=${g.id}`, { headers: auth, data: { accountId: bob } });
      await page.request.post(`${V1}/space/${s.key}/permission`, { headers: auth, data: { subject: { type: 'group', identifier: g.id }, operation: { key: 'create', target: 'page' } } });
      await page.request.post(`${V1}/space/${s.key}/permission`, { headers: auth, data: { subject: { type: 'user', identifier: bob }, operation: { key: 'read', target: 'space' } } });
      await page.request.put(`${V1}/content/${s.id}/restriction/byOperation/read/byGroupId/${g.id}`, { headers: auth });
      await page.request.put(`${V1}/content/${s.id}/restriction/byOperation/read/user?accountId=${bob}`, { headers: auth });
      await login(page, s.url);
      await page.getByRole('button', { name: 'Share, restricted' }).click();
      await expect(page.getByRole('list', { name: 'Who can view' }).getByText('Bob Reader')).toBeVisible();
      await shot('restrictions');
      await page.keyboard.press('Escape');
      await page.goto(`/wiki/spaces/${s.key}/settings`);
      await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
      await shot('settings-details');
      await page.getByRole('tab', { name: 'Permissions' }).click();
      await expect(page.getByRole('table')).toBeVisible();
      await shot('settings-permissions');
      await page.goto('/wiki/admin/groups');
      await page.getByRole('button', { name: g.name }).click();
      await expect(page.getByRole('list', { name: 'Members' }).getByText('Bob Reader')).toBeVisible();
      await shot('groups');
      await page.getByRole('button', { name: 'Delete group', exact: true }).first().click();
      await shot('group-delete');
      await page.keyboard.press('Escape');
      await page.goto(s.url);
      await page.getByRole('button', { name: 'More actions', exact: true }).click();
      await page.getByRole('menuitem', { name: 'Move', exact: true }).click();
      await page.getByLabel('Find the destination page').fill('Look');
      await shot('move-dialog');
      expect(violations, violations.join('\n')).toEqual([]);
    });
  }
}
