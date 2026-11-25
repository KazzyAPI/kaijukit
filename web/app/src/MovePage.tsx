import { Button, Dialog, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { errorTitle } from './api';
import { searchPages } from './contentApi';
import { useDebounced } from './hooks';
import { movePage, type MovePosition } from './permApi';

export function useMove() {
  const qc = useQueryClient();
  const toast = useToast();
  return useMutation({
    mutationFn: (v: { id: string; position: MovePosition; target: string }) => movePage(v.id, v.position, v.target),
    onSuccess: () => {
      for (const k of ['tree', 'ancestors', 'page']) void qc.invalidateQueries({ queryKey: [k] });
      toast({ title: 'Page moved', appearance: 'success' });
    },
    onError: (e) => toast({ title: errorTitle(e, "Couldn't move the page"), description: e.message, appearance: 'error' }),
  });
}

const POSITIONS: { id: MovePosition; label: string }[] = [
  { id: 'append', label: 'Inside it, as the last child' },
  { id: 'before', label: 'Before it, as a sibling' },
  { id: 'after', label: 'After it, as a sibling' },
];

/** Move dialog: search for the destination page by title or text, choose where the page goes relative to it. */
export function MoveDialog({ page, onClose }: { page: { id: string; title: string } | null; onClose: () => void }) {
  const [text, setText] = useState('');
  const [target, setTarget] = useState('');
  const [position, setPosition] = useState<MovePosition>('append');
  const q = useDebounced(text.trim());
  const move = useMove();
  const hits = useQuery({ queryKey: ['search', q, 10], queryFn: () => searchPages(q, 10), enabled: !!page && !!q });
  const rows = (hits.data ?? []).filter((h) => h.content.id !== page?.id);
  const close = () => {
    setText('');
    setTarget('');
    setPosition('append');
    onClose();
  };
  return (
    <Dialog
      open={!!page}
      onClose={close}
      title={`Move "${page?.title || 'Untitled'}"`}
      onSubmit={() => page && target && move.mutate({ id: page.id, position, target }, { onSuccess: close })}
      footer={
        <>
          <Button onClick={close}>Cancel</Button>
          <Button type="submit" appearance="primary" disabled={!target} loading={move.isPending}>Move</Button>
        </>
      }
    >
      <input className="kk-input" aria-label="Find the destination page" placeholder="Find the destination page" autoFocus value={text} onChange={(e) => { setText(e.target.value); setTarget(''); }} />
      <div aria-live="polite">
        {q && hits.isPending && <Skeleton height={32} />}
        {q && hits.isError && <p role="alert" className="kk-field__error">Search failed.</p>}
        {q && hits.data && rows.length === 0 && <p className="kk-muted">No other page matches "{q}".</p>}
        {rows.length > 0 && (
          <fieldset className="kk-restrict__levels">
            <legend>Destination</legend>
            {rows.map((h) => (
              <label key={h.content.id} className="kk-restrict__level">
                <input type="radio" name="move-target" checked={target === h.content.id} onChange={() => setTarget(h.content.id)} />
                <span>{h.title} <span className="kk-muted">in {h.content.space.name}</span></span>
              </label>
            ))}
          </fieldset>
        )}
      </div>
      <fieldset className="kk-restrict__levels" disabled={!target}>
        <legend>Place the page</legend>
        {POSITIONS.map((p) => (
          <label key={p.id} className="kk-restrict__level">
            <input type="radio" name="move-position" checked={position === p.id} onChange={() => setPosition(p.id)} />
            <span>{p.label}</span>
          </label>
        ))}
      </fieldset>
    </Dialog>
  );
}
