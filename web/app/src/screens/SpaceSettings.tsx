import { Avatar, Button, EmptyState, Icon, Skeleton, Tabs, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useOutletContext, useSearchParams } from 'react-router-dom';
import { errorTitle, lookupUsers, type Space } from '../api';
import { shortDate } from '../format';
import { availablePermissions, grant, listGroups, revoke, spaceGrants, userName, type Grant, type Subject } from '../permApi';
import { QueryState } from '../QueryState';
import { SubjectPicker } from '../SubjectPicker';
import { TemplateManager } from '../Templates';
import { useWatch } from '../Watch';

type Tab = 'details' | 'permissions' | 'templates';

export function SpaceSettings() {
  const space = useOutletContext<Space>();
  const [params] = useSearchParams();
  const [tab, setTab] = useState<Tab>(params.get('tab') === 'templates' ? 'templates' : 'details');
  return (
    <div className="kk-container">
      <h1 className="kk-h1">Space settings: {space.name}</h1>
      <Tabs label="Space settings" value={tab} onChange={(t) => setTab(t as Tab)} tabs={[{ id: 'details', label: 'Details' }, { id: 'permissions', label: 'Permissions' }, { id: 'templates', label: 'Templates' }]}>
        {tab === 'details' ? <Details space={space} /> : tab === 'permissions' ? <Permissions space={space} /> : <TemplateManager spaceKey={space.key} />}
      </Tabs>
    </div>
  );
}

function Details({ space }: { space: Space }) {
  const watch = useWatch('space', space.key);
  const rows: [string, string][] = [
    ['Name', space.name],
    ['Key', space.key],
    ['Status', space.status],
    ['Created', shortDate(space.createdAt)],
    ['Description', space.description?.plain?.value || 'None'],
  ];
  return (
    <div className="kk-settings">
      <dl className="kk-dl">
        {rows.map(([k, v]) => (
          <div key={k}>
            <dt>{k}</dt>
            <dd>{v}</dd>
          </div>
        ))}
      </dl>
      <p className="kk-muted">Editing the name, description and key isn't built yet: the server has no space update endpoint.</p>
      <h2 className="kk-h2">Notifications</h2>
      {watch.ready ? (
        <div>
          <Button aria-pressed={!!watch.watching} loading={watch.pending} onClick={watch.toggle}>
            <Icon name="eye" size={14} /> {watch.watching ? 'Stop watching this space' : 'Watch this space'}
          </Button>
          <p className="kk-muted">Watchers are notified when pages are created or published in the space.</p>
        </div>
      ) : (
        <p className="kk-muted">Watching isn't available on this server.</p>
      )}
      <p>
        <Link to="/wiki/admin/groups">Manage groups</Link>
      </p>
    </div>
  );
}

const groupKey = (s: Pick<Subject, 'kind' | 'id'>) => `${s.kind}:${s.id}`;

function Permissions({ space }: { space: Space }) {
  const qc = useQueryClient();
  const toast = useToast();
  const [extra, setExtra] = useState<Subject[]>([]);
  const perms = useQuery({ queryKey: ['space-perms', 'available'], queryFn: availablePermissions });
  const grants = useQuery({ queryKey: ['space-perms', space.id], queryFn: () => spaceGrants(space.id) });
  const groups = useQuery({ queryKey: ['groups'], queryFn: listGroups });
  const userIds = [...new Set((grants.data ?? []).filter((g) => g.principal.type === 'user').map((g) => g.principal.id))];
  const users = useQuery({ queryKey: ['users', userIds], queryFn: () => lookupUsers(userIds), enabled: userIds.length > 0, retry: false });
  const refresh = () => qc.invalidateQueries({ queryKey: ['space-perms', space.id] });
  const onError = (e: Error) => toast({ title: errorTitle(e, "Couldn't change the permission"), description: e.message, appearance: 'error' });
  const add = useMutation({ mutationFn: (v: { s: Subject; perm: string }) => grant(space.key, v.s, v.perm), onSuccess: refresh, onError });
  const remove = useMutation({ mutationFn: (g: Grant) => revoke(space.key, g.id), onSuccess: refresh, onError });

  const name = (kind: Subject['kind'], id: string) =>
    (kind === 'group' ? groups.data?.find((g) => g.id === id)?.name : (() => { const u = users.data?.find((x) => x.accountId === id); return u && userName(u); })()) ?? `${kind === 'group' ? 'Group' : 'User'} ${id.slice(0, 8)}`;
  const subjects: Subject[] = [];
  for (const g of grants.data ?? []) {
    const s: Subject = { kind: g.principal.type, id: g.principal.id, name: name(g.principal.type, g.principal.id) };
    if (!subjects.some((x) => groupKey(x) === groupKey(s))) subjects.push(s);
  }
  for (const s of extra) if (!subjects.some((x) => groupKey(x) === groupKey(s))) subjects.push(s);
  subjects.sort((a, b) => (a.kind === b.kind ? a.name.localeCompare(b.name) : a.kind === 'group' ? -1 : 1));

  return (
    <QueryState query={perms} skeleton={<Skeleton height={200} />}>
      {(list) => (
        <QueryState query={grants} skeleton={<Skeleton height={200} />}>
          {(gs) => {
            const find = (s: Subject, perm: string) => gs.find((g) => g.principal.type === s.kind && g.principal.id === s.id && `${g.operation.key}/${g.operation.targetType}` === perm);
            return (
              <div className="kk-settings">
                {subjects.length === 0 ? (
                  <EmptyState title="Nobody has been granted anything" description="Add a user or group below." />
                ) : (
                  <div className="kk-grid-scroll" tabIndex={0} role="region" aria-label="Permissions grid, scrolls sideways">
                    <table className="kk-grid">
                      <caption className="kk-sr">Permissions by user and group</caption>
                      <thead>
                        <tr>
                          <th scope="col">User or group</th>
                          {list.map((p) => (
                            <th key={p.id} scope="col" title={p.displayName}>
                              <span>{p.id.split('/')[0]!.replace('_', ' ')}</span>
                              <span className="kk-muted">{p.id.split('/')[1]}</span>
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {subjects.map((s) => (
                          <tr key={groupKey(s)}>
                            <th scope="row">
                              <div className="kk-grid__who">
                                {s.kind === 'group' ? <Icon name="users" size={20} /> : <Avatar name={s.name} size="sm" />}
                                <span>{s.name}</span>
                              </div>
                            </th>
                            {list.map((p) => {
                              const g = find(s, p.id);
                              return (
                                <td key={p.id}>
                                  <input type="checkbox" checked={!!g} aria-label={`${s.name}: ${p.displayName}`} onChange={() => (g ? remove.mutate(g) : add.mutate({ s, perm: p.id }))} />
                                </td>
                              );
                            })}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
                <SubjectPicker label="Add a user or group" exclude={subjects} onPick={(s) => setExtra([...extra, s])} />
                <p className="kk-muted">Changing permissions needs the administer space permission. A space without a grant for "read space" is invisible to that person.</p>
              </div>
            );
          }}
        </QueryState>
      )}
    </QueryState>
  );
}
