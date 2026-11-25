import { Dialog, Icon, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { listSpaces, pagesByTitle } from './api';
import { pageUrl, relativeTime } from './format';
import { useDebounced } from './hooks';

export function QuickSearch({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [text, setText] = useState('');
  const q = useDebounced(text.trim());
  const navigate = useNavigate();
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces(), enabled: open });
  const hits = useQuery({ queryKey: ['title-search', 'all', q], queryFn: () => pagesByTitle(q), enabled: open && !!q });
  const keyOf = (id: string) => spaces.data?.find((s) => s.id === id);
  const rows = (hits.data ?? []).flatMap((p) => {
    const s = keyOf(p.spaceId);
    return s ? [{ p, s }] : [];
  });
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
        if (first) {
          navigate(pageUrl(first.s.key, first.p.id, first.p.title));
          close();
        }
      }}
    >
      <label className="kk-qs__field">
        <Icon name="search" />
        <input className="kk-input kk-qs__input" aria-label="Search pages by title" placeholder="Exact page title" autoFocus value={text} onChange={(e) => setText(e.target.value)} />
      </label>
      <div className="kk-qs__results" aria-live="polite">
        {!q && <p className="kk-muted">Type a page's exact title. Partial matching and full-text search isn't built yet.</p>}
        {q && hits.isPending && <Skeleton height={36} />}
        {q && hits.isError && <p role="alert" className="kk-error-box">Search failed. Try again.</p>}
        {q && hits.data && rows.length === 0 && <p className="kk-muted">No page is titled "{q}".</p>}
        {rows.length > 0 && (
          <ul className="kk-list" aria-label="Results">
            {rows.map(({ p, s }) => (
              <li key={p.id} className="kk-list__item">
                <Link to={pageUrl(s.key, p.id, p.title)} onClick={close}>{p.title || 'Untitled'}</Link>
                <span className="kk-muted">{s.name} · {relativeTime(p.version.createdAt)}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </Dialog>
  );
}
