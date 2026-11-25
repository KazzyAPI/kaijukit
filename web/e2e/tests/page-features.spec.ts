import { expect, test } from '@playwright/test';
import { api, doc, login, para, seedPage, selectText } from './helpers';

test.skip(!process.env.E2E_BASE_URL, 'these specs need the real server (E2E_BASE_URL)');

const task = (localId: string, text: string, state = 'TODO') => ({ type: 'taskItem', attrs: { localId, state }, content: [{ type: 'text', text }] });

test('footer comments: add, reply, edit, delete', async ({ page }) => {
  const p = await seedPage(page, 'Footer comments', [para('Body text')]);
  await login(page, p.url);
  const comments = page.getByRole('region', { name: /^Comments/ });
  await expect(comments.getByText('No comments yet.')).toBeVisible();

  await comments.getByRole('button', { name: 'Write a comment...' }).click();
  await page.getByLabel('Add a comment').fill('First comment\nsecond line');
  await page.getByLabel('Add a comment').press('ControlOrMeta+Enter');
  const first = comments.locator('.kk-comment').first();
  await expect(first.locator('.kk-comment__body p')).toHaveText(['First comment', 'second line']);
  await expect(comments.getByRole('heading', { name: 'Comments (1)' })).toBeVisible();

  await first.getByRole('button', { name: /^Reply to/ }).click();
  await page.getByLabel('Reply', { exact: true }).fill('A reply');
  await page.getByRole('button', { name: 'Reply', exact: true }).click();
  const reply = first.getByRole('list', { name: /^Replies to/ }).locator('.kk-comment');
  await expect(reply.locator('.kk-comment__body')).toHaveText('A reply');

  await reply.getByRole('button', { name: 'Edit' }).click();
  await page.getByLabel('Edit comment').fill('A reply, edited');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(reply.locator('.kk-comment__body')).toHaveText('A reply, edited');
  await expect(reply.getByText('edited', { exact: true })).toBeVisible();

  await page.reload();
  await expect(comments.getByText('A reply, edited')).toBeVisible();

  await reply.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click();
  await expect(comments.getByText('A reply, edited')).toHaveCount(0);

  await first.getByRole('button', { name: 'Delete' }).first().click();
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click();
  await expect(comments.getByText('No comments yet.')).toBeVisible();
  const left = await api<{ results: unknown[] }>(page, 'get', `/pages/${p.id}/footer-comments`);
  expect(left.results).toHaveLength(0);
});

test('inline comments: select, comment, highlight, panel, resolve, reopen', async ({ page }) => {
  const p = await seedPage(page, 'Inline comments', [para('alpha beta alpha gamma')]);
  await login(page, p.url);
  await selectText(page, 'alpha', 1);
  await page.getByRole('button', { name: 'Comment', exact: true }).click();
  const panel = page.getByRole('complementary', { name: 'Inline comments' });
  await expect(panel.locator('.kk-quote').first()).toHaveText('alpha');
  await panel.getByLabel('New inline comment').fill('Why twice?');
  await panel.getByRole('button', { name: 'Comment', exact: true }).click();

  const mark = page.locator('.kk-canvas mark.kk-annotation');
  await expect(mark).toHaveText('alpha');
  await expect(panel.getByText('Why twice?')).toBeVisible();
  const saved = await api<{ body: { atlas_doc_format: { value: string } } }>(page, 'get', `/pages/${p.id}?body-format=atlas_doc_format`);
  const nodes = (JSON.parse(saved.body.atlas_doc_format.value) as { content: { content: { text: string; marks?: unknown[] }[] }[] }).content[0]!.content;
  expect(nodes.map((n) => [n.text, !!n.marks])).toEqual([['alpha beta ', false], ['alpha', true], [' gamma', false]]);

  await page.reload();
  await expect(mark).toHaveText('alpha');
  await mark.click();
  await expect(page.getByRole('complementary', { name: 'Inline comments' }).locator('.kk-comment[data-focused]')).toContainText('Why twice?');
  await expect(mark).toHaveClass(/is-focused/);

  await panel.getByRole('button', { name: 'Resolve', exact: true }).click();
  await expect(mark).toHaveCount(0);
  await expect(panel.getByText('No open inline comments')).toBeVisible();
  await panel.getByRole('button', { name: /^Resolved/ }).click();
  await expect(panel.getByText('Why twice?')).toBeVisible();
  await panel.getByRole('button', { name: 'Reopen' }).click();
  await expect(panel.getByText('No open inline comments').or(panel.getByText('Nothing resolved yet'))).toBeVisible();
  await expect(mark).toHaveText('alpha');
});

