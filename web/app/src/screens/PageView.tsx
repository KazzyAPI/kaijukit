import { Avatar, Button, Dialog, EmptyState, Icon, IconButton, Lozenge, Menu, MenuItem, Skeleton, useToast } from '@kaijukit/ui';
import { parseAdf, wordCount } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useNavigate, useOutletContext, useParams, useSearchParams } from 'react-router-dom';
import { deletePage, getAncestors, getPage, resolveMedia, type Kind, type Space } from '../api';
import { Attachments } from '../Attachments';
import { FooterComments } from '../Comments';
import { InlinePanel, isOpen, SelectionButton, useInlineComments, type Pending } from '../InlineComments';
import { MoveDialog } from '../MovePage';
import { Labels } from '../Labels';
import { LikeButton } from '../LikeButton';
import { ContentIcon } from '../ContentIcon';
import { readingMinutes, routes, shortDate } from '../format';
import { Collaborators, EditedAt, NotBuilt, PageHeaderBar, ShareControl, useCopyLink } from '../PageHeader';
import { QueryState } from '../QueryState';
import { SpaceActivity } from '../SpaceActivity';
import { useUserNames } from '../users';
import { WatchButton } from '../Watch';
import { NotFound } from './Errors';

export function PageRoute({ of = 'page' }: { of?: Kind }) {
  const { id } = useParams();
  return <PageView key={id} id={id!} of={of} />;
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
  return <PageView key={space.homepageId} id={space.homepageId} activity />;
}

