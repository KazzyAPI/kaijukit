import { parseAdf } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { Button, EmptyState, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { listSpaces } from '../api';
import { listTasks, pagesByIds, setTaskStatus, type Task } from '../contentApi';
import { pageUrl, shortDate } from '../format';
import { QueryState } from '../QueryState';
import { useMyId, useUserNames } from '../users';

type Scope = 'all' | 'assigned' | 'created';

export function Tasks() {
  const qc = useQueryClient();
  const toast = useToast();
  const me = useMyId();
  const [scope, setScope] = useState<Scope>('all');
  const [status, setStatus] = useState<Task['status']>('incomplete');
  const key = ['tasks', scope, status, me];
  const tasks = useQuery({
    queryKey: key,
    queryFn: () => listTasks({ status, assignedTo: scope === 'assigned' ? me : undefined, createdBy: scope === 'created' ? me : undefined }),
    enabled: scope === 'all' || !!me,
  });
  const pageIds = [...new Set(tasks.data?.map((t) => t.pageId))];
  const pages = useQuery({ queryKey: ['pages-by-id', pageIds], queryFn: () => pagesByIds(pageIds), enabled: pageIds.length > 0 });
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  const name = useUserNames(tasks.data?.map((t) => t.assignedTo) ?? []);

  const toggle = useMutation({
    mutationFn: (t: Task) => setTaskStatus(t.id, t.status === 'complete' ? 'incomplete' : 'complete'),
    onMutate: async (t) => {
      await qc.cancelQueries({ queryKey: key });
      const before = qc.getQueryData<Task[]>(key);
      qc.setQueryData<Task[]>(key, (d) => d?.filter((x) => x.id !== t.id));
      return { before };
    },
    onSuccess: (t) => toast({ title: t.status === 'complete' ? 'Task completed' : 'Task reopened', appearance: 'success' }),
    onError: (e, _t, ctx) => {
      qc.setQueryData(key, ctx?.before);
      toast({ title: "Couldn't update the task", description: e.message, appearance: 'error' });
    },
    onSettled: () => Promise.all([qc.invalidateQueries({ queryKey: ['tasks'] }), qc.invalidateQueries({ queryKey: ['page'] }), qc.invalidateQueries({ queryKey: ['versions'] })]),
  });

  const groups = new Map<string, Task[]>();
  tasks.data?.forEach((t) => groups.set(t.pageId, [...(groups.get(t.pageId) ?? []), t]));

  return (
    <div className="kk-container">
      <h1 className="kk-h1">Tasks</h1>
      <div className="kk-row kk-filters">
        <div className="kk-segment" role="group" aria-label="Whose tasks">
          {([['all', 'All tasks'], ['assigned', 'Assigned to me'], ['created', 'Created by me']] as const).map(([v, l]) => (
            <Button key={v} aria-pressed={scope === v} onClick={() => setScope(v)}>{l}</Button>
          ))}
        </div>
        <div className="kk-segment" role="group" aria-label="Status">
          {([['incomplete', 'Incomplete'], ['complete', 'Complete']] as const).map(([v, l]) => (
            <Button key={v} aria-pressed={status === v} onClick={() => setStatus(v)}>{l}</Button>
          ))}
        </div>
      </div>
      {scope !== 'all' && !me && <p role="alert" className="kk-error-box">Couldn't work out which account you are, so this view can't be filtered.</p>}
      <QueryState query={tasks} skeleton={<><Skeleton height={20} width="30%" /><Skeleton height={36} style={{ marginTop: 8 }} /></>}>
        {(list) =>
          list.length === 0 ? (
            <EmptyState title={status === 'incomplete' ? 'No open tasks' : 'No completed tasks'} description="Tasks come from action items in pages. Type [] in a page to add one." />
          ) : (
            <div className="kk-taskgroups">
              {[...groups].map(([pageId, items]) => {
                const page = pages.data?.find((p) => p.id === pageId);
                const space = spaces.data?.find((s) => s.id === items[0]!.spaceId);
                return (
                  <section key={pageId} aria-label={page?.title || 'Page'} className="kk-taskgroup">
                    <h2 className="kk-h2">
                      {page && space ? <Link to={pageUrl(space.key, pageId, page.title)}>{page.title || 'Untitled'}</Link> : (page?.title ?? 'Page')}
                      {space && <span className="kk-muted"> in {space.name}</span>}
                    </h2>
                    <ul className="kk-tasks">
                      {items.map((t) => (
                        <li key={t.id} className="kk-taskrow">
                          <input type="checkbox" checked={t.status === 'complete'} aria-labelledby={`task-${t.id}`} onChange={() => toggle.mutate(t)} />
                          <div id={`task-${t.id}`} className="kk-taskrow__body"><AdfView doc={parseAdf(t.body?.atlas_doc_format?.value)} /></div>
                          <span className="kk-muted kk-taskrow__meta">
                            {t.assignedTo && <>For {name(t.assignedTo)}</>}
                            {t.dueAt && <> · Due {shortDate(t.dueAt)}</>}
                          </span>
                        </li>
                      ))}
                    </ul>
                  </section>
                );
              })}
            </div>
          )
        }
      </QueryState>
    </div>
  );
}
