import { Button, EmptyState, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { listSpaces, recentPages } from '../api';
import { CreateSpaceDialog } from '../CreateSpaceDialog';
import { exactTime, pageUrl, relativeTime } from '../format';
import { QueryState } from '../QueryState';

export function Home() {
  const recent = useQuery({ queryKey: ['recent'], queryFn: recentPages });
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  const [dialog, setDialog] = useState(false);
  const keyOf = (id: string) => spaces.data?.find((s) => s.id === id);

  return (
    <div className="kk-container">
      <h1 className="kk-h1">Home</h1>
      <div className="kk-home">
        <section aria-labelledby="recent-h">
          <h2 id="recent-h" className="kk-h2">
            Recently updated
          </h2>
          <QueryState query={recent} skeleton={<Skeleton height={56} style={{ marginBottom: 8 }} />}>
            {(pages) =>
              pages.length === 0 ? (
                <EmptyState title="Nothing here yet" description="Pages you and your team create will show up here." action={<Button appearance="primary" onClick={() => setDialog(true)}>Create a space</Button>} />
              ) : (
                <ul className="kk-list">
                  {pages.map((p) => {
                    const s = keyOf(p.spaceId);
                    return (
                      <li key={p.id} className="kk-list__item">
                        {s ? <Link to={pageUrl(s.key, p.id, p.title)}>{p.title || 'Untitled'}</Link> : <span>{p.title || 'Untitled'}</span>}
                        <span className="kk-muted">
                          {s?.name} · <time dateTime={p.version.createdAt} title={exactTime(p.version.createdAt)}>{relativeTime(p.version.createdAt)}</time>
                        </span>
                      </li>
                    );
                  })}
                </ul>
              )
            }
          </QueryState>
        </section>
        <aside aria-labelledby="spaces-h">
          <h2 id="spaces-h" className="kk-h2">
            Spaces
          </h2>
          <QueryState query={spaces} skeleton={<Skeleton height={20} style={{ marginBottom: 8 }} />}>
            {(list) =>
              list.length === 0 ? (
                <p className="kk-muted">No spaces yet.</p>
              ) : (
                <ul className="kk-list">
                  {list.slice(0, 8).map((s) => (
                    <li key={s.id} className="kk-list__item">
                      <Link to={`/wiki/spaces/${s.key}/overview`}>{s.name}</Link>
                    </li>
                  ))}
                  <li className="kk-list__item">
                    <Link to="/wiki/spaces">View all spaces</Link>
                  </li>
                </ul>
              )
            }
          </QueryState>
        </aside>
      </div>
      <CreateSpaceDialog open={dialog} onClose={() => setDialog(false)} />
    </div>
  );
}
