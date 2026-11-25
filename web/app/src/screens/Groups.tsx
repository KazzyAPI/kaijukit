import { Avatar, Button, Dialog, EmptyState, Icon, IconButton, Skeleton, TextField, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { errorTitle } from '../api';
import { addMember, createGroup, deleteGroup, groupMembers, listGroups, removeMember, userName, type GroupInfo } from '../permApi';
import { QueryState } from '../QueryState';
import { SubjectPicker } from '../SubjectPicker';

const EVERYONE = 'kk-users';

export function Groups() {
  const qc = useQueryClient();
  const toast = useToast();
  const [selected, setSelected] = useState<string>();
  const [name, setName] = useState('');
  const [error, setError] = useState('');
  const [confirm, setConfirm] = useState<GroupInfo | null>(null);
  const groups = useQuery({ queryKey: ['groups'], queryFn: listGroups });
  const fail = (title: string) => (e: Error) => toast({ title: errorTitle(e, title), description: e.message, appearance: 'error' });
  const create = useMutation({
    mutationFn: () => createGroup(name.trim()),
    onSuccess: (g) => {
      setName('');
      setSelected(g.id);
      void qc.invalidateQueries({ queryKey: ['groups'] });
    },
    onError: (e) => setError(e.message),
  });
  const del = useMutation({
    mutationFn: (g: GroupInfo) => deleteGroup(g.id),
    onSuccess: () => {
      setConfirm(null);
      setSelected(undefined);
      void qc.invalidateQueries({ queryKey: ['groups'] });
    },
    onError: fail("Couldn't delete the group"),
  });
  const current = groups.data?.find((g) => g.id === selected);

  return (
    <div className="kk-container">
      <h1 className="kk-h1">Groups</h1>
      <div className="kk-groups">
        <div>
          <form
            className="kk-groups__new"
            onSubmit={(e) => {
              e.preventDefault();
              setError('');
              if (name.trim()) create.mutate();
            }}
          >
            <TextField label="New group name" value={name} error={error} onChange={(e) => { setName(e.target.value); setError(''); }} />
            <Button type="submit" appearance="primary" loading={create.isPending} disabled={!name.trim()}>Create group</Button>
          </form>
          <QueryState query={groups} skeleton={<Skeleton height={120} />}>
            {(list) => (
              <ul className="kk-groups__list" aria-label="Groups">
                {list.map((g) => (
                  <li key={g.id}>
                    <button type="button" className="kk-groups__row" aria-pressed={g.id === selected} onClick={() => setSelected(g.id)}>
                      <Icon name="users" size={16} /> {g.name}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </QueryState>
        </div>
        <div>
          {current ? <Members group={current} onDelete={() => setConfirm(current)} /> : <EmptyState title="Select a group" description="Pick a group to see and change its members." />}
        </div>
      </div>
      <Dialog
        open={!!confirm}
        onClose={() => setConfirm(null)}
        title="Delete group?"
        footer={
          <>
            <Button onClick={() => setConfirm(null)}>Cancel</Button>
            <Button appearance="danger" loading={del.isPending} onClick={() => confirm && del.mutate(confirm)}>Delete group</Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>"{confirm?.name}" will be deleted, and so will every space permission and page restriction that names it.</p>
      </Dialog>
    </div>
  );
}

function Members({ group, onDelete }: { group: GroupInfo; onDelete: () => void }) {
  const qc = useQueryClient();
  const toast = useToast();
  const everyone = group.name === EVERYONE;
  const key = ['group-members', group.id];
  const members = useQuery({ queryKey: key, queryFn: () => groupMembers(group.id) });
  const refresh = () => qc.invalidateQueries({ queryKey: key });
  const onError = (title: string) => (e: Error) => toast({ title: errorTitle(e, title), description: e.message, appearance: 'error' });
  const add = useMutation({ mutationFn: (id: string) => addMember(group.id, id), onSuccess: refresh, onError: onError("Couldn't add the member") });
  const remove = useMutation({ mutationFn: (id: string) => removeMember(group.id, id), onSuccess: refresh, onError: onError("Couldn't remove the member") });
  return (
    <section aria-labelledby="members-h">
      <div className="kk-row kk-row--between">
        <h2 id="members-h" className="kk-h2">{group.name}</h2>
        {!everyone && <Button appearance="danger" onClick={onDelete}>Delete group</Button>}
      </div>
      {everyone && <p className="kk-muted">Everyone with an account is a member of this built-in group. It can't be edited or deleted.</p>}
      <QueryState query={members} skeleton={<Skeleton height={80} />}>
        {(list) => (
          <ul className="kk-restrict__list" aria-label="Members">
            {list.map((u) => (
              <li key={u.accountId}>
                <Avatar name={userName(u)} size="sm" />
                <span className="kk-restrict__name">{userName(u)}</span>
                {u.email && <span className="kk-muted">{u.email}</span>}
                {!everyone && <IconButton label={`Remove ${userName(u)} from ${group.name}`} icon={<Icon name="close" size={14} />} onClick={() => remove.mutate(u.accountId)} />}
              </li>
            ))}
            {list.length === 0 && <li className="kk-muted">No members yet.</li>}
          </ul>
        )}
      </QueryState>
      {!everyone && <SubjectPicker label="Add a member" kinds={['user']} onPick={(s) => add.mutate(s.id)} />}
    </section>
  );
}
