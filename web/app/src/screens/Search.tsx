import { Button, EmptyState, Icon, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { searchPages } from '../contentApi';
import { ContentIcon } from '../ContentIcon';
import { exactTime, pageUrl, relativeTime } from '../format';
import { QueryState } from '../QueryState';

const plain = (t: string) => t.replace(/@@@(end)?hl@@@/g, '');

export function Search() {
  const [params, setParams] = useSearchParams();
  const text = params.get('text')?.trim() ?? '';
  const [draft, setDraft] = useState(text);
  const hits = useQuery({ queryKey: ['search', text, 25], queryFn: () => searchPages(text), enabled: !!text });

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
          <input className="kk-input" aria-label="Search pages" placeholder="Search page titles and text" value={draft} onChange={(e) => setDraft(e.target.value)} />
        </label>
        <Button type="submit" appearance="primary">Search</Button>
      </form>
      <p className="kk-muted">Matches page titles (partial) and page text. Filters, sorting and advanced CQL aren't built yet.</p>
      {!text && <EmptyState title="Search for a page" description="Type part of a title or words from the page." />}
      {text && (
        <QueryState query={hits} skeleton={<Skeleton height={72} style={{ marginBottom: 8 }} />}>
          {(pages) =>
            pages.length === 0 ? (
              <EmptyState title={`No pages match "${text}"`} description="Check the spelling or try fewer words." />
            ) : (
              <>
                <p className="kk-muted" role="status">{pages.length} {pages.length === 1 ? 'result' : 'results'} for "{text}"</p>
                <ul className="kk-results" aria-label="Search results">
                  {pages.map((h) => {
                    const ex = plain(h.excerpt);
                    return (
                      <li key={h.content.id} className="kk-result">
                        <ContentIcon />
                        <div className="kk-result__text">
                          <Link className="kk-result__title" to={pageUrl(h.content.space.key, h.content.id, h.title)}>{h.title || 'Untitled'}</Link>
                          <span className="kk-muted">{h.content.space.name} · <time dateTime={h.lastModified} title={exactTime(h.lastModified)}>{relativeTime(h.lastModified)}</time></span>
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
