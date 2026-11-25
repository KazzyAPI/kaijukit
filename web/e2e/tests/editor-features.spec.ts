import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { ADMIN, createSpace, login } from './helpers';

const auth = { Authorization: `Basic ${Buffer.from(`${ADMIN.email}:${ADMIN.token}`).toString('base64')}` };
// 1x1 transparent PNG
const END = process.platform === 'darwin' ? 'Meta+ArrowDown' : 'Control+End';
const ALL = 'ControlOrMeta+a';
const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==', 'base64');

type N = { type: string; attrs?: Record<string, unknown>; content?: N[]; marks?: unknown[]; text?: string };
const strip = (n: unknown): unknown => JSON.parse(JSON.stringify(n).replace(/"localId":"[^"]+"/g, '"localId":"ID"'));

async function savedAdf(page: Page, id: string): Promise<N[]> {
  await expect(page.locator('.kk-edit__chip')).toContainText('Saved', { timeout: 8000 });
  const r = await page.request.get(`/wiki/api/v2/pages/${id}?body-format=atlas_doc_format&get-draft=true`, { headers: auth });
  return (JSON.parse(((await r.json()) as { body: { atlas_doc_format: { value: string } } }).body.atlas_doc_format.value) as { content: N[] }).content;
}
const find = (nodes: N[], type: string) => nodes.find((n) => n.type === type);

test('editor features write ADF and render in light and dark at both widths', async ({ page }) => {
  test.setTimeout(240_000);
  const violations: string[] = [];
  await login(page);
  await createSpace(page, `Features ${Date.now()}`);
  await page.getByRole('button', { name: 'Create page', exact: true }).click();
  await expect(page).toHaveURL(/edit-v2\/(\d+)$/);
  const id = page.url().match(/edit-v2\/(\d+)/)![1]!;
  await page.getByLabel('Title', { exact: true }).fill('Editor features');
  await page.getByLabel('Title', { exact: true }).press('Enter');
  const editor = page.getByRole('textbox', { name: 'Page content' });
  const pick = async (tool: string, item: string | RegExp, dialog = false) => {
    await page.getByRole('button', { name: tool, exact: true }).click();
    await page.getByRole('menuitem', { name: item }).click();
    if (!dialog) await expect(editor).toBeFocused();
  };
  const insert = async (item: string | RegExp, dialog = false) => pick('Insert', item, dialog);
  const end = async () => {
    await page.keyboard.press(END);
  };

  await page.keyboard.type('Colored centered text');
  await page.keyboard.press(ALL);
  await pick('Text color', 'Red');
  await pick('Highlight', 'Yellow');
  await pick('Alignment', 'Align center');
  await pick('Alignment', 'Indent');
  await end();
  await page.keyboard.press('Enter');
  await pick('Alignment', 'Align left');
  await pick('Alignment', 'Outdent');

  await page.keyboard.type('[] Write the spec\nReview it');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter');
  await page.keyboard.type('<> Ship on Friday');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter');

  await insert('Info panel');
  await page.keyboard.type('Panel body');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter');

  await insert('Expand');
  await page.getByLabel('Expand title').fill('More detail');
  await editor.locator('.kk-expand p').click();
  await page.keyboard.type('Hidden body');
  await end();

  await page.getByRole('button', { name: 'Layouts' }).click();
  await page.getByRole('menuitem', { name: 'Three columns' }).click();
  await page.keyboard.type('One');
  await page.keyboard.press(END);

  await insert('Status', true);
  await page.getByRole('dialog').getByLabel('Text').fill('In review');
  await page.getByRole('dialog').getByLabel('Color').selectOption('purple');
  await page.getByRole('dialog').getByRole('button', { name: 'Insert' }).click();
  await expect(editor).toBeFocused();
  await insert('Date', true);
  await page.getByRole('dialog').getByLabel('Date').fill('2026-11-25');
  await page.getByRole('dialog').getByRole('button', { name: 'Insert' }).click();
  await expect(editor).toBeFocused();

  await page.keyboard.type(' @');
  await expect(page.getByRole('option').first()).toBeVisible();
  await page.keyboard.press('Enter');
  await page.keyboard.type(':smi');
  await expect(page.getByRole('listbox', { name: 'Emoji' })).toBeVisible();
  await page.keyboard.press('Enter');
  await end();
  await page.keyboard.press('Enter');
  await pick('Text style', 'Heading 2');
  await page.keyboard.type('Section heading');
  await page.keyboard.press('Enter');
  await insert('Table of contents');

  await page.locator('input[type=file]').setInputFiles({ name: 'pixel.png', mimeType: 'image/png', buffer: PNG });
  await expect(editor.locator('img.kk-media__img')).toBeVisible();
  await page.locator('input[type=file]').setInputFiles({ name: 'notes.txt', mimeType: 'text/plain', buffer: Buffer.from('hello') });
  await expect(editor.locator('.kk-file')).toHaveText('notes.txt');

  const adf = strip(await savedAdf(page, id)) as N[];
  expect(adf[0]).toEqual({
    type: 'paragraph',
    marks: [{ type: 'alignment', attrs: { align: 'center' } }, { type: 'indentation', attrs: { level: 1 } }],
    content: [{ type: 'text', text: 'Colored centered text', marks: [{ type: 'textColor', attrs: { color: '#bf2600' } }, { type: 'backgroundColor', attrs: { color: '#fff0b3' } }] }],
  });
  expect(find(adf, 'taskList')).toEqual({
    type: 'taskList',
    attrs: { localId: 'ID' },
    content: [
      { type: 'taskItem', attrs: { localId: 'ID', state: 'TODO' }, content: [{ type: 'text', text: 'Write the spec' }] },
      { type: 'taskItem', attrs: { localId: 'ID', state: 'TODO' }, content: [{ type: 'text', text: 'Review it' }] },
    ],
  });
  expect(find(adf, 'decisionList')).toEqual({ type: 'decisionList', attrs: { localId: 'ID' }, content: [{ type: 'decisionItem', attrs: { localId: 'ID', state: 'DECIDED' }, content: [{ type: 'text', text: 'Ship on Friday' }] }] });
  expect(find(adf, 'panel')).toEqual({ type: 'panel', attrs: { panelType: 'info' }, content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Panel body' }] }] });
  expect(find(adf, 'expand')).toEqual({ type: 'expand', attrs: { title: 'More detail' }, content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Hidden body' }] }] });
  expect(find(adf, 'layoutSection')?.content?.map((c) => c.attrs?.width)).toEqual([33.33, 33.33, 33.33]);
  const descendants = (n: N): N[] => (n.content ?? []).flatMap((c) => [c, ...descendants(c)]);
  const inline = adf.flatMap(descendants);
  expect(inline).toContainEqual({ type: 'status', attrs: { text: 'In review', color: 'purple', localId: 'ID' } });
  expect(inline).toContainEqual({ type: 'date', attrs: { timestamp: '1795564800000' } });
  expect(inline.find((n) => n.type === 'mention')).toMatchObject({ attrs: { id: expect.any(String), text: expect.stringMatching(/^@./) } });
  expect(inline).toContainEqual({ type: 'emoji', attrs: { shortName: ':smile:', id: '1f604', text: '😄' } });
  expect(find(adf, 'extension')?.attrs?.extensionKey).toBe('toc');
  const single = find(adf, 'mediaSingle')!;
  expect(single).toMatchObject({ attrs: { layout: 'center' }, content: [{ type: 'media', attrs: { type: 'file', __fileName: 'pixel.png' } }] });
  expect(find(adf, 'mediaGroup')).toMatchObject({ content: [{ type: 'media', attrs: { __fileName: 'notes.txt' } }] });

  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Publish' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Editor features' })).toBeVisible();
  const view = page.locator('.kk-doc');
  await expect(view.locator('.kk-panel')).toHaveText('Panel body');
  await expect(view.locator('.kk-status')).toHaveText('IN REVIEW');
  await expect(view.locator('.kk-toc a')).toHaveText('Section heading');
  await expect(view.locator('img.kk-media__img')).toBeVisible();
  const viewUrl = page.url();
  const editUrl = viewUrl.replace(/\/pages\/(\d+).*/, '/pages/edit-v2/$1');

  for (const scheme of ['light', 'dark'] as const) {
    for (const [name, vp] of [['wide', { width: 1280, height: 900 }], ['narrow', { width: 390, height: 844 }]] as const) {
      await page.setViewportSize(vp);
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' });
      for (const [mode, url] of [['view', viewUrl], ['edit', editUrl]] as const) {
        await page.goto(url);
        await expect(page.locator('.kk-doc .kk-panel')).toBeVisible();
        await expect(page.locator('.kk-doc .kk-media__img')).toBeVisible();
        await page.waitForTimeout(200);
        await page.screenshot({ path: `screenshots/features-${scheme}-${name}-${mode}.png`, fullPage: true });
        const r = await new AxeBuilder({ page: page as never }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
        for (const v of r.violations) violations.push(`${scheme}-${name}-${mode}: ${v.id} (${v.impact}) ${v.nodes.slice(0, 2).map((n) => n.target.join(' ')).join(' | ')}`);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        if (overflow > 0) violations.push(`${scheme}-${name}-${mode}: horizontal page overflow ${overflow}px`);
      }
    }
  }
  console.log(violations.length ? `FEATURE FINDINGS:\n${[...new Set(violations)].join('\n')}` : 'FEATURE FINDINGS: none');
  expect(violations).toEqual([]);
});
