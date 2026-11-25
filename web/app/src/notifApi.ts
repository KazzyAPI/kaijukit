import { request } from './api';

// kaijukit extension (not in the pinned API specs): everything under /wiki/rest/kk/v1.
const KK = '/wiki/rest/kk/v1';

export interface Notification {
  id: string;
  kind: 'content.created' | 'content.updated';
  read: boolean;
  createdAt: string;
  actor: { accountId: string; displayName?: string } | null;
  content: { id: string; type: string; title: string; spaceKey: string };
}

export const listNotifications = (limit = 25, start = 0) => request<{ results: Notification[] }>('GET', `${KK}/notifications`, { query: { limit, start } }).then((r) => r.results);
export const unreadCount = () => request<{ count: number }>('GET', `${KK}/notifications/unread-count`).then((r) => r.count);
export const markRead = (id: string) => request<void>('POST', `${KK}/notifications/${id}/read`);
export const markAllRead = () => request<void>('POST', `${KK}/notifications/read-all`);
