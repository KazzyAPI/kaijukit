import { request, type Paged, type Page, type Version } from './api';

const V1 = '/wiki/rest/api';
const ADF = 'atlas_doc_format';
const adfBody = (value: string) => ({ representation: ADF, value });

export type CommentKind = 'footer' | 'inline';
export type Resolution = 'open' | 'resolved' | 'reopened' | 'dangling';
export interface Comment {
  id: string;
  pageId?: string;
  parentCommentId?: string;
  version: Version;
  body?: { atlas_doc_format?: { value: string } };
  resolutionStatus?: Resolution;
  properties?: { inlineMarkerRef?: string; inlineOriginalSelection?: string };
}
export interface InlineAnchor {
  textSelection: string;
  textSelectionMatchCount: number;
  textSelectionMatchIndex: number;
}

const list = <T>(path: string, query: Record<string, string | string[] | number> = {}) => request<Paged<T>>('GET', path, { query: { limit: 250, ...query } }).then((r) => r.results);

export const pageComments = (pageId: string, kind: CommentKind) => list<Comment>(`/pages/${pageId}/${kind}-comments`, { 'body-format': ADF, sort: 'created-date' });
export const commentReplies = (kind: CommentKind, id: string) => list<Comment>(`/${kind}-comments/${id}/children`, { 'body-format': ADF, sort: 'created-date' });
export const createComment = (kind: CommentKind, c: { pageId?: string; parentCommentId?: string; body: string; anchor?: InlineAnchor }) =>
  request<Comment>('POST', `/${kind}-comments`, { body: { pageId: c.pageId, parentCommentId: c.parentCommentId, body: adfBody(c.body), inlineCommentProperties: c.anchor } });
export const editComment = (kind: CommentKind, c: Comment, body: string) =>
  request<Comment>('PUT', `/${kind}-comments/${c.id}`, { body: { version: { number: c.version.number + 1 }, body: adfBody(body) } });
export const resolveComment = (id: string, resolved: boolean) => request<Comment>('PUT', `/inline-comments/${id}`, { body: { resolved } });
export const deleteComment = (kind: CommentKind, id: string) => request<void>('DELETE', `/${kind}-comments/${id}`);

export interface Label {
  id: string;
  prefix: string;
  name: string;
}
export const pageLabels = (id: string) => list<Label>(`/pages/${id}/labels`);
export const knownLabels = () => list<Label>('/labels', { limit: 100 });
export const addLabel = (id: string, name: string) => request<unknown>('POST', `${V1}/content/${id}/label`, { body: [{ prefix: 'global', name }] });
export const removeLabel = (id: string, name: string) => request<void>('DELETE', `${V1}/content/${id}/label/${encodeURIComponent(name)}`);

export interface Likes {
  count: number;
  accountIds: string[];
}
export async function pageLikes(id: string): Promise<Likes> {
  const [count, users] = await Promise.all([request<{ count: number }>('GET', `/pages/${id}/likes/count`), list<{ accountId: string }>(`/pages/${id}/likes/users`)]);
  return { count: count.count, accountIds: users.map((u) => u.accountId) };
}
export const setLike = (id: string, liked: boolean) => request<void>(liked ? 'PUT' : 'DELETE', `${V1}/content/${id}/like`);

export interface PageVersion extends Version {
  minorEdit?: boolean;
}
export const pageVersions = (id: string) => list<PageVersion>(`/pages/${id}/versions`, { sort: '-modified-date' });

export interface Attachment {
  id: string;
  title: string;
  mediaType: string;
  fileSize: number;
  comment?: string;
  downloadLink: string;
  version: PageVersion;
}
export const pageAttachments = (id: string) => list<Attachment>(`/pages/${id}/attachments`);
export const deleteAttachment = (id: string) => request<void>('DELETE', `/attachments/${id}`);

export interface Task {
  id: string;
  pageId: string;
  spaceId: string;
  status: 'complete' | 'incomplete';
  body?: { atlas_doc_format?: { value: string } };
  createdBy: string;
  assignedTo?: string;
  dueAt?: string;
  completedAt?: string;
}
export const listTasks = (q: { status: Task['status']; assignedTo?: string; createdBy?: string }) =>
  list<Task>('/tasks', { 'body-format': ADF, status: q.status, ...(q.assignedTo ? { 'assigned-to': q.assignedTo } : {}), ...(q.createdBy ? { 'created-by': q.createdBy } : {}) });
export const setTaskStatus = (id: string, status: Task['status']) => request<Task>('PUT', `/tasks/${id}`, { body: { status } });

export const pagesByIds = (ids: string[]) => (ids.length ? list<Page>('/pages', { id: ids }) : Promise.resolve([]));
export const restoreVersion = async (id: string, v: Page, message: string) => {
  const current = await request<Page>('GET', `/pages/${id}`);
  return request<Page>('PUT', `/pages/${id}`, { body: { id, status: 'current', title: v.title, body: adfBody(v.body?.atlas_doc_format?.value ?? ''), version: { number: current.version.number + 1, message } } });
};
export const searchPages = (title: string) => list<Page>('/pages', { title, 'body-format': ADF });
