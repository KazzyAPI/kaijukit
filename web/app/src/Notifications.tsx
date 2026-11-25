import { Icon, IconButton, Menu, MenuItem, MenuLabel, MenuSeparator, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { errorTitle } from './api';
import { listNotifications, markAllRead, markRead, unreadCount, type Notification } from './notifApi';
import { pageUrl, relativeTime, routes } from './format';

/** What happened, in words: "Ada edited Roadmap". The server sends the kind, actor and title only. */
export const describe = (n: Notification) => `${n.actor?.displayName ?? 'Someone'} ${n.kind === 'content.created' ? 'created' : 'updated'} ${n.content.title || 'Untitled'}`;
export const notificationUrl = (n: Notification) => (n.content.type === 'blogpost' ? routes('blogpost', n.content.spaceKey).view(n.content.id) : pageUrl(n.content.spaceKey, n.content.id, n.content.title));

export function useNotifications() {
  const qc = useQueryClient();
  const toast = useToast();
  const refresh = () => Promise.all([qc.invalidateQueries({ queryKey: ['notifications'] }), qc.invalidateQueries({ queryKey: ['notifications-unread'] })]);
  const onError = (e: Error) => toast({ title: errorTitle(e, "Couldn't update notifications"), description: e.message, appearance: 'error' });
  return {
    read: useMutation({ mutationFn: markRead, onSuccess: refresh, onError }),
    readAll: useMutation({ mutationFn: markAllRead, onSuccess: refresh, onError }),
  };
}

export function Bell() {
  const navigate = useNavigate();
  const { read, readAll } = useNotifications();
  const count = useQuery({ queryKey: ['notifications-unread'], queryFn: unreadCount, refetchInterval: 30_000, retry: false });
  const list = useQuery({ queryKey: ['notifications', 'menu'], queryFn: () => listNotifications(6), retry: false });
  const unread = count.data ?? 0;
  const open = (n: Notification) => {
    if (!n.read) read.mutate(n.id);
    navigate(notificationUrl(n));
  };
  return (
    <Menu
      label="Notifications"
      align="end"
      renderTrigger={(p) => (
        <IconButton
          label={unread ? `Notifications, ${unread} unread` : 'Notifications'}
          icon={
            <span className="kk-badged">
              <Icon name="bell" />
              {unread > 0 && <span className="kk-badge" aria-hidden="true">{unread > 9 ? '9+' : unread}</span>}
            </span>
          }
          {...p}
          onClick={() => {
            p.onClick();
            void list.refetch();
          }}
        />
      )}
    >
      <MenuLabel>Notifications</MenuLabel>
      {list.isError && <MenuItem onSelect={() => undefined} disabled>Couldn't load notifications</MenuItem>}
      {list.data?.length === 0 && <MenuItem onSelect={() => undefined} disabled>You're all caught up</MenuItem>}
      {list.data?.map((n) => (
        <MenuItem key={n.id} onSelect={() => open(n)}>
          <span className="kk-notif" data-unread={!n.read}>
            <span className="kk-notif__text">{describe(n)}</span>
            <span className="kk-muted">{n.content.spaceKey} · {relativeTime(n.createdAt)}</span>
          </span>
        </MenuItem>
      ))}
      <MenuSeparator />
      {unread > 0 && <MenuItem onSelect={() => readAll.mutate()}>Mark all as read</MenuItem>}
      <MenuItem onSelect={() => navigate('/wiki/notifications')}>View all notifications</MenuItem>
    </Menu>
  );
}
