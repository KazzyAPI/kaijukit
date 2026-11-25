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

const auth = { Authorization: `Basic ${Buffer.from(`${ADMIN.email}:${ADMIN.token}`).toString('base64')}` };
const V2 = '/wiki/api/v2';
export const adfBody = (d: object) => ({ representation: 'atlas_doc_format', value: JSON.stringify(d) });
export const para = (...text: string[]) => ({ type: 'paragraph', content: text.map((t) => ({ type: 'text', text: t })) });
export const doc = (...content: object[]) => ({ version: 1, type: 'doc', content });

/** Creates a space and a published page through the API, so a feature test starts on the page view. */
export async function seedPage(page: Page, title: string, content: object[]) {
  const sp = await page.request.post(`${V2}/spaces`, { headers: auth, data: { name: `S ${title}`, key: uniqueKey() } });
  expect(sp.ok(), await sp.text()).toBeTruthy();
  const space = (await sp.json()) as { id: string; key: string };
  const pg = await page.request.post(`${V2}/pages`, { headers: auth, data: { spaceId: space.id, status: 'current', title, body: adfBody(doc(...content)) } });
  expect(pg.ok(), await pg.text()).toBeTruthy();
  const { id } = (await pg.json()) as { id: string };
  return { id, key: space.key, spaceId: space.id, url: `/wiki/spaces/${space.key}/pages/${id}` };
}

export async function api<T>(page: Page, method: 'get' | 'put' | 'post', path: string, data?: object): Promise<T> {
  const r = await page.request[method](`${V2}${path}`, { headers: auth, data });
  expect(r.ok(), await r.text()).toBeTruthy();
  return (await r.json()) as T;
}

/** Selects the nth occurrence of `text` inside one rendered text node, as a user drag would. */
export async function selectText(page: Page, text: string, nth = 0) {
  await expect(page.locator('.kk-canvas .kk-doc p').first()).toBeVisible();
  await page.evaluate(
    ([needle, n]) => {
      const walker = document.createTreeWalker(document.querySelector('.kk-canvas .kk-doc')!, NodeFilter.SHOW_TEXT);
      let seen = 0;
      for (let t = walker.nextNode() as Text | null; t; t = walker.nextNode() as Text | null) {
        const node: Text = t;
        for (let i = node.data.indexOf(needle as string); i !== -1; i = node.data.indexOf(needle as string, i + (needle as string).length)) {
          if (seen++ !== n) continue;
          const r = document.createRange();
          r.setStart(node, i);
          r.setEnd(node, i + (needle as string).length);
          getSelection()!.removeAllRanges();
          getSelection()!.addRange(r);
          return;
        }
      }
      throw new Error(`text not found: ${String(needle)}`);
    },
    [text, nth] as const,
  );
}
