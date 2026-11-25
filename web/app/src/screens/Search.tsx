import { parseAdf, type AdfNode } from '@kaijukit/editor/adf';
import { Button, EmptyState, Icon, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { listSpaces } from '../api';
import { searchPages } from '../contentApi';
import { ContentIcon } from '../ContentIcon';
import { exactTime, pageUrl, relativeTime } from '../format';
import { QueryState } from '../QueryState';

const textOf = (n: AdfNode): string => n.text ?? (n.content ?? []).map(textOf).join(n.type === 'doc' ? ' ' : '');
const excerpt = (value?: string) => {
  const doc = parseAdf(value);
  const t = doc ? textOf(doc).replace(/\s+/g, ' ').trim() : '';
  return t.length > 180 ? `${t.slice(0, 180)}...` : t;
};

export function Search() {
  const [params, setParams] = useSearchParams();
  const text = params.get('text')?.trim() ?? '';
  const [draft, setDraft] = useState(text);
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  const hits = useQuery({ queryKey: ['search', text], queryFn: () => searchPages(text), enabled: !!text });

  return (
    <div className="kk-container">
      <h1 className="kk-h1">Search</h1>
      <form
        className="kk-row kk-searchbar"
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          setParams(draft.trim() ? { text: draft.trim() } : {});
        }}
      >
        <label className="kk-qs__field kk-searchbar__field">
          <Icon name="search" />
          <input className="kk-input" aria-label="Search pages by title" placeholder="Exact page title" value={draft} onChange={(e) => setDraft(e.target.value)} />
        </label>
        <Button type="submit" appearance="primary">Search</Button>
      </form>
      <p className="kk-muted">Full-text search, filters (space, contributor, label, date), sorting and CQL need a search endpoint that the server doesn't have yet. For now this finds pages by exact title.</p>
      {!text && <EmptyState title="Search for a page" description="Type the exact title of a page." />}
      {text && (
        <QueryState query={hits} skeleton={<Skeleton height={72} style={{ marginBottom: 8 }} />}>
          {(pages) =>
            pages.length === 0 ? (
              <EmptyState title={`No page is titled "${text}"`} description="Check the spelling. The match is on the whole title, ignoring case." />
            ) : (
              <>
                <p className="kk-muted" role="status">{pages.length} {pages.length === 1 ? 'result' : 'results'} for "{text}"</p>
                <ul className="kk-results" aria-label="Search results">
                  {pages.map((p) => {
                    const s = spaces.data?.find((x) => x.id === p.spaceId);
                    const ex = excerpt(p.body?.atlas_doc_format?.value);
                    return (
                      <li key={p.id} className="kk-result">
                        <ContentIcon />
                        <div className="kk-result__text">
                          {s ? <Link className="kk-result__title" to={pageUrl(s.key, p.id, p.title)}>{p.title || 'Untitled'}</Link> : <span className="kk-result__title">{p.title || 'Untitled'}</span>}
                          <span className="kk-muted">{s?.name} · <time dateTime={p.version.createdAt} title={exactTime(p.version.createdAt)}>{relativeTime(p.version.createdAt)}</time></span>
                          {ex && <p className="kk-result__excerpt">{ex}</p>}
                        </div>
                      </li>
                    );
                  })}
                </ul>
              </>
            )
          }
        </QueryState>
      )}
    </div>
  );
}
