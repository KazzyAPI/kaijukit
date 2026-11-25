import { Avatar, Icon, IconButton, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { errorTitle } from './api';
import { useMyId } from './users';
import { addRestriction, clearRestrictions, getRestrictions, removeRestriction, type Op, type Subject } from './permApi';
import { SubjectPicker } from './SubjectPicker';

type Level = 'open' | 'edit' | 'view';
const LEVELS: { id: Level; title: string; hint: string }[] = [
  { id: 'open', title: 'No restrictions', hint: 'Anyone who can see this space can view and edit.' },
  { id: 'edit', title: 'Editing restricted', hint: 'Anyone who can see this space can view; only the people and groups below can edit.' },
  { id: 'view', title: 'Viewing and editing restricted', hint: 'Only the people and groups below can open this page.' },
];
const OPS: { op: Op; title: string }[] = [
  { op: 'read', title: 'Who can view' },
  { op: 'update', title: 'Who can edit' },
];

/** Page restrictions editor: pick a level, then manage who is allowed to view and to edit. Writes through the v1 content-restriction operations. */
export function RestrictionsEditor({ pageId }: { pageId: string }) {
  const qc = useQueryClient();
  const toast = useToast();
  const me = useMyId();
  const key = ['restrictions', pageId];
  const data = useQuery({ queryKey: key, queryFn: () => getRestrictions(pageId), retry: false });
  const refresh = () => Promise.all([qc.invalidateQueries({ queryKey: key }), qc.invalidateQueries({ queryKey: ['restriction-state', pageId] })]);
  const fail = (title: string) => (e: Error) => toast({ title: errorTitle(e, title), description: e.message, appearance: 'error' });
  const level: Level = data.data?.read.length ? 'view' : data.data?.update.length ? 'edit' : 'open';

  const setLevel = useMutation({
    mutationFn: async (to: Level) => {
      if (to === 'open') return void (await clearRestrictions(pageId));
      // The editor keeps access: adding yourself first avoids locking yourself out (the server refuses a lockout).
      const self = { kind: 'user' as const, id: me ?? '' };
      if (to === 'view') {
        await addRestriction(pageId, 'update', self);
        await addRestriction(pageId, 'read', self);
      } else {
        await clearRestrictions(pageId);
        await addRestriction(pageId, 'update', self);
      }
    },
    onSuccess: refresh,
    onError: fail("Couldn't change restrictions"),
  });
  const add = useMutation({ mutationFn: (v: { op: Op; s: Subject }) => addRestriction(pageId, v.op, v.s), onSuccess: refresh, onError: fail("Couldn't add") });
  const remove = useMutation({ mutationFn: (v: { op: Op; s: Subject }) => removeRestriction(pageId, v.op, v.s), onSuccess: refresh, onError: fail("Couldn't remove") });

  if (data.isPending) return <Skeleton height={80} />;
  if (data.isError) return <p role="alert" className="kk-error-box">{data.error.message}</p>;
  const busy = setLevel.isPending;
  return (
    <section className="kk-restrict" aria-label="Restrictions">
      <fieldset className="kk-restrict__levels" disabled={busy}>
        <legend>Restrictions</legend>
        {LEVELS.map((l) => (
          <label key={l.id} className="kk-restrict__level">
            <input type="radio" name={`level-${pageId}`} checked={level === l.id} onChange={() => setLevel.mutate(l.id)} />
            <span>
              <strong>{l.title}</strong>
              <span className="kk-muted"> {l.hint}</span>
            </span>
          </label>
        ))}
      </fieldset>
      {level !== 'open' &&
        OPS.filter((o) => level === 'view' || o.op === 'update').map(({ op, title }) => {
          const list = data.data[op];
          return (
            <div key={op} className="kk-restrict__op">
              <h3>{title}</h3>
              <ul className="kk-restrict__list" aria-label={title}>
                {list.map((s) => (
                  <li key={`${s.kind}:${s.id}`}>
                    {s.kind === 'group' ? <Icon name="users" size={20} /> : <Avatar name={s.name} size="sm" />}
                    <span className="kk-restrict__name">{s.name}</span>
                    <span className="kk-muted">{s.kind === 'group' ? 'Group' : s.id === me ? 'You' : 'User'}</span>
                    <IconButton label={`Remove ${s.name} from ${title.toLowerCase()}`} icon={<Icon name="close" size={14} />} onClick={() => remove.mutate({ op, s })} />
                  </li>
                ))}
                {list.length === 0 && <li className="kk-muted">Nobody yet.</li>}
              </ul>
              <SubjectPicker label={`Add to ${title.toLowerCase()}`} exclude={list} onPick={(s) => add.mutate({ op, s })} />
            </div>
          );
        })}
    </section>
  );
}
