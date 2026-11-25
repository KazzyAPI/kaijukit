import { Button, Icon, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useId, useState } from 'react';
import { Link } from 'react-router-dom';
import { addLabel, knownLabels, pageLabels, removeLabel } from './contentApi';

export function Labels({ pageId }: { pageId: string }) {
  const qc = useQueryClient();
  const toast = useToast();
  const listId = useId();
  const key = ['labels', pageId];
  const labels = useQuery({ queryKey: key, queryFn: () => pageLabels(pageId) });
  const [adding, setAdding] = useState(false);
  const [text, setText] = useState('');
  const [error, setError] = useState('');
  const known = useQuery({ queryKey: ['labels', 'all'], queryFn: knownLabels, enabled: adding, retry: false });
  const done = () => qc.invalidateQueries({ queryKey: ['labels'] });
  const add = useMutation({
    mutationFn: (name: string) => addLabel(pageId, name),
    onSuccess: done,
    onError: (e) => setError(e.message),
  });
  const remove = useMutation({
    mutationFn: (name: string) => removeLabel(pageId, name),
    onSuccess: done,
    onError: (e) => toast({ title: "Couldn't remove the label", description: e.message, appearance: 'error' }),
  });
  const names = labels.data?.map((l) => l.name) ?? [];

  const commit = (value: string) => {
    const name = value.trim().toLowerCase();
    if (!name) return;
    if (/\s/.test(name)) return setError("Labels can't contain spaces. Use a dash instead.");
    setError('');
    setText('');
    if (!names.includes(name)) add.mutate(name);
  };

  return (
    <section className="kk-labels" aria-labelledby="labels-h">
      <h2 id="labels-h" className="kk-labels__title">Labels</h2>
      {labels.isPending && <Skeleton width={120} height={24} />}
      {labels.isError && <span role="alert" className="kk-muted">Couldn't load labels.</span>}
      {labels.data && (
        <ul className="kk-chips" aria-label="Labels">
          {labels.data.map((l) => (
            <li key={l.id} className="kk-chip">
              <Link className="kk-chip__link" to={`/wiki/labels/${l.id}?name=${encodeURIComponent(l.name)}`}>{l.name}</Link>
              <button type="button" className="kk-chip__x" aria-label={`Remove label ${l.name}`} onClick={() => remove.mutate(l.name)}>
                <Icon name="close" size={12} />
              </button>
            </li>
          ))}
        </ul>
      )}
      {labels.data?.length === 0 && !adding && <span className="kk-muted">No labels</span>}
      {labels.data &&
        (adding ? (
          <div className="kk-labels__add">
            <input
              className="kk-input"
              aria-label="New label"
              aria-invalid={error ? true : undefined}
              aria-describedby={error ? `${listId}-e` : undefined}
              list={listId}
              autoFocus
              placeholder="Add a label, then press Enter"
              value={text}
              onChange={(e) => {
                setError('');
                const v = e.target.value;
                if (v.endsWith(',')) commit(v.slice(0, -1));
                else setText(v);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  commit(text);
                } else if (e.key === 'Escape') {
                  setAdding(false);
                  setText('');
                  setError('');
                }
              }}
            />
            <datalist id={listId}>
              {known.data?.filter((l) => !names.includes(l.name)).map((l) => <option key={l.id} value={l.name} />)}
            </datalist>
            <Button onClick={() => setAdding(false)}>Done</Button>
            {error && <span id={`${listId}-e`} role="alert" className="kk-field__error">{error}</span>}
          </div>
        ) : (
          <Button appearance="subtle" onClick={() => setAdding(true)}>
            <Icon name="plus" size={14} /> Add label
          </Button>
        ))}
    </section>
  );
}
