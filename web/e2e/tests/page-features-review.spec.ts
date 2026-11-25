import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { ADMIN, adfBody, api, doc, login, para, seedPage, selectText } from './helpers';

test.skip(!process.env.E2E_BASE_URL, 'these specs need the real server (E2E_BASE_URL)');

const auth = { Authorization: `Basic ${Buffer.from(`${ADMIN.email}:${ADMIN.token}`).toString('base64')}` };
const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==', 'base64');
const viewports = { wide: { width: 1280, height: 800 }, narrow: { width: 390, height: 844 } };
const violations: string[] = [];

async function seed(page: Page, tag: string) {
  const p = await seedPage(page, `Review ${tag}`, [
    { type: 'heading', attrs: { level: 2 }, content: [{ type: 'text', text: 'Launch plan' }] },
    para('The launch plan depends on the migration finishing first. Review the plan with the team before the freeze, and keep the plan short.'),
    { type: 'taskList', attrs: { localId: 'tl' }, content: [{ type: 'taskItem', attrs: { localId: 't1', state: 'TODO' }, content: [{ type: 'text', text: `Confirm the freeze date ${tag}` }] }, { type: 'taskItem', attrs: { localId: 't2', state: 'DONE' }, content: [{ type: 'text', text: `Book the review room ${tag}` }] }] },
  ]);
  const make = (kind: string, data: object) => api<{ id: string }>(page, 'post', `/${kind}-comments`, { pageId: p.id, body: adfBody(doc(para(String((data as { t: string }).t)))), ...(data as { inlineCommentProperties?: object }) });
  const open = await make('inline', { t: 'Is the migration date firm?', inlineCommentProperties: { textSelection: 'migration', textSelectionMatchCount: 1, textSelectionMatchIndex: 0 } });
  const done = await make('inline', { t: 'Fixed in the latest draft.', inlineCommentProperties: { textSelection: 'freeze', textSelectionMatchCount: 2, textSelectionMatchIndex: 0 } });
  await api(page, 'put', `/inline-comments/${done.id}`, { resolved: true });
  await api(page, 'post', `/inline-comments`, { parentCommentId: open.id, body: adfBody(doc(para('Yes, confirmed Monday.'))) });
  const foot = await make('footer', { t: 'Great summary. One question: who signs off on the freeze?' });
  await api(page, 'post', '/footer-comments', { parentCommentId: foot.id, body: adfBody(doc(para('The release manager does.'))) });
  await make('footer', { t: 'Thanks, this is useful.' });
  for (const name of ['roadmap', 'q4-launch']) {
    expect((await page.request.post(`/wiki/rest/api/content/${p.id}/label`, { headers: auth, data: [{ prefix: 'global', name }] })).ok()).toBeTruthy();
  }
  const me = (await (await page.request.get('/wiki/rest/api/user/current', { headers: auth })).json()) as { accountId: string };
  expect((await page.request.post(`/wiki/rest/api/content/${p.id}/restriction`, { headers: auth, data: [{ operation: 'update', restrictions: { user: [{ accountId: me.accountId }] } }] })).ok()).toBeTruthy();
  expect((await page.request.put(`/wiki/rest/api/content/${p.id}/like`, { headers: auth })).ok()).toBeTruthy();
  for (const [name, mimeType, buffer] of [['diagram.png', 'image/png', PNG], ['launch-notes-with-a-rather-long-file-name.txt', 'text/plain', Buffer.from('notes')]] as const) {
    expect((await page.request.post(`/wiki/rest/api/content/${p.id}/child/attachment`, { headers: auth, multipart: { file: { name, mimeType, buffer }, minorEdit: 'true' } })).ok()).toBeTruthy();
  }
  await api(page, 'put', `/pages/${p.id}`, { id: p.id, status: 'current', title: `Review ${tag}`, body: { representation: 'atlas_doc_format', value: (await api<{ body: { atlas_doc_format: { value: string } } }>(page, 'get', `/pages/${p.id}?body-format=atlas_doc_format`)).body.atlas_doc_format.value }, version: { number: 2, message: 'Tighten the plan' } });
  return p;
}

