import { parseAdf } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { Avatar, Button, Dialog, Lozenge, Skeleton, TextField, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useNavigate, useOutletContext, useParams, useSearchParams } from 'react-router-dom';
import { getPage, resolveMedia, type Kind, type Space } from '../api';
import { pageVersions, restoreVersion } from '../contentApi';
import { exactTime, relativeTime, routes } from '../format';
import { NotBuilt, PageHeaderBar } from '../PageHeader';
import { QueryState } from '../QueryState';
import { useUserNames } from '../users';
import { NotFound } from './Errors';

const PLAIN = { open: new Set<string>(), onSelect: () => undefined };

export function History({ of = 'page' }: { of?: Kind }) {
  const id = useParams().id!;
  const space = useOutletContext<Space>();
  const navigate = useNavigate();
  const go = routes(of, space.key);
  const qc = useQueryClient();
  const toast = useToast();
  const [params, setParams] = useSearchParams();
  const [restoring, setRestoring] = useState(false);
  const [message, setMessage] = useState('');
  const versions = useQuery({ queryKey: ['versions', id], queryFn: () => pageVersions(id, of) });
  const latest = versions.data?.[0];
  const selected = Number(params.get('v')) || latest?.number;
  const view = useQuery({ queryKey: ['page', id, 'version', selected], queryFn: () => getPage(id, false, selected, of), enabled: !!selected });
  const name = useUserNames(versions.data?.map((v) => v.authorId) ?? []);
  const isCurrent = selected === latest?.number;
  const meta = versions.data?.find((v) => v.number === selected);

  const restore = useMutation({
    mutationFn: () => restoreVersion(id, selected!, message.trim() || `Restored version ${selected}`),
    onSuccess: async (r) => {
      await Promise.all([qc.invalidateQueries({ queryKey: ['page', id] }), qc.invalidateQueries({ queryKey: ['versions', id] }), qc.invalidateQueries({ queryKey: ['tree'] })]);
      setRestoring(false);
      toast({ title: `Version ${selected} restored as version ${r.number}`, appearance: 'success' });
      navigate(go.view(id, view.data?.title ?? ''));
    },
    onError: (e) => toast({ title: "Couldn't restore the version", description: e.message, appearance: 'error' }),
  });

  return (
    <div className="kk-pane">
      <QueryState query={versions} skeleton={<div className="kk-container"><Skeleton height={36} width="40%" /><Skeleton height={240} style={{ marginTop: 16 }} /></div>}>
        {(list) => {
          const title = view.data?.title ?? '';
          return view.data && view.data.spaceId !== space.id ? (
            <NotFound />
          ) : (
            <>
              <PageHeaderBar type={of} crumbs={[{ label: space.name, href: `/wiki/spaces/${space.key}/overview` }, { label: title || (of === 'page' ? 'Page' : 'Blog post'), href: go.view(id, title) }, { label: 'History' }]}>
                <NotBuilt icon="history" label="Compare versions" />
              </PageHeaderBar>
              <div className="kk-history">
                <nav className="kk-history__list" aria-label="Versions">
                  <h1 className="kk-h2">{of === 'page' ? 'Page' : 'Blog post'} history</h1>
                  <ol className="kk-versions">
                    {list.map((v) => (
                      <li key={v.number}>
                        <button type="button" className="kk-version" aria-current={v.number === selected ? 'true' : undefined} onClick={() => setParams({ v: String(v.number) })}>
                          <span className="kk-row">
                            <strong>Version {v.number}</strong>
                            {v.number === latest?.number && <Lozenge appearance="brand">Current</Lozenge>}
                            {v.minorEdit && <span className="kk-muted">minor</span>}
                          </span>
                          <span className="kk-row kk-muted"><Avatar name={name(v.authorId)} size="sm" />{name(v.authorId)}</span>
                          <time className="kk-muted" dateTime={v.createdAt} title={exactTime(v.createdAt)}>{relativeTime(v.createdAt)}</time>
                          {v.message && <span className="kk-version__msg">{v.message}</span>}
                        </button>
                      </li>
                    ))}
                  </ol>
                </nav>
                <div className="kk-history__view">
                  <QueryState query={view} skeleton={<Skeleton height={240} />}>
                    {(p) => (
                      <>
                        <div className="kk-banner" role="status">
                          <span>{isCurrent ? `This is the current version (${selected}).` : `You are viewing version ${selected}${meta ? `, saved ${relativeTime(meta.createdAt)} by ${name(meta.authorId)}` : ''}.`}</span>
                          {isCurrent ? (
                            <Link to={go.view(id, p.title)}>{of === 'page' ? 'Back to page' : 'Back to blog post'}</Link>
                          ) : (
                            <Button appearance="primary" onClick={() => { setMessage(''); setRestoring(true); }}>Restore this version</Button>
                          )}
                        </div>
                        <article className="kk-canvas kk-canvas--history">
                          <h2 className="kk-canvas__title">{p.title || 'Untitled'}</h2>
                          <AdfView doc={parseAdf(p.body?.atlas_doc_format?.value)} resolveMedia={resolveMedia} annotations={PLAIN} />
                        </article>
                      </>
                    )}
                  </QueryState>
                </div>
              </div>
              <Dialog
                open={restoring}
                onClose={() => setRestoring(false)}
                title={`Restore version ${selected}?`}
                onSubmit={() => restore.mutate()}
                footer={
                  <>
                    <Button onClick={() => setRestoring(false)}>Cancel</Button>
                    <Button type="submit" appearance="primary" loading={restore.isPending}>Restore</Button>
                  </>
                }
              >
                <p style={{ margin: 0 }}>This publishes the title and content of version {selected} as a new version. Nothing is deleted, and the versions in between stay in the history. Inline comment highlights that are not in version {selected} are dropped, and their comments become orphaned.</p>
                <TextField label="Version comment (optional)" value={message} onChange={(e) => setMessage(e.target.value)} placeholder={`Restored version ${selected}`} />
              </Dialog>
            </>
          );
        }}
      </QueryState>
    </div>
  );
}
