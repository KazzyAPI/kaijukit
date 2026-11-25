import { Avatar, Icon } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useId, useState } from 'react';
import { useDebounced } from './hooks';
import { findUsers, listGroups, userName, type Subject } from './permApi';

/** Type to find users (by name) and groups; picking a result calls onPick. `kinds` limits what is offered. */
export function SubjectPicker({ label, kinds = ['user', 'group'], exclude = [], onPick }: { label: string; kinds?: Subject['kind'][]; exclude?: Subject[]; onPick: (s: Subject) => void }) {
  const [text, setText] = useState('');
  const q = useDebounced(text.trim());
  const id = useId();
  const found = useQuery({
    queryKey: ['subject-search', q, kinds.join()],
    enabled: !!q,
    queryFn: async (): Promise<Subject[]> => {
      const [users, groups] = await Promise.all([
        kinds.includes('user') ? findUsers(q) : [],
        kinds.includes('group') ? listGroups().then((g) => g.filter((x) => x.name.toLowerCase().includes(q.toLowerCase()))) : [],
      ]);
      return [...groups.map((g): Subject => ({ kind: 'group', id: g.id, name: g.name })), ...users.map((u): Subject => ({ kind: 'user', id: u.accountId, name: userName(u) }))];
    },
  });
  const rows = (found.data ?? []).filter((s) => !exclude.some((e) => e.kind === s.kind && e.id === s.id));
  return (
    <div className="kk-picker">
      <input className="kk-input" aria-label={label} aria-describedby={`${id}-r`} placeholder={label} value={text} onChange={(e) => setText(e.target.value)} />
      <div id={`${id}-r`} aria-live="polite">
        {q && found.isError && <p role="alert" className="kk-field__error">Search failed.</p>}
        {q && found.data && rows.length === 0 && <p className="kk-muted kk-picker__none">No matching {kinds.includes('group') ? 'users or groups' : 'users'}.</p>}
        {rows.length > 0 && (
          <ul className="kk-picker__list" aria-label={`${label} results`}>
            {rows.map((s) => (
              <li key={`${s.kind}:${s.id}`}>
                <button
                  type="button"
                  className="kk-picker__row"
                  onClick={() => {
                    onPick(s);
                    setText('');
                  }}
                >
                  {s.kind === 'group' ? <Icon name="users" size={20} /> : <Avatar name={s.name} size="sm" />}
                  <span>{s.name}</span>
                  <span className="kk-muted">{s.kind === 'group' ? 'Group' : 'User'}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
