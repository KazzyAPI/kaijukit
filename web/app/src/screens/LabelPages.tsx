import { EmptyState, Icon, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { Link, useParams, useSearchParams } from 'react-router-dom';
import { listSpaces } from '../api';
import { labelPages } from '../contentApi';
import { ContentIcon } from '../ContentIcon';
import { exactTime, pageUrl, relativeTime } from '../format';
import { QueryState } from '../QueryState';

export function LabelPages() {
  const { id } = useParams();
  const name = useSearchParams()[0].get('name');
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  const pages = useQuery({ queryKey: ['label-pages', id], queryFn: () => labelPages(id!) });

  return (
    <div className="kk-container">
      <h1 className="kk-h1"><Icon name="tag" /> {name ? `Label: ${name}` : 'Label'}</h1>
      <QueryState query={pages} skeleton={<Skeleton height={56} style={{ marginBottom: 8 }} />}>
        {(list) =>
          list.length === 0 ? (
            <EmptyState title="No pages have this label" description="Pages you can't view aren't listed." />
          ) : (
            <>
              <p className="kk-muted" role="status">{list.length} {list.length === 1 ? 'page' : 'pages'}</p>
              <ul className="kk-results" aria-label="Pages with this label">
                {list.map((p) => {
                  const s = spaces.data?.find((x) => x.id === p.spaceId);
                  return (
                    <li key={p.id} className="kk-result">
                      <ContentIcon />
                      <div className="kk-result__text">
                        {s ? <Link className="kk-result__title" to={pageUrl(s.key, p.id, p.title)}>{p.title || 'Untitled'}</Link> : <span className="kk-result__title">{p.title || 'Untitled'}</span>}
                        <span className="kk-muted">{s?.name} · <time dateTime={p.version.createdAt} title={exactTime(p.version.createdAt)}>{relativeTime(p.version.createdAt)}</time></span>
                      </div>
                    </li>
                  );
                })}
              </ul>
            </>
          )
        }
      </QueryState>
    </div>
  );
}
