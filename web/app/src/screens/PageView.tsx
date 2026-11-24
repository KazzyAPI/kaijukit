import { Avatar, Button, Dialog, EmptyState, Icon, IconButton, Lozenge, Menu, MenuItem, Skeleton, useToast } from '@kaijukit/ui';
import { parseAdf, wordCount } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useNavigate, useOutletContext, useParams } from 'react-router-dom';
import { deletePage, getAncestors, getPage, type Space } from '../api';
import { ContentIcon } from '../ContentIcon';
import { pageUrl, readingMinutes } from '../format';
import { Collaborators, EditedAt, NotBuilt, PageHeaderBar, ShareControl, useCopyLink } from '../PageHeader';
import { QueryState } from '../QueryState';
import { useUserNames } from '../users';
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
  const p = page.data;
  const author = p?.authorId ?? p?.version.authorId;
  const name = useUserNames([author, p?.version.authorId]);
  const copy = useCopyLink(p ? pageUrl(space.key, p.id, p.title) : '');
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
    <div className="kk-pane">
      <QueryState
        query={page}
        skeleton={
          <>
            <div className="kk-pagehead"><Skeleton height={16} width="40%" /></div>
            <div className="kk-canvas">
              <Skeleton height={40} width="60%" style={{ margin: '16px 0 12px' }} />
              <Skeleton height={14} width="40%" />
              <Skeleton height={14} style={{ marginTop: 24 }} />
              <Skeleton height={14} style={{ marginTop: 8 }} />
              <Skeleton height={14} width="80%" style={{ marginTop: 8 }} />
            </div>
          </>
        }
      >
        {(p) => {
          const doc = parseAdf(p.body?.atlas_doc_format?.value);
          const url = pageUrl(space.key, p.id, p.title);
          return p.spaceId !== space.id ? (
            <NotFound />
          ) : (
            <>
              <PageHeaderBar
                type="page"
                crumbs={[
                  ...(p.id === space.homepageId ? [] : [{ label: space.name, href: `/wiki/spaces/${space.key}/overview` }]),
                  ...(ancestors.data ?? []).filter((a) => a.id !== space.homepageId).map((a) => ({ label: a.title || 'Untitled', href: pageUrl(space.key, a.id, a.title) })),
                  { label: p.title || 'Untitled' },
                ]}
              >
                <EditedAt iso={p.version.createdAt} version={p.version.number} />
                <Collaborators ids={[p.authorId, p.version.authorId]} />
                <Button appearance="primary" onClick={() => navigate(`/wiki/spaces/${space.key}/pages/edit-v2/${p.id}`)}>
                  <Icon name="edit" /> {p.status === 'draft' ? 'Resume editing' : 'Edit'}
                </Button>
                <ShareControl title={p.title} url={url} />
                <Menu label="Page actions" align="end" renderTrigger={(t) => <IconButton label="More actions" icon={<Icon name="more" />} {...t} />}>
                  <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/pages/create?parentPageId=${p.id}`)}>Add child page</MenuItem>
                  <MenuItem onSelect={copy}>Copy link</MenuItem>
                  <MenuItem danger onSelect={() => setConfirm(true)}>Move to trash</MenuItem>
                </Menu>
              </PageHeaderBar>
              <article className="kk-canvas">
                <div className="kk-canvas__titlerow">
                  <h1 className="kk-canvas__title">{p.title || 'Untitled'}</h1>
                  <div className="kk-canvas__art" aria-hidden="true"><ContentIcon size={36} /></div>
                </div>
                <div className="kk-byline">
                  <span className="kk-byline__item">
                    <Avatar name={name(author)} size="sm" />
                    By {name(author)}
                  </span>
                  <span className="kk-byline__item">
                    <Icon name="book" size={14} />
                    {readingMinutes(wordCount(doc))} min read
                  </span>
                  <NotBuilt icon="chart" label="See views" />
                  <NotBuilt icon="smilePlus" label="Add a reaction" />
                  <span className="kk-byline__item">Version {p.version.number}</span>
                  {p.status === 'draft' && <Lozenge appearance="warning">Draft</Lozenge>}
                </div>
                <AdfView doc={doc} />
                <section className="kk-labels" aria-labelledby="labels-h">
                  <h2 id="labels-h" className="kk-labels__title">Labels</h2>
                  <span className="kk-muted">No labels</span>
                </section>
              </article>
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
          );
        }}
      </QueryState>
    </div>
  );
}
