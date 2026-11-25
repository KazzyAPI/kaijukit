import { request, type Paged } from './api';
import { cql } from './contentApi';

const V1 = '/wiki/rest/api';

export type Op = 'read' | 'update';
export interface Subject {
  kind: 'user' | 'group';
  id: string;
  name: string;
}
export interface GroupInfo {
  id: string;
  name: string;
}
export interface UserInfo {
  accountId: string;
  displayName?: string;
  publicName?: string;
  email?: string;
}
export const userName = (u: UserInfo) => u.displayName || u.publicName || u.email || u.accountId;

interface OpRestrictions {
  restrictions?: { user?: { results: UserInfo[] }; group?: { results: GroupInfo[] } };
}
export async function getRestrictions(pageId: string): Promise<Record<Op, Subject[]>> {
  const expand = ['read', 'update'].flatMap((o) => [`${o}.restrictions.user`, `${o}.restrictions.group`]).join(',');
  const r = await request<Record<Op, OpRestrictions>>('GET', `${V1}/content/${pageId}/restriction/byOperation`, { query: { expand } });
  const subjects = (o: OpRestrictions): Subject[] => [
    ...(o.restrictions?.group?.results ?? []).map((g): Subject => ({ kind: 'group', id: g.id, name: g.name })),
    ...(o.restrictions?.user?.results ?? []).map((u): Subject => ({ kind: 'user', id: u.accountId, name: userName(u) })),
  ];
  return { read: subjects(r.read), update: subjects(r.update) };
}

const subjectPath = (pageId: string, op: Op, s: Pick<Subject, 'kind' | 'id'>) =>
  s.kind === 'user' ? `${V1}/content/${pageId}/restriction/byOperation/${op}/user?accountId=${encodeURIComponent(s.id)}` : `${V1}/content/${pageId}/restriction/byOperation/${op}/byGroupId/${encodeURIComponent(s.id)}`;
export const addRestriction = (pageId: string, op: Op, s: Pick<Subject, 'kind' | 'id'>) => request<void>('PUT', subjectPath(pageId, op, s));
export const removeRestriction = (pageId: string, op: Op, s: Pick<Subject, 'kind' | 'id'>) => request<void>('DELETE', subjectPath(pageId, op, s));
export const clearRestrictions = (pageId: string) => request<unknown>('DELETE', `${V1}/content/${pageId}/restriction`);

export const listGroups = () => request<Paged<GroupInfo>>('GET', `${V1}/group`, { query: { limit: 200 } }).then((r) => r.results);
export const createGroup = (name: string) => request<GroupInfo>('POST', `${V1}/group`, { body: { name } });
export const deleteGroup = (id: string) => request<void>('DELETE', `${V1}/group/by-id`, { query: { id } });
export const groupMembers = (id: string) => request<Paged<UserInfo>>('GET', `${V1}/group/${id}/membersByGroupId`, { query: { limit: 200 } }).then((r) => r.results);
export const addMember = (groupId: string, accountId: string) => request<void>('POST', `${V1}/group/userByGroupId`, { query: { groupId }, body: { accountId } });
export const removeMember = (groupId: string, accountId: string) => request<void>('DELETE', `${V1}/group/userByGroupId`, { query: { groupId, accountId } });
export const findUsers = (name: string) =>
  request<{ results: { user: UserInfo }[] }>('GET', `${V1}/search/user`, { query: { cql: `user.fullname ~ ${cql(name)}`, limit: 10 } }).then((r) => r.results.map((x) => x.user));

export interface SpacePermission {
  id: string; // "read/space"
  displayName: string;
}
export interface Grant {
  id: string;
  principal: { type: 'user' | 'group'; id: string };
  operation: { key: string; targetType: string };
}
export const availablePermissions = () => request<Paged<SpacePermission>>('GET', '/space-permissions', { query: { limit: 250 } }).then((r) => r.results);
export const spaceGrants = (spaceId: string) => request<Paged<Grant>>('GET', `/spaces/${spaceId}/permissions`, { query: { limit: 250 } }).then((r) => r.results);
export const grant = (key: string, s: Pick<Subject, 'kind' | 'id'>, perm: string) => {
  const [op, target] = perm.split('/');
  return request<{ id: number }>('POST', `${V1}/space/${key}/permission`, { body: { subject: { type: s.kind, identifier: s.id }, operation: { key: op, target } } });
};
export const revoke = (key: string, grantId: string) => request<void>('DELETE', `${V1}/space/${key}/permission/${grantId}`);

export const watchStatus = (kind: 'content' | 'space', id: string) => request<{ watching: boolean }>('GET', `${V1}/user/watch/${kind}/${encodeURIComponent(id)}`).then((r) => r.watching);
export const setWatch = (kind: 'content' | 'space', id: string, on: boolean) => request<void>(on ? 'POST' : 'DELETE', `${V1}/user/watch/${kind}/${encodeURIComponent(id)}`);

export type MovePosition = 'before' | 'after' | 'append';
export const movePage = (id: string, position: MovePosition, target: string) => request<unknown>('PUT', `${V1}/content/${id}/move/${position}/${target}`);
