// Dev-only stand-in for the backend: mirrors the v2 shapes the UI uses, in memory.
import type { IncomingMessage, ServerResponse } from 'node:http';
import type { Plugin } from 'vite';

const BASE = '/wiki/api/v2';
const USER = { email: 'admin@kaijukit.local', token: 'kk-dev-token', id: '1' };

interface Page {
  id: string;
  status: 'current' | 'draft' | 'trashed';
  title: string;
  spaceId: string;
  parentId: string | null;
  position: number;
  authorId: string;
  createdAt: string;
  version: { number: number; message: string; createdAt: string; authorId: string; minorEdit: boolean };
  body: string;
  draft?: { title: string; body: string };
  history: Page['version'][];
}
interface Space {
  id: string;
  key: string;
  name: string;
  type: string;
  status: string;
  authorId: string;
  createdAt: string;
  homepageId: string;
  description?: string;
}

const spaces: Space[] = [];
const pages: Page[] = [];
let seq = 1000;
const nid = () => String(++seq);
const now = () => new Date().toISOString();
const emptyBody = JSON.stringify({ version: 1, type: 'doc', content: [{ type: 'paragraph' }] });

class HttpError extends Error {
  constructor(public status: number, public detail: string) {
    super(detail);
  }
}

function newPage(p: { spaceId: string; title: string; parentId: string | null; status: Page['status']; body?: string }): Page {
  const v = { number: 1, message: '', createdAt: now(), authorId: USER.id, minorEdit: false };
  const siblings = pages.filter((x) => x.parentId === p.parentId && x.spaceId === p.spaceId);
  const page: Page = { id: nid(), authorId: USER.id, createdAt: now(), position: siblings.length, version: v, history: [v], body: p.body ?? emptyBody, ...p };
  pages.push(page);
  return page;
}

function checkTitle(spaceId: string, title: string, selfId?: string) {
  if (!title.trim()) throw new HttpError(400, 'Title is required.');
  if (pages.some((p) => p.spaceId === spaceId && p.status === 'current' && p.id !== selfId && p.title.toLowerCase() === title.toLowerCase()))
    throw new HttpError(400, `A page with the title "${title}" already exists in this space.`);
}

function pageJson(p: Page, getDraft: boolean) {
  const d = getDraft && p.draft;
  return {
    id: p.id,
    status: d ? 'draft' : p.status,
    title: d ? d.title : p.title,
    spaceId: p.spaceId,
    parentId: p.parentId ?? undefined,
    parentType: p.parentId ? 'page' : undefined,
    position: p.position,
    authorId: p.authorId,
    ownerId: p.authorId,
    createdAt: p.createdAt,
    version: p.version,
    body: { atlas_doc_format: { representation: 'atlas_doc_format', value: d ? d.body : p.body } },
    _links: { webui: `/spaces/${spaces.find((s) => s.id === p.spaceId)?.key}/pages/${p.id}`, base: '/wiki' },
  };
}
const bulk = (p: Page) => ({ ...pageJson(p, false), body: undefined });
const spaceJson = (s: Space) => ({ ...s, description: s.description ? { plain: { representation: 'plain', value: s.description } } : undefined, spaceOwnerId: s.authorId, _links: { webui: `/spaces/${s.key}` } });

function paged<T>(all: T[], q: URLSearchParams, url: string) {
  const limit = Math.min(250, Number(q.get('limit') ?? 25) || 25);
  const off = Number(q.get('cursor') ?? 0) || 0;
  const results = all.slice(off, off + limit);
  const next = off + limit < all.length ? `${BASE}${url}?${new URLSearchParams({ ...Object.fromEntries(q), cursor: String(off + limit) })}` : undefined;
  return { results, _links: next ? { next, base: '/wiki' } : { base: '/wiki' } };
}

const byPos = (a: Page, b: Page) => a.position - b.position || a.title.localeCompare(b.title);
const getPage = (id?: string) => pages.find((p) => p.id === id && p.status !== 'trashed') ?? (() => { throw new HttpError(404, 'Page not found.'); })();
const live = (p: Page) => p.status === 'current';

function ancestors(p: Page): Page[] {
  const out: Page[] = [];
  for (let c = p; c.parentId; ) {
    const parent = pages.find((x) => x.id === c.parentId);
    if (!parent) break;
    out.unshift(parent);
    c = parent;
  }
  return out;
}