function PageView({ id, of = 'page', activity }: { id: string; of?: Kind; activity?: boolean }) {
  const space = useOutletContext<Space>();
  const blog = of === 'blogpost';
  const go = routes(of, space.key);
  const noun = blog ? 'blog post' : 'page';
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const [confirm, setConfirm] = useState(false);
  const [moving, setMoving] = useState(false);
  const [params] = useSearchParams();
  const deepLink = params.get('focusedCommentId') ?? undefined;
  const [panelOpen, setPanelOpen] = useState(false);
  const [pending, setPending] = useState<Pending | null>(null);
  const [focusedId, setFocusedId] = useState(deepLink);
  const docRef = useRef<HTMLDivElement>(null);
  const inline = useInlineComments(id, of);
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
  const page = useQuery({ queryKey: ['page', id], queryFn: () => getPage(id, false, undefined, of) });
  const ancestors = useQuery({ queryKey: ['ancestors', id], queryFn: () => getAncestors(id), enabled: !blog });
  const p = page.data;
  const author = p?.authorId ?? p?.version.authorId;
  const name = useUserNames([author, p?.version.authorId]);
  const copy = useCopyLink(p ? go.view(p.id, p.title) : '');
  const del = useMutation({
    mutationFn: () => deletePage(id, false, of),
    onSuccess: () => {
      for (const k of ['tree', 'blogs', 'recent']) void qc.invalidateQueries({ queryKey: [k] });
      qc.removeQueries({ queryKey: ['page', id] });
      setConfirm(false);
      toast({ title: `${blog ? 'Blog post' : 'Page'} moved to trash`, appearance: 'success' });
      navigate(`/wiki/spaces/${space.key}/overview`);
    },
    onError: (e) => toast({ title: `Couldn't delete the ${noun}`, description: e.message, appearance: 'error' }),
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
          const url = go.view(p.id, p.title);
          return p.spaceId !== space.id ? (
            <NotFound />
          ) : (
            <>
              <PageHeaderBar
                type={of}
                crumbs={[
                  ...(p.id === space.homepageId ? [] : [{ label: space.name, href: `/wiki/spaces/${space.key}/overview` }]),
                  ...(ancestors.data ?? []).filter((a) => a.id !== space.homepageId).map((a) => ({ label: a.title || 'Untitled', href: routes('page', space.key).view(a.id, a.title) })),
                  { label: p.title || 'Untitled' },
                ]}
              >
                <EditedAt iso={p.version.createdAt} version={p.version.number} />
                <Collaborators ids={[p.authorId, p.version.authorId]} />
                <Button aria-pressed={panelOpen} aria-label={`Inline comments, ${openInline.length} open`} onClick={() => setPanelOpen(!panelOpen)}>
                  <Icon name="comment" /> {openInline.length}
                </Button>
                <Button appearance="primary" onClick={() => navigate(go.edit(p.id))}>
                  <Icon name="edit" /> {p.status === 'draft' ? 'Resume editing' : 'Edit'}
                </Button>
                <ShareControl pageId={p.id} title={p.title} url={url} of={of} />
                <Menu label="Page actions" align="end" renderTrigger={(t) => <IconButton label="More actions" icon={<Icon name="more" />} {...t} />}>
                  {!blog && <MenuItem onSelect={() => navigate(`${go.create}?parentPageId=${p.id}`)}>Add child page</MenuItem>}
                  <MenuItem onSelect={copy}>Copy link</MenuItem>
                  {!blog && <MenuItem onSelect={() => setMoving(true)}>Move</MenuItem>}
                  <MenuItem onSelect={() => navigate(go.history(p.id))}>{blog ? 'Blog post history' : 'Page history'}</MenuItem>
                  <MenuItem danger onSelect={() => setConfirm(true)}>Move to trash</MenuItem>
                </Menu>
              </PageHeaderBar>
              <div className="kk-pagebody">
              <article className="kk-canvas">
                <div className="kk-canvas__titlerow">
                  <h1 className="kk-canvas__title">{p.title || 'Untitled'}</h1>
                  <div className="kk-canvas__art" aria-hidden="true"><ContentIcon type={of} size={36} /></div>
                </div>
                <div className="kk-byline">
                  <span className="kk-byline__item">
                    <Avatar name={name(author)} size="sm" />
                    By {name(author)}
                  </span>
                  {blog && <span className="kk-byline__item">{shortDate(p.createdAt)}</span>}
                  <span className="kk-byline__item">
                    <Icon name="book" size={14} />
                    {readingMinutes(wordCount(doc))} min read
                  </span>
                  <NotBuilt icon="chart" label="See views" />
                  <NotBuilt icon="smilePlus" label="Add a reaction" />
                  <LikeButton pageId={p.id} of={of} />
                  <WatchButton kind="content" id={p.id} />
                  <Link className="kk-byline__item" to={go.history(p.id)}>Version {p.version.number}</Link>
                  {p.status === 'draft' && <Lozenge appearance="warning">Draft</Lozenge>}
                </div>
                <div ref={docRef}>
                  <AdfView doc={doc} resolveMedia={resolveMedia} annotations={annotations} />
                </div>
                <Labels pageId={p.id} of={of} />
                <Attachments pageId={p.id} of={of} />
                <FooterComments pageId={p.id} of={of} />
                {activity && <SpaceActivity space={space} />}
              </article>
              {panelOpen && <InlinePanel pageId={p.id} of={of} pending={pending} onDone={() => setPending(null)} focusedId={focusedId} onClose={() => { setPanelOpen(false); setPending(null); }} />}
              </div>
              <SelectionButton root={docRef} onStart={startComment} />
              <MoveDialog page={moving ? p : null} onClose={() => setMoving(false)} />
              <Dialog
                open={confirm}
                onClose={() => setConfirm(false)}
                title={`Move ${noun} to trash?`}
                footer={
                  <>
                    <Button onClick={() => setConfirm(false)}>Cancel</Button>
                    <Button appearance="danger" loading={del.isPending} onClick={() => del.mutate()}>Move to trash</Button>
                  </>
                }
              >
                <p style={{ margin: 0 }}>"{p.title || 'Untitled'}" will be removed from the space.{blog ? '' : ' Its child pages go to the trash with it.'}</p>
              </Dialog>
            </>
          );
        }}
      </QueryState>
    </div>
  );
}
