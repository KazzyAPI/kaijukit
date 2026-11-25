import { Avatar, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { request, type Paged, type Page, type Space } from './api';
import { ContentIcon } from './ContentIcon';
import { exactTime, relativeTime, routes } from './format';
import { useUserNames } from './users';

const recent = (path: string, spaceId: string) => request<Paged<Page>>('GET', path, { query: { 'space-id': spaceId, sort: '-modified-date', limit: 8 } }).then((r) => r.results);

/** Overview section: the latest edits to pages and blog posts of a space, newest first. */
export function SpaceActivity({ space }: { space: Space }) {
  const q = useQuery({
    queryKey: ['space-activity', space.id],
    queryFn: async () => {
      const [pages, blogs] = await Promise.all([recent('/pages', space.id), recent('/blogposts', space.id)]);
      return [...pages.map((p) => ({ p, of: 'page' as const })), ...blogs.map((p) => ({ p, of: 'blogpost' as const }))].sort((a, b) => b.p.version.createdAt.localeCompare(a.p.version.createdAt)).slice(0, 8);
    },
  });
  const name = useUserNames(q.data?.map((x) => x.p.version.authorId) ?? []);
  return (
    <section className="kk-section" aria-labelledby="activity-h">
      <h2 id="activity-h" className="kk-section__title">Recent activity</h2>
      {q.isPending && <Skeleton height={48} />}
      {q.isError && <p role="alert" className="kk-error-box">Couldn't load recent activity.</p>}
      {q.data?.length === 0 && <p className="kk-muted">Nothing has been edited in this space yet.</p>}
      {q.data && q.data.length > 0 && (
        <ul className="kk-activity" aria-label="Recent activity">
          {q.data.map(({ p, of }) => (
            <li key={p.id} className="kk-activity__row">
              <ContentIcon type={of} />
              <Link to={routes(of, space.key).view(p.id, p.title)}>{p.title || 'Untitled'}</Link>
              <span className="kk-muted kk-activity__who">
                <Avatar name={name(p.version.authorId)} size="sm" />
                {name(p.version.authorId)}
              </span>
              <time className="kk-muted" dateTime={p.version.createdAt} title={exactTime(p.version.createdAt)}>{relativeTime(p.version.createdAt)}</time>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
