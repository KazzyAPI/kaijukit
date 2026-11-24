import { Avatar, Button, EmptyState, Skeleton, Tabs, TextField } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { listSpaces } from '../api';
import { CreateSpaceDialog } from '../CreateSpaceDialog';
import { QueryState } from '../QueryState';

export function Spaces() {
  const [tab, setTab] = useState<'current' | 'archived'>('current');
  const [filter, setFilter] = useState('');
  const [dialog, setDialog] = useState(false);
  const q = useQuery({ queryKey: ['spaces', tab], queryFn: () => listSpaces(tab) });

  return (
    <div className="kk-container">
      <div className="kk-row kk-row--between">
        <h1 className="kk-h1">Spaces</h1>
        <Button appearance="primary" onClick={() => setDialog(true)}>
          Create space
        </Button>
      </div>
      <Tabs
        label="Space lists"
        value={tab}
        onChange={(t) => setTab(t as typeof tab)}
        tabs={[
          { id: 'current', label: 'All spaces' },
          { id: 'archived', label: 'Archived' },
        ]}
      >
        <div style={{ maxWidth: 320, marginBottom: 16 }}>
          <TextField label="Filter by name" value={filter} onChange={(e) => setFilter(e.target.value)} />
        </div>
        <QueryState
          query={q}
          skeleton={
            <div className="kk-cards">
              {[0, 1, 2].map((i) => (
                <Skeleton key={i} height={96} />
              ))}
            </div>
          }
        >
          {(list) => {
            const shown = list.filter((s) => `${s.name} ${s.key}`.toLowerCase().includes(filter.toLowerCase()));
            if (shown.length === 0)
              return list.length === 0 ? (
                <EmptyState
                  title={tab === 'archived' ? 'No archived spaces' : 'No spaces yet'}
                  description={tab === 'archived' ? undefined : 'Spaces keep related pages together. Create one to get started.'}
                  action={tab === 'current' ? <Button appearance="primary" onClick={() => setDialog(true)}>Create space</Button> : undefined}
                />
              ) : (
                <EmptyState title="No matching spaces" description={`Nothing matches "${filter}".`} />
              );
            return (
              <ul className="kk-cards">
                {shown.map((s) => (
                  <li key={s.id} className="kk-card">
                    <Avatar name={s.name} shape="square" size="lg" />
                    <div className="kk-card__text">
                      <Link to={`/wiki/spaces/${s.key}/overview`} className="kk-card__title">
                        {s.name}
                      </Link>
                      <div className="kk-muted">{s.key}</div>
                      {s.description?.plain?.value && <div className="kk-card__desc">{s.description.plain.value}</div>}
                    </div>
                  </li>
                ))}
              </ul>
            );
          }}
        </QueryState>
      </Tabs>
      <CreateSpaceDialog open={dialog} onClose={() => setDialog(false)} />
    </div>
  );
}
