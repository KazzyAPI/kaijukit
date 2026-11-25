import { Avatar, Button, Dialog, EmptyState, Icon, IconButton, Lozenge, Menu, MenuItem, Skeleton, useToast } from '@kaijukit/ui';
import { parseAdf, wordCount } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useNavigate, useOutletContext, useParams, useSearchParams } from 'react-router-dom';
import { deletePage, getAncestors, getPage, resolveMedia, type Space } from '../api';
import { Attachments } from '../Attachments';
import { FooterComments } from '../Comments';
import { InlinePanel, isOpen, SelectionButton, useInlineComments, type Pending } from '../InlineComments';
import { Labels } from '../Labels';
import { LikeButton } from '../LikeButton';
import { ContentIcon } from '../ContentIcon';
import { pageUrl, readingMinutes } from '../format';
import { Collaborators, EditedAt, NotBuilt, PageHeaderBar, ShareControl, useCopyLink } from '../PageHeader';
import { QueryState } from '../QueryState';
import { useUserNames } from '../users';
import { NotFound } from './Errors';

export function PageRoute() {
  const { id } = useParams();
  return <PageView key={id} id={id!} />;
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
  return <PageView key={space.homepageId} id={space.homepageId} />;
}

function PageView({ id }: { id: string }) {
  const space = useOutletContext<Space>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const [confirm, setConfirm] = useState(false);
  const [params] = useSearchParams();
  const deepLink = params.get('focusedCommentId') ?? undefined;
  const [panelOpen, setPanelOpen] = useState(false);
  const [pending, setPending] = useState<Pending | null>(null);
  const [focusedId, setFocusedId] = useState(deepLink);
  const docRef = useRef<HTMLDivElement>(null);
  const inline = useInlineComments(id);
  const openInline = useMemo(() => (inline.data ?? []).filter(isOpen), [inline.data]);
  const showThread = useCallback((commentId: string) => {
    setFocusedId(commentId);
    setPanelOpen(true);
  }, []);
  const startComment = useCallback((p: Pending) => {
    setPending(p);
    setPanelOpen(true);
  }, []);
  useEffect(() => {
    if (deepLink && inline.data?.some((c) => c.id === deepLink)) setPanelOpen(true);
  }, [deepLink, inline.data]);
  const annotations = useMemo(
    () => ({
      open: new Set(openInline.flatMap((c) => c.properties?.inlineMarkerRef ?? [])),
      onSelect: (marker: string) => {
        const c = openInline.find((x) => x.properties?.inlineMarkerRef === marker);
        if (c) showThread(c.id);
      },
    }),
    [openInline, showThread],
  );
  const focusedMarker = inline.data?.find((c) => c.id === focusedId)?.properties?.inlineMarkerRef;
  useEffect(() => {
    docRef.current?.querySelectorAll('[data-annotation-id]').forEach((el) => el.classList.toggle('is-focused', el.getAttribute('data-annotation-id') === focusedMarker));
  });
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
                <Button aria-pressed={panelOpen} aria-label={`Inline comments, ${openInline.length} open`} onClick={() => setPanelOpen(!panelOpen)}>
                  <Icon name="comment" /> {openInline.length}
                </Button>
                <Button appearance="primary" onClick={() => navigate(`/wiki/spaces/${space.key}/pages/edit-v2/${p.id}`)}>
                  <Icon name="edit" /> {p.status === 'draft' ? 'Resume editing' : 'Edit'}
                </Button>
                <ShareControl pageId={p.id} title={p.title} url={url} />
                <Menu label="Page actions" align="end" renderTrigger={(t) => <IconButton label="More actions" icon={<Icon name="more" />} {...t} />}>
                  <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/pages/create?parentPageId=${p.id}`)}>Add child page</MenuItem>
                  <MenuItem onSelect={copy}>Copy link</MenuItem>
                  <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/pages/history/${p.id}`)}>Page history</MenuItem>
                  <MenuItem danger onSelect={() => setConfirm(true)}>Move to trash</MenuItem>
                </Menu>
              </PageHeaderBar>
              <div className="kk-pagebody">
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
                  <LikeButton pageId={p.id} />
                  <Link className="kk-byline__item" to={`/wiki/spaces/${space.key}/pages/history/${p.id}`}>Version {p.version.number}</Link>
                  {p.status === 'draft' && <Lozenge appearance="warning">Draft</Lozenge>}
                </div>
                <div ref={docRef}>
                  <AdfView doc={doc} resolveMedia={resolveMedia} annotations={annotations} />
                </div>
                <Labels pageId={p.id} />
                <Attachments pageId={p.id} />
                <FooterComments pageId={p.id} />
              </article>
              {panelOpen && <InlinePanel pageId={p.id} pending={pending} onDone={() => setPending(null)} focusedId={focusedId} onClose={() => { setPanelOpen(false); setPending(null); }} />}
              </div>
              <SelectionButton root={docRef} onStart={startComment} />
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
