const BASE = '/wiki/api/v2';

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

export interface Paged<T> {
  results: T[];
  _links?: { next?: string };
}
export interface Space {
  id: string;
  key: string;
  name: string;
  status: string;
  homepageId?: string;
  createdAt: string;
  description?: { plain?: { value: string } };
}
export interface Version {
  number: number;
  message?: string;
  createdAt: string;
  authorId?: string;
}
export interface Page {
  id: string;
  status: 'current' | 'draft' | 'archived' | 'trashed';
  title: string;
  spaceId: string;
  parentId?: string;
  authorId?: string;
  createdAt: string;
  version: Version;
  body?: { atlas_doc_format?: { value: string } };
}
export interface ChildPage {
  id: string;
  title: string;
  spaceId: string;
}

let authHeader: string | null = null;
export const setCredentials = (c: { email: string; token: string } | null) => {
  authHeader = c ? `Basic ${btoa(`${c.email}:${c.token}`)}` : null;
};

async function request<T>(method: string, path: string, opts: { query?: Record<string, string | string[] | number | boolean | undefined>; body?: unknown; auth?: string } = {}): Promise<T> {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(opts.query ?? {})) {
    if (v === undefined) continue;
    for (const x of Array.isArray(v) ? v : [v]) q.append(k, String(x));
  }
  const qs = q.toString() ? `?${q}` : '';
  let res: Response;
  try {
    res = await fetch(`${BASE}${path}${qs}`, {
      method,
      headers: { Accept: 'application/json', ...(opts.body ? { 'Content-Type': 'application/json' } : {}), Authorization: opts.auth ?? authHeader ?? '' },
      body: opts.body ? JSON.stringify(opts.body) : undefined,
    });
  } catch {
    throw new ApiError(0, 'Cannot reach the server. Check your connection.');
  }
  if (res.status === 204) return undefined as T;
  const text = await res.text();
  let json: unknown;
  try {
    json = text ? JSON.parse(text) : undefined;
  } catch {
    json = undefined;
  }
  if (!res.ok) {
    const j = json as { errors?: { title?: string; detail?: string }[]; message?: string } | undefined;
    const e = j?.errors?.[0];
    throw new ApiError(res.status, e?.detail ?? e?.title ?? j?.message ?? `Request failed (${res.status})`);
  }
  return json as T;
}

export const verifyLogin = (email: string, token: string) =>
  request('GET', '/spaces', { query: { limit: 1 }, auth: `Basic ${btoa(`${email}:${token}`)}` });

export const listSpaces = (status: 'current' | 'archived' = 'current') => request<Paged<Space>>('GET', '/spaces', { query: { status, limit: 250 } }).then((r) => r.results);
export const getSpaceByKey = async (key: string) => {
  const s = (await request<Paged<Space>>('GET', '/spaces', { query: { keys: key } })).results[0];
  if (!s) throw new ApiError(404, 'Space not found.');
  return s;
};
export const createSpace = (name: string, key: string) => request<Space>('POST', '/spaces', { body: { name, key } });

export const rootPages = (spaceId: string) => request<Paged<ChildPage>>('GET', `/spaces/${spaceId}/pages`, { query: { depth: 'root', limit: 250 } }).then((r) => r.results);
export const childPages = (id: string) => request<Paged<ChildPage>>('GET', `/pages/${id}/direct-children`, { query: { limit: 250 } }).then((r) => r.results);
export const recentPages = () => request<Paged<Page>>('GET', '/pages', { query: { sort: '-modified-date', limit: 12 } }).then((r) => r.results);

export const getPage = (id: string, draft = false) => request<Page>('GET', `/pages/${id}`, { query: { 'body-format': 'atlas_doc_format', 'get-draft': draft } });
export async function getAncestors(id: string): Promise<{ id: string; title: string }[]> {
  const ids = (await request<Paged<{ id: string }>>('GET', `/pages/${id}/ancestors`)).results.map((a) => a.id);
  if (!ids.length) return [];
  const pages = (await request<Paged<Page>>('GET', '/pages', { query: { id: ids, limit: 250 } })).results;
  return ids.flatMap((i) => pages.filter((p) => p.id === i).map((p) => ({ id: p.id, title: p.title })));
}

const adfBody = (value: string) => ({ representation: 'atlas_doc_format', value });
export const createDraft = (spaceId: string, parentId: string | undefined, value: string) =>
  request<Page>('POST', '/pages', { body: { spaceId, parentId, status: 'draft', title: '', body: adfBody(value) } });
export const savePage = (p: { id: string; status: 'draft' | 'current'; title: string; body: string; version: number; message?: string }) =>
  request<Page>('PUT', `/pages/${p.id}`, { body: { id: p.id, status: p.status, title: p.title, body: adfBody(p.body), version: { number: p.version, message: p.message } } });
export const deletePage = (id: string, draft = false) => request<void>('DELETE', `/pages/${id}`, { query: { draft } });