for (const scheme of ['light', 'dark'] as const) {
  for (const [vpName, vp] of Object.entries(viewports)) {
    test(`page features ${scheme} ${vpName}`, async ({ page }) => {
      test.setTimeout(120_000);
      await page.setViewportSize(vp);
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' });
      const shot = async (name: string, fullPage = false) => {
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(200);
        await page.screenshot({ path: `screenshots/pf-${scheme}-${vpName}-${name}.png`, fullPage });
        const r = await new AxeBuilder({ page: page as never }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
        for (const v of r.violations) violations.push(`${scheme}-${vpName}-${name}: ${v.id} (${v.impact}) ${v.nodes.slice(0, 2).map((n) => n.target.join(' ')).join(' | ')}`);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        if (overflow > 0) violations.push(`${scheme}-${vpName}-${name}: horizontal page overflow ${overflow}px`);
      };
      const tag = `${scheme}${vpName}${Date.now()}`;
      const p = await seed(page, tag);
      await login(page, p.url);
      await expect(page.locator('.kk-canvas mark.kk-annotation')).toHaveCount(1);
      await expect(page.getByRole('region', { name: /^Attachments/ }).getByRole('listitem')).toHaveCount(2);
      await expect(page.getByRole('button', { name: 'Share, restricted' })).toBeVisible();
      await shot('view', true);
      await page.getByRole('button', { name: 'Share, restricted' }).click();
      await shot('lock-state');
      await page.keyboard.press('Escape');
      await page.getByRole('region', { name: 'Labels' }).getByRole('link', { name: 'roadmap' }).click();
      await expect(page.getByRole('heading', { level: 1, name: 'Label: roadmap' })).toBeVisible();
      await expect(page.getByRole('list', { name: 'Pages with this label' })).toBeVisible();
      await shot('label-view');
      await page.goBack();
      await page.getByRole('button', { name: 'Search', exact: true }).click();
      await page.getByLabel('Search pages', { exact: true }).fill('migration finishing');
      await expect(page.getByRole('dialog').getByRole('list', { name: 'Results' })).toBeVisible();
      await shot('quick-search-text');
      await page.keyboard.press('Escape');

      await page.getByRole('button', { name: /^Inline comments/ }).click();
      await page.locator('.kk-canvas mark.kk-annotation').click();
      await shot('inline-panel');
      await page.getByRole('complementary').getByRole('button', { name: /^Resolved/ }).click();
      await shot('inline-resolved');
      await page.getByRole('complementary').getByRole('button', { name: /^Open/ }).click();

      await selectText(page, 'plan', 1);
      await shot('inline-selection');
      await page.getByRole('button', { name: 'Comment', exact: true }).click();
      await page.getByLabel('New inline comment').fill('Should this say "short" or "concise"?');
      await shot('inline-composer');
      await page.getByRole('complementary').getByRole('button', { name: 'Cancel' }).click();

      await page.getByRole('button', { name: 'Close inline comments' }).click();
      const comments = page.getByRole('region', { name: /^Comments/ });
      await comments.getByRole('button', { name: 'Write a comment...' }).scrollIntoViewIfNeeded();
      await comments.getByRole('button', { name: 'Write a comment...' }).click();
      await comments.getByLabel('Add a comment').fill('A draft comment that is still being written.');
      await shot('footer-composer');
      await comments.getByRole('button', { name: 'Cancel' }).click();
      await comments.getByRole('button', { name: 'Delete' }).first().click();
      await shot('delete-dialog');
      await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();

      await page.getByRole('region', { name: 'Labels' }).getByRole('button', { name: 'Add label' }).click();
      await page.getByLabel('New label').fill('two words');
      await page.getByLabel('New label').press('Enter');
      await shot('label-error');

      await page.goto(`/wiki/spaces/${p.key}/pages/history/${p.id}`);
      await expect(page.getByRole('navigation', { name: 'Versions' })).toBeVisible();
      await shot('history-current');
      await page.getByRole('navigation', { name: 'Versions' }).getByRole('button', { name: /Version 1/ }).click();
      await expect(page.getByText(/You are viewing version 1/)).toBeVisible();
      await shot('history-old');
      await page.getByRole('button', { name: 'Restore this version' }).click();
      await shot('history-restore');
      await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();

      await page.goto('/wiki/tasks');
      await expect(page.getByRole('checkbox', { name: `Confirm the freeze date ${tag}` })).toBeVisible();
      await shot('tasks');
      await page.getByRole('button', { name: 'Complete', exact: true }).click();
      await expect(page.getByRole('checkbox', { name: `Book the review room ${tag}` })).toBeChecked();
      await shot('tasks-complete');
      await page.getByRole('button', { name: 'Assigned to me' }).click();
      await expect(page.getByText('No completed tasks')).toBeVisible();
      await shot('tasks-empty');

      await page.goto(`/wiki/search?text=${encodeURIComponent(`Review ${tag}`)}`);
      await expect(page.getByRole('list', { name: 'Search results' })).toBeVisible();
      await shot('search');
      await page.goto('/wiki/search?text=Nothing%20by%20this%20title');
      await expect(page.getByText(/No pages match/)).toBeVisible();
      await shot('search-none');
      await page.goto('/wiki/search');
      await shot('search-empty');
    });
  }
}

test.afterAll(() => {
  expect(violations, violations.join('\n')).toEqual([]);
});
