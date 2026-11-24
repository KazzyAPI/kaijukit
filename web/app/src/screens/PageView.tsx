import { Avatar, Breadcrumbs, Button, Dialog, EmptyState, Icon, IconButton, Lozenge, Menu, MenuItem, Skeleton, useToast } from '@kaijukit/ui';
import { parseAdf } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useNavigate, useOutletContext, useParams } from 'react-router-dom';
import { deletePage, getAncestors, getPage, type Space } from '../api';
import { exactTime, pageUrl, relativeTime } from '../format';
import { QueryState } from '../QueryState';
import { NotFound } from './Errors';

export function PageRoute() {
  const { id } = useParams();
  return <PageView id={id!} />;
}

export function Overview() {
  const space = useOutletContext<Space>();
  const navigate = useNavigate();
  if (!space.homepageId)
    return (
      <div className="kk-container">
        <EmptyState title="This space has no home page yet" action={<Button appearance="primary" onClick={() => navigate(`/wiki/spaces/${space.key}/pages/create`)}>Create a page</Button>} />
      </div>
    );
  return <PageView id={space.homepageId} />;
}

function PageView({ id }: { id: string }) {
  const space = useOutletContext<Space>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const [confirm, setConfirm] = useState(false);
  const page = useQuery({ queryKey: ['page', id], queryFn: () => getPage(id) });
  const ancestors = useQuery({ queryKey: ['ancestors', id], queryFn: () => getAncestors(id) });
  const del = useMutation({
    mutationFn: () => deletePage(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['tree'] });
      void qc.invalidateQueries({ queryKey: ['recent'] });
      qc.removeQueries({ queryKey: ['page', id] });
      setConfirm(false);
      toast({ title: 'Page moved to trash', appearance: 'success' });
      navigate(`/wiki/spaces/${space.key}/overview`);
    },
    onError: (e) => toast({ title: "Couldn't delete the page", description: e.message, appearance: 'error' }),
  });

  return (
    <article className="kk-container kk-page">
      <QueryState
        query={page}
        skeleton={
          <>
            <Skeleton height={14} width="30%" />
            <Skeleton height={36} width="60%" style={{ margin: '16px 0 12px' }} />
            <Skeleton height={14} width="40%" />
            <Skeleton height={14} style={{ marginTop: 24 }} />
            <Skeleton height={14} style={{ marginTop: 8 }} />
            <Skeleton height={14} width="80%" style={{ marginTop: 8 }} />
          </>
        }
      >
        {(p) =>
          p.spaceId !== space.id ? (
            <NotFound />
          ) : (
            <>
              <Breadcrumbs
                items={[
                  { label: space.name, href: `/wiki/spaces/${space.key}/overview` },
                  ...(ancestors.data ?? []).filter((a) => a.id !== space.homepageId).map((a) => ({ label: a.title || 'Untitled', href: pageUrl(space.key, a.id, a.title) })),
                  { label: p.title || 'Untitled' },
                ]}
                renderLink={({ href, children }) => <Link to={href}>{children}</Link>}
              />
              <div className="kk-row kk-row--between kk-page__head">
                <h1 className="kk-h1 kk-page__title">{p.title || 'Untitled'}</h1>
                <div className="kk-row">
                  <Button appearance="primary" onClick={() => navigate(`/wiki/spaces/${space.key}/pages/edit-v2/${p.id}`)}>
                    <Icon name="edit" /> {p.status === 'draft' ? 'Resume editing' : 'Edit'}
                  </Button>
                  <Menu
                    label="Page actions"
                    align="end"
                    renderTrigger={(t) => <IconButton label="More actions" icon={<Icon name="more" />} {...t} />}
                  >
                    <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/pages/create?parentPageId=${p.id}`)}>Add child page</MenuItem>
                    <MenuItem
                      onSelect={() =>
                        void navigator.clipboard?.writeText(`${location.origin}${pageUrl(space.key, p.id, p.title)}`).then(
                          () => toast({ title: 'Link copied', appearance: 'success' }),
                          () => toast({ title: "Couldn't copy the link", appearance: 'warning' }),
                        )
                      }
                    >
                      Copy link
                    </MenuItem>
                    <MenuItem danger onSelect={() => setConfirm(true)}>Move to trash</MenuItem>
                  </Menu>
                </div>
              </div>
              <div className="kk-byline">
                <Avatar name={`User ${p.version.authorId ?? p.authorId ?? '?'}`} size="sm" />
                <span>
                  Updated <time dateTime={p.version.createdAt} title={exactTime(p.version.createdAt)}>{relativeTime(p.version.createdAt)}</time> · Version {p.version.number}
                </span>
                {p.status === 'draft' && <Lozenge appearance="warning">Draft</Lozenge>}
              </div>
              <AdfView doc={parseAdf(p.body?.atlas_doc_format?.value)} />
              <section className="kk-labels" aria-labelledby="labels-h">
                <h2 id="labels-h" className="kk-labels__title">Labels</h2>
                <span className="kk-muted">No labels</span>
              </section>
              <Dialog
                open={confirm}
                onClose={() => setConfirm(false)}
                title="Move to trash?"
                footer={
                  <>
                    <Button onClick={() => setConfirm(false)}>Cancel</Button>
                    <Button appearance="danger" loading={del.isPending} onClick={() => del.mutate()}>Move to trash</Button>
                  </>
                }
              >
                <p style={{ margin: 0 }}>"{p.title || 'Untitled'}" will be removed from the space. Child pages stay where they are.</p>
              </Dialog>
            </>
          )
        }
      </QueryState>
    </article>
  );
}
