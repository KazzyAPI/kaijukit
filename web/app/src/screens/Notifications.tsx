import { Button, EmptyState, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { exactTime, relativeTime } from '../format';
import { listNotifications } from '../notifApi';
import { describe, notificationUrl, useNotifications } from '../Notifications';
import { QueryState } from '../QueryState';

const PAGE = 25;

export function NotificationsPage() {
  const [limit, setLimit] = useState(PAGE);
  const { read, readAll } = useNotifications();
  const list = useQuery({ queryKey: ['notifications', 'page', limit], queryFn: () => listNotifications(limit), placeholderData: (p) => p });
  return (
    <div className="kk-container">
      <div className="kk-row kk-row--between">
        <h1 className="kk-h1">Notifications</h1>
        <Button loading={readAll.isPending} onClick={() => readAll.mutate()} disabled={!list.data?.some((n) => !n.read)}>Mark all as read</Button>
      </div>
      <QueryState query={list} skeleton={<Skeleton height={56} style={{ marginBottom: 8 }} />}>
        {(items) =>
          items.length === 0 ? (
            <EmptyState title="You're all caught up" description="When someone creates or updates a page you watch, it shows up here." />
          ) : (
            <>
              <ul className="kk-results" aria-label="Notifications">
                {items.map((n) => (
                  <li key={n.id} className="kk-result kk-notif" data-unread={!n.read}>
                    <div className="kk-result__text">
                      <Link className="kk-result__title" to={notificationUrl(n)} onClick={() => !n.read && read.mutate(n.id)}>{describe(n)}</Link>
                      <span className="kk-muted">
                        {n.content.spaceKey} · <time dateTime={n.createdAt} title={exactTime(n.createdAt)}>{relativeTime(n.createdAt)}</time>
                        {n.read ? '' : ' · Unread'}
                      </span>
                    </div>
                    {!n.read && <Button appearance="subtle" onClick={() => read.mutate(n.id)}>Mark as read</Button>}
                  </li>
                ))}
              </ul>
              {items.length >= limit && <Button onClick={() => setLimit(limit + PAGE)}>Show more</Button>}
            </>
          )
        }
      </QueryState>
    </div>
  );
}
