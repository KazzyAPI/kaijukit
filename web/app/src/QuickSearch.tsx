import { Dialog, Icon, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { searchPages } from './contentApi';
import { pageUrl, relativeTime } from './format';
import { useDebounced } from './hooks';

export function QuickSearch({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [text, setText] = useState('');
  const q = useDebounced(text.trim());
  const navigate = useNavigate();
  const hits = useQuery({ queryKey: ['search', q, 8], queryFn: () => searchPages(q, 8), enabled: open && !!q });
  const rows = hits.data ?? [];
  const close = () => {
    setText('');
    onClose();
  };

  return (
    <Dialog
      open={open}
      onClose={close}
      title="Search"
      onSubmit={() => {
        const first = rows[0];
        if (first) navigate(pageUrl(first.content.space.key, first.content.id, first.title));
        else if (q) navigate(`/wiki/search?text=${encodeURIComponent(q)}`);
        else return;
        close();
      }}
    >
      <label className="kk-qs__field">
        <Icon name="search" />
        <input className="kk-input kk-qs__input" aria-label="Search pages" placeholder="Search pages" autoFocus value={text} onChange={(e) => setText(e.target.value)} />
      </label>
      <div className="kk-qs__results" aria-live="polite">
        {!q && <p className="kk-muted">Search page titles and text.</p>}
        {q && hits.isPending && <Skeleton height={36} />}
        {q && hits.isError && <p role="alert" className="kk-error-box">Search failed. Try again.</p>}
        {q && hits.data && rows.length === 0 && <p className="kk-muted">No pages match "{q}".</p>}
        {q && (
          <Link className="kk-qs__all" to={`/wiki/search?text=${encodeURIComponent(q)}`} onClick={close}>
            Search for "{q}" on the results page
          </Link>
        )}
        {rows.length > 0 && (
          <ul className="kk-list" aria-label="Results">
            {rows.map((h) => (
              <li key={h.content.id} className="kk-list__item">
                <Link to={pageUrl(h.content.space.key, h.content.id, h.title)} onClick={close}>{h.title || 'Untitled'}</Link>
                <span className="kk-muted">{h.content.space.name} · {relativeTime(h.lastModified)}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </Dialog>
  );
}
