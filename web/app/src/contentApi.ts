import { request, SEG, type Kind, type Paged, type Page, type Version } from './api';

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
  authorId?: string; // original author (kaijukit extension); version.authorId is the last editor
  resolutionStatus?: Resolution;
  properties?: { inlineMarkerRef?: string; inlineOriginalSelection?: string };
}
export interface InlineAnchor {
  textSelection: string;
  textSelectionMatchCount: number;
  textSelectionMatchIndex: number;
}

const list = <T>(path: string, query: Record<string, string | string[] | number> = {}) => request<Paged<T>>('GET', path, { query: { limit: 250, ...query } }).then((r) => r.results);

export const pageComments = (pageId: string, kind: CommentKind, of: Kind = 'page') => list<Comment>(`/${SEG[of]}/${pageId}/${kind}-comments`, { 'body-format': ADF, sort: 'created-date' });
export const commentReplies = (kind: CommentKind, id: string) => list<Comment>(`/${kind}-comments/${id}/children`, { 'body-format': ADF, sort: 'created-date' });
export const createComment = (kind: CommentKind, c: { pageId?: string; of?: Kind; parentCommentId?: string; body: string; anchor?: InlineAnchor }) =>
  request<Comment>('POST', `/${kind}-comments`, { body: { ...(c.of === 'blogpost' ? { blogPostId: c.pageId } : { pageId: c.pageId }), parentCommentId: c.parentCommentId, body: adfBody(c.body), inlineCommentProperties: c.anchor } });
export const editComment = (kind: CommentKind, c: Comment, body: string) =>
  request<Comment>('PUT', `/${kind}-comments/${c.id}`, { body: { version: { number: c.version.number + 1 }, body: adfBody(body) } });
export const resolveComment = (id: string, resolved: boolean) => request<Comment>('PUT', `/inline-comments/${id}`, { body: { resolved } });
export const deleteComment = (kind: CommentKind, id: string) => request<void>('DELETE', `/${kind}-comments/${id}`);

export interface Label {
  id: string;
  prefix: string;
  name: string;
}
export const pageLabels = (id: string, of: Kind = 'page') => list<Label>(`/${SEG[of]}/${id}/labels`);
export const knownLabels = () => list<Label>('/labels', { limit: 100 });
export const addLabel = (id: string, name: string) => request<unknown>('POST', `${V1}/content/${id}/label`, { body: [{ prefix: 'global', name }] });
export const removeLabel = (id: string, name: string) => request<void>('DELETE', `${V1}/content/${id}/label/${encodeURIComponent(name)}`);

export interface Likes {
  count: number;
  accountIds: string[];
}
export async function pageLikes(id: string, of: Kind = 'page'): Promise<Likes> {
  const [count, users] = await Promise.all([request<{ count: number }>('GET', `/${SEG[of]}/${id}/likes/count`), list<{ accountId: string }>(`/${SEG[of]}/${id}/likes/users`)]);
  return { count: count.count, accountIds: users.map((u) => u.accountId) };
}
export const setLike = (id: string, liked: boolean) => request<void>(liked ? 'PUT' : 'DELETE', `${V1}/content/${id}/like`);

export interface PageVersion extends Version {
  minorEdit?: boolean;
}
export const pageVersions = (id: string, of: Kind = 'page') => list<PageVersion>(`/${SEG[of]}/${id}/versions`, { sort: '-modified-date' });

export interface Attachment {
  id: string;
  title: string;
  mediaType: string;
  fileSize: number;
  comment?: string;
  downloadLink: string;
  version: PageVersion;
}
export const pageAttachments = (id: string, of: Kind = 'page') => list<Attachment>(`/${SEG[of]}/${id}/attachments`);
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
// v1 restore appends a copy of the old version (body and title) for any content type, so no body round trip is needed.
export const restoreVersion = (id: string, number: number, message: string) =>
  request<{ number: number }>('POST', `${V1}/content/${id}/version`, { body: { operationKey: 'restore', params: { versionNumber: number, message, restoreTitle: true } } });
export const cql = (s: string) => `"${s.replace(/[\\"]/g, '\\$&')}"`;
export interface SearchHit {
  title: string;
  excerpt: string;
  lastModified: string;
  content: { id: string; space: { key: string; name: string } };
}
export const searchPages = (text: string, limit = 25) =>
  request<{ results: SearchHit[] }>('GET', `${V1}/search`, { query: { cql: `type = page AND (title ~ ${cql(text)} OR text ~ ${cql(text)})`, limit } }).then((r) => r.results);

export interface Person {
  accountId: string;
  displayName?: string;
}
export const searchPeople = (name: string) =>
  request<{ results: { user: Person }[] }>('GET', `${V1}/search/user`, { query: { cql: `user.fullname ~ ${cql(name)}`, limit: 10 } }).then((r) => r.results.map((x) => x.user));

export const labelPages = (labelId: string) => list<Page>(`/labels/${labelId}/pages`);

export type RestrictionState = 'OPEN' | 'VIEW_RESTRICTED' | 'EDIT_RESTRICTED';
export const restrictionState = (pageId: string, of: Kind = 'page') => request<{ state: RestrictionState }>('GET', `/${SEG[of]}/${pageId}/restriction-state`).then((r) => r.state);

export const spaceBlogs = (spaceId: string) => list<Page>(`/spaces/${spaceId}/blogposts`, { sort: '-created-date', limit: 25 });
export const getBlog = (id: string) => request<Page>('GET', `/blogposts/${id}`, { query: { 'body-format': ADF } });
// Root-level folders are not listed by any endpoint, so folders are created under the space home page.
export const createFolder = (spaceId: string, title: string, parentId?: string) => request<{ id: string }>('POST', '/folders', { body: { spaceId, title, parentId } });