test('inline comments: a selection across formatting is refused with a reason', async ({ page }) => {
  const p = await seedPage(page, 'Inline refused', [{ type: 'paragraph', content: [{ type: 'text', text: 'plain ' }, { type: 'text', text: 'bold', marks: [{ type: 'strong' }] }] }]);
  await login(page, p.url);
  await expect(page.locator('.kk-canvas .kk-doc strong')).toBeVisible();
  await page.evaluate(() => {
    const strong = document.querySelector('.kk-canvas .kk-doc strong')!;
    const r = document.createRange();
    r.setStart(strong.parentElement!.firstChild!, 2);
    r.setEnd(strong.firstChild!, 2);
    getSelection()!.removeAllRanges();
    getSelection()!.addRange(r);
  });
  await page.getByRole('button', { name: 'Comment', exact: true }).click();
  await expect(page.getByText("Can't comment on this selection")).toBeVisible();
  await expect(page.getByRole('complementary', { name: 'Inline comments' })).toHaveCount(0);
});

test('labels: add, reject spaces, remove, persist', async ({ page }) => {
  const p = await seedPage(page, 'Labels', [para('x')]);
  await login(page, p.url);
  const labels = page.getByRole('region', { name: 'Labels' });
  await expect(labels.getByText('No labels')).toBeVisible();
  await labels.getByRole('button', { name: 'Add label' }).click();
  const input = labels.getByLabel('New label');
  await input.fill('two words');
  await input.press('Enter');
  await expect(labels.getByRole('alert')).toContainText("can't contain spaces");
  await input.fill('Roadmap');
  await input.press('Enter');
  await input.fill('q4,');
  await expect(labels.getByRole('list', { name: 'Labels' }).getByRole('listitem')).toHaveText(['roadmap', 'q4']);
  await page.reload();
  const chips = page.getByRole('region', { name: 'Labels' }).getByRole('list', { name: 'Labels' }).getByRole('listitem');
  await expect(chips).toHaveText(['roadmap', 'q4']);
  await page.getByRole('button', { name: 'Remove label roadmap' }).click();
  await expect(chips).toHaveText(['q4']);
  const server = await api<{ results: { name: string }[] }>(page, 'get', `/pages/${p.id}/labels`);
  expect(server.results.map((l) => l.name)).toEqual(['q4']);
});

test('like: toggle, count, persist', async ({ page }) => {
  const p = await seedPage(page, 'Likes', [para('x')]);
  await login(page, p.url);
  const like = page.getByRole('button', { name: /^Like/ });
  await expect(like).toHaveAttribute('aria-pressed', 'false');
  await expect(page.getByLabel('0 likes')).toBeVisible();
  await like.click();
  await expect(page.getByRole('button', { name: /^Liked/ })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByLabel('1 like')).toBeVisible();
  await page.reload();
  await expect(page.getByRole('button', { name: /^Liked/ })).toHaveAttribute('aria-pressed', 'true');
  await page.getByRole('button', { name: /^Liked/ }).click();
  await expect(page.getByLabel('0 likes')).toBeVisible();
  expect((await api<{ count: number }>(page, 'get', `/pages/${p.id}/likes/count`)).count).toBe(0);
});