function route(method: string, path: string, q: URLSearchParams, body: Record<string, any>): [number, unknown?] {
  let m: RegExpMatchArray | null;
  if (path === '/spaces' && method === 'GET') {
    const keys = q.getAll('keys');
    const status = q.get('status') ?? 'current';
    return [200, paged(spaces.filter((s) => (!keys.length || keys.includes(s.key)) && s.status === status).map(spaceJson), q, path)];
  }
  if (path === '/spaces' && method === 'POST') {
    const name = String(body.name ?? '').trim();
    const key = String(body.key ?? '').trim();
    if (!name) throw new HttpError(400, 'Space name is required.');
    if (!/^[A-Za-z0-9]{1,255}$/.test(key)) throw new HttpError(400, 'Space key may only contain letters and numbers.');
    if (spaces.some((s) => s.key.toLowerCase() === key.toLowerCase())) throw new HttpError(400, `A space with the key ${key} already exists.`);
    const s: Space = { id: nid(), key, name, type: 'global', status: 'current', authorId: USER.id, createdAt: now(), homepageId: '', description: body.description?.value };
    spaces.push(s);
    s.homepageId = newPage({ spaceId: s.id, title: name, parentId: null, status: 'current' }).id;
    return [201, spaceJson(s)];
  }
  if ((m = path.match(/^\/spaces\/(\d+)$/)) && method === 'GET') {
    const s = spaces.find((x) => x.id === m![1]);
    if (!s) throw new HttpError(404, 'Space not found.');
    return [200, spaceJson(s)];
  }
  if ((m = path.match(/^\/spaces\/(\d+)\/pages$/)) && method === 'GET') {
    const root = q.get('depth') === 'root';
    return [200, paged(pages.filter((p) => p.spaceId === m![1] && live(p) && (!root || !p.parentId)).sort(byPos).map(bulk), q, path)];
  }
  if (path === '/pages' && method === 'GET') {
    const ids = q.getAll('id');
    const list = pages.filter((p) => live(p) && (!ids.length || ids.includes(p.id)));
    if (q.get('sort') === '-modified-date') list.sort((a, b) => b.version.createdAt.localeCompare(a.version.createdAt));
    return [200, paged(list.map(bulk), q, path)];
  }
  if (path === '/pages' && method === 'POST') {
    const status = body.status === 'draft' ? 'draft' : 'current';
    if (!spaces.some((s) => s.id === body.spaceId)) throw new HttpError(404, 'Space not found.');
    if (status === 'current') checkTitle(body.spaceId, body.title ?? '');
    return [200, pageJson(newPage({ spaceId: body.spaceId, title: body.title ?? '', parentId: body.parentId ?? null, status, body: body.body?.value }), false)];
  }
  if ((m = path.match(/^\/pages\/(\d+)$/))) {
    const p = method === 'GET' || method === 'PUT' || method === 'DELETE' ? getPage(m[1]) : null;
    if (!p) throw new HttpError(405, 'Method not allowed.');
    if (method === 'GET') return [200, pageJson(p, q.get('get-draft') === 'true')];
    if (method === 'DELETE') {
      if (q.get('draft') === 'true' && p.status === 'current') delete p.draft;
      else p.status = 'trashed';
      return [204];
    }
    const next = body.version?.number as number;
    if (body.status === 'draft') {
      if (p.status === 'draft') Object.assign(p, { title: body.title, body: body.body.value });
      else p.draft = { title: body.title, body: body.body.value };
      return [200, pageJson(p, true)];
    }
    const expected = p.status === 'draft' ? p.version.number : p.version.number + 1;
    if (next !== expected) throw new HttpError(409, `Version must be ${expected}.`);
    checkTitle(p.spaceId, body.title, p.id);
    const v = { number: next, message: body.version?.message ?? '', createdAt: now(), authorId: USER.id, minorEdit: false };
    Object.assign(p, { status: 'current', title: body.title, body: body.body.value, version: v });
    p.history.push(v);
    delete p.draft;
    return [200, pageJson(p, false)];
  }
  if ((m = path.match(/^\/pages\/(\d+)\/(direct-children|children)$/)) && method === 'GET') {
    const kids = pages.filter((p) => p.parentId === getPage(m![1]).id && live(p)).sort(byPos);
    return [200, paged(kids.map((p) => ({ id: p.id, status: 'current', title: p.title, type: 'page', spaceId: p.spaceId, childPosition: p.position })), q, path)];
  }
  if ((m = path.match(/^\/pages\/(\d+)\/ancestors$/)) && method === 'GET') return [200, paged(ancestors(getPage(m[1])).map((p) => ({ id: p.id, type: 'page' })), q, path)];
  if ((m = path.match(/^\/pages\/(\d+)\/versions$/)) && method === 'GET') return [200, paged([...getPage(m[1]).history].reverse(), q, path)];
  throw new HttpError(404, 'No such endpoint.');
}

function readBody(req: IncomingMessage): Promise<Record<string, any>> {
  return new Promise((resolve) => {
    let s = '';
    req.on('data', (c) => (s += c));
    req.on('end', () => {
      try {
        resolve(s ? JSON.parse(s) : {});
      } catch {
        resolve({});
      }
    });
  });
}

function send(res: ServerResponse, status: number, data?: unknown) {
  res.statusCode = status;
  if (data === undefined) return res.end();
  res.setHeader('content-type', 'application/json');
  res.end(JSON.stringify(data));
}

export function mockApi(): Plugin {
  return {
    name: 'kk-mock-api',
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://x');
        if (!url.pathname.startsWith(BASE + '/')) return next();
        const fail = (status: number, detail: string) => send(res, status, { errors: [{ status, code: status === 401 ? 'UNAUTHORIZED' : 'ERROR', title: detail, detail }] });
        const auth = req.headers.authorization ?? '';
        const [email, token] = Buffer.from(auth.replace(/^Basic /, ''), 'base64').toString().split(':');
        if (email !== USER.email || token !== USER.token) return fail(401, 'Authentication failed.');
        try {
          const [status, data] = route(req.method ?? 'GET', url.pathname.slice(BASE.length), url.searchParams, await readBody(req));
          send(res, status, data);
        } catch (e) {
          if (e instanceof HttpError) fail(e.status, e.detail);
          else throw e;
        }
      });
    },
  };
}