test('history: list versions, view an old one, restore it', async ({ page }) => {
  const p = await seedPage(page, 'History', [para('first draft text')]);
  for (const [n, text] of [[2, 'second text'], [3, 'third text']] as const) {
    await api(page, 'put', `/pages/${p.id}`, { id: p.id, status: 'current', title: 'History', body: { representation: 'atlas_doc_format', value: JSON.stringify(doc(para(text))) }, version: { number: n, message: `edit ${n}` } });
  }
  await login(page, p.url);
  await expect(page.getByText('third text')).toBeVisible();
  await page.getByRole('link', { name: 'Version 3' }).click();
  const list = page.getByRole('navigation', { name: 'Versions' });
  await expect(list.getByRole('listitem')).toHaveCount(3);
  await expect(list.getByRole('listitem').first()).toContainText('Current');
  await expect(page.getByText('This is the current version (3).')).toBeVisible();
  await list.getByRole('button', { name: /Version 1/ }).click();
  await expect(page.getByText('first draft text')).toBeVisible();
  await expect(page.getByText(/You are viewing version 1/)).toBeVisible();
  await page.getByRole('button', { name: 'Restore this version' }).click();
  await page.getByLabel('Version comment (optional)').fill('back to the start');
  await page.getByRole('dialog').getByRole('button', { name: 'Restore' }).click();
  await expect(page).toHaveURL(new RegExp(`${p.url}/history$`));
  await expect(page.getByText('first draft text')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Version 4' })).toBeVisible();
  const versions = await api<{ results: { number: number; message: string }[] }>(page, 'get', `/pages/${p.id}/versions`);
  expect(versions.results.at(-1)).toMatchObject({ number: 4, message: 'back to the start' });
});

test('attachments: upload, download, delete', async ({ page }) => {
  const p = await seedPage(page, 'Attachments', [para('x')]);
  await login(page, p.url);
  const section = page.getByRole('region', { name: /^Attachments/ });
  await expect(section.getByText('No attachments.')).toBeVisible();
  await section.getByLabel('Choose files to attach').setInputFiles([{ name: 'notes.txt', mimeType: 'text/plain', buffer: Buffer.from('hello attachment') }]);
  const row = section.getByRole('listitem').filter({ hasText: 'notes.txt' });
  await expect(row).toContainText('16 B');
  const download = page.waitForEvent('download');
  await row.getByRole('button', { name: 'Download notes.txt' }).click();
  const d = await download;
  expect(d.suggestedFilename()).toBe('notes.txt');
  expect((await import('node:fs')).readFileSync((await d.path())!, 'utf8')).toBe('hello attachment');
  await row.getByRole('button', { name: 'Delete notes.txt' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Delete' }).click();
  await expect(section.getByText('No attachments.')).toBeVisible();
  const left = await api<{ results: unknown[] }>(page, 'get', `/pages/${p.id}/attachments`);
  expect(left.results).toHaveLength(0);
});

test('tasks page lists tasks across pages and toggles completion', async ({ page }) => {
  const run = Date.now();
  const [ship, notes] = [`ship the thing ${run}`, `write notes ${run}`];
  const a = await seedPage(page, `Tasks A ${run}`, [{ type: 'taskList', attrs: { localId: 'la' }, content: [task('a1', ship), task('a2', notes)] }]);
  await login(page, '/wiki/tasks');
  const box = page.getByRole('checkbox', { name: ship });
  await expect(box).not.toBeChecked();
  await expect(page.getByRole('link', { name: `Tasks A ${run}` })).toBeVisible();
  await box.click();
  await expect(box).toHaveCount(0);
  await expect(page.getByRole('checkbox', { name: notes })).toBeVisible();
  await page.getByRole('button', { name: 'Complete', exact: true }).click();
  await expect(page.getByRole('checkbox', { name: ship })).toBeChecked();
  const body = await api<{ body: { atlas_doc_format: { value: string } } }>(page, 'get', `/pages/${a.id}?body-format=atlas_doc_format`);
  expect(body.body.atlas_doc_format.value).toContain('"state":"DONE"');
  await page.getByRole('checkbox', { name: ship }).click();
  await expect(page.getByRole('checkbox', { name: ship })).toHaveCount(0);
});

test('search page finds a page by exact title, and quick search links to it', async ({ page }) => {
  const title = `Findable ${Date.now()}`;
  await seedPage(page, title, [para('some searchable body words')]);
  await login(page);
  await page.getByRole('button', { name: 'Search' }).click();
  await page.getByLabel('Search pages by title').fill(title);
  await page.getByRole('link', { name: /on the results page/ }).click();
  await expect(page).toHaveURL(/\/wiki\/search\?text=/);
  const results = page.getByRole('list', { name: 'Search results' });
  await expect(results.getByRole('link', { name: title })).toBeVisible();
  await expect(results).toContainText('some searchable body words');
  await page.getByLabel('Search pages by title').fill('No such page anywhere');
  await page.getByRole('main').getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page.getByText('No page is titled "No such page anywhere"')).toBeVisible();
});
