import { Button, Dialog, Icon, IconButton, Lozenge, Menu, MenuItem, Skeleton, TextArea, useToast } from '@kaijukit/ui';
import { emptyDoc, parseAdf, unsupportedTypes, type AdfDoc } from '@kaijukit/editor/adf';
import { Editor, type EditorHandle } from '@kaijukit/editor';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useOutletContext, useParams, useSearchParams } from 'react-router-dom';
import { ApiError, createDraft, deletePage, getPage, resolveMedia, savePage, uploadAttachment, type Page, type Space } from '../api';
import { searchPeople } from '../contentApi';
import { pageUrl } from '../format';
import { PageHeaderBar } from '../PageHeader';
import { QueryState } from '../QueryState';
import { getJson, store } from '../storage';
import { ErrorPanel } from './Errors';

const SAVE_DELAY = 1500;

type SaveState = 'idle' | 'saving' | 'saved' | 'error' | 'offline' | 'unsaved';
const CHIP: Record<SaveState, [string, 'default' | 'info' | 'success' | 'danger' | 'warning']> = {
  idle: ['Draft', 'default'],
  saving: ['Saving...', 'info'],
  saved: ['Saved', 'success'],
  error: ['Not saved', 'danger'],
  offline: ['Offline', 'warning'],
  unsaved: ['Unpublished changes', 'warning'],
};

export function CreateRoute() {
  const [params] = useSearchParams();
  const s = useOutletContext<Space>();
  const navigate = useNavigate();
  const started = useRef(false);
  const [error, setError] = useState<unknown>(null);
  const [attempt, setAttempt] = useState(0);
  const parent = params.get('parentPageId');

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    createDraft(s.id, parent ?? s.homepageId, JSON.stringify(emptyDoc()))
      .then((p) => navigate(`/wiki/spaces/${s.key}/pages/edit-v2/${p.id}`, { replace: true }))
      .catch((e) => {
        started.current = false;
        setError(e);
      });
  }, [s, parent, navigate, attempt]);

  if (error) return <div className="kk-container"><ErrorPanel error={error} onRetry={() => { setError(null); setAttempt((a) => a + 1); }} /></div>;
  return <div className="kk-container" aria-busy="true"><Skeleton height={36} width="50%" /><Skeleton height={200} style={{ marginTop: 16 }} /></div>;
}

export function EditorRoute() {
  const { id } = useParams();
  const space = useOutletContext<Space>();
  const page = useQuery({ queryKey: ['page', id, 'draft'], queryFn: () => getPage(id!, true), staleTime: 0, gcTime: 0, refetchOnWindowFocus: false });
  return (
    <QueryState query={page} skeleton={<div className="kk-canvas"><Skeleton height={40} width="60%" /><Skeleton height={300} style={{ marginTop: 16 }} /></div>}>
      {(p) => (p.spaceId !== space.id ? <ErrorPanel error={new ApiError(404, '')} /> : <EditorScreen key={p.id} space={space} page={p} />)}
    </QueryState>
  );
}

function EditorScreen({ space, page }: { space: Space; page: Page }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const doc = useMemo<AdfDoc>(() => parseAdf(page.body?.atlas_doc_format?.value) ?? emptyDoc(), [page]);
  const unsupported = useMemo(() => unsupportedTypes(doc), [doc]);
  const searchUsers = useCallback(async (q: string) => (await searchPeople(q)).map((u) => ({ id: u.accountId, name: u.displayName || `User ${u.accountId.slice(0, 8)}` })), []);
  const upload = useCallback((f: File) => uploadAttachment(page.id, f), [page.id]);
  const [title, setTitle] = useState(page.title);
  const [state, setState] = useState<SaveState>('idle');
  const [dirty, setDirty] = useState(false);
  const [publishOpen, setPublishOpen] = useState(false);
  const [discardOpen, setDiscardOpen] = useState(false);
  const [leaveOpen, setLeaveOpen] = useState(false);
  const [message, setMessage] = useState('');
  const [toolbarHost, setToolbarHost] = useState<HTMLElement | null>(null);
  const [pinned, setPinned] = useState(() => getJson('kk.toolbar.pinned', true));
  const editor = useRef<EditorHandle>(null);
  const titleEl = useRef<HTMLTextAreaElement>(null);
  const latest = useRef({ title: page.title, body: JSON.stringify(doc) });
  const rev = useRef(0);
  const savedRev = useRef(0);
  const autosave = page.status === 'draft';
  const baseVersion = useRef(page.version.number);
  const timer = useRef<ReturnType<typeof setTimeout>>();
  const chain = useRef<Promise<void>>(Promise.resolve());

  // The server appends a version on every PUT, so the next number is always the last returned one + 1.
  // A 409 means someone else wrote in between: adopt the server's number once and retry.
  const put = useCallback(
    async (status: 'draft' | 'current', title: string, body: string, message?: string) => {
      const send = () => savePage({ id: page.id, status, title, body, version: baseVersion.current + 1, message });
      let p: Page;
      try {
        p = await send();
      } catch (e) {
        if (!(e instanceof ApiError && e.status === 409)) throw e;
        baseVersion.current = (await getPage(page.id, true)).version.number;
        p = await send();
      }
      baseVersion.current = p.version.number;
      return p;
    },
    [page.id],
  );

  const flush = useCallback((): Promise<void> => {
    clearTimeout(timer.current);
    chain.current = chain.current.then(async () => {
      if (!autosave || rev.current === savedRev.current) return;
      const at = rev.current;
      setState('saving');
      try {
        await put('draft', latest.current.title, latest.current.body);
        savedRev.current = at;
        setState(navigator.onLine ? 'saved' : 'offline');
        setDirty(rev.current !== savedRev.current);
      } catch (e) {
        if (e instanceof ApiError && e.status === 409) toast({ title: 'This page changed elsewhere', description: 'Your draft could not be saved. Copy your text, reload, and reapply it.', appearance: 'error' });
        else if (e instanceof ApiError && e.status === 403) toast({ title: "You don't have permission to edit this page", description: e.message, appearance: 'error' });
        setState(navigator.onLine ? 'error' : 'offline');
      }
    });
    return chain.current;
  }, [put, toast, autosave]);

  const changed = useCallback(() => {
    rev.current++;
    setDirty(true);
    clearTimeout(timer.current);
    // The server cannot hold a draft of an already published page, so those edits stay local until Publish.
    if (!autosave) return setState('unsaved');
    setState('idle');
    timer.current = setTimeout(() => void flush(), SAVE_DELAY);
  }, [flush, autosave]);

  useEffect(() => () => clearTimeout(timer.current), []);
  useEffect(() => {
    const el = titleEl.current;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${el.scrollHeight}px`;
    }
  }, [title]);
  useEffect(() => {
    const warn = (e: BeforeUnloadEvent) => {
      if (rev.current !== savedRev.current) e.preventDefault();
    };
    const off = () => setState('offline');
    const on = () => rev.current !== savedRev.current && void flush();
    window.addEventListener('beforeunload', warn);
    window.addEventListener('offline', off);
    window.addEventListener('online', on);
    return () => {
      window.removeEventListener('beforeunload', warn);
      window.removeEventListener('offline', off);
      window.removeEventListener('online', on);
    };
  }, [flush]);

  const done = () => {
    void qc.invalidateQueries({ queryKey: ['page'] });
    void qc.invalidateQueries({ queryKey: ['tree'] });
    void qc.invalidateQueries({ queryKey: ['recent'] });
  };

  const publish = useMutation({
    mutationFn: async () => {
      clearTimeout(timer.current);
      await chain.current;
      return put('current', latest.current.title.trim(), latest.current.body, message.trim() || undefined);
    },
    onSuccess: (p) => {
      savedRev.current = rev.current;
      done();
      toast({ title: 'Page published', appearance: 'success' });
      navigate(pageUrl(space.key, p.id, p.title), { replace: true });
    },
  });

  const discard = useMutation({
    mutationFn: async () => {
      clearTimeout(timer.current);
      await chain.current;
      await deletePage(page.id, true);
    },
    onSuccess: () => {
      savedRev.current = rev.current;
      done();
      toast({ title: 'Draft discarded' });
      navigate(`/wiki/spaces/${space.key}/overview`, { replace: true });
    },
    onError: (e) => toast({ title: "Couldn't discard the draft", description: e.message, appearance: 'error' }),
  });

  const close = async (force = false) => {
    if (!autosave && dirty && !force) return setLeaveOpen(true);
    await flush();
    done();
    navigate(page.status === 'draft' && page.version.number === 1 ? pageUrl(space.key, page.id, latest.current.title) : pageUrl(space.key, page.id, page.title));
  };

  const [chipText, chipTone] = CHIP[state];
  const canPublish = title.trim().length > 0 && (dirty || page.status === 'draft' || state !== 'idle');

  return (
    <div className="kk-pane">
      <PageHeaderBar type="page" crumbs={[{ label: space.name, href: `/wiki/spaces/${space.key}/overview` }, { label: title.trim() || 'New page' }]}>
        <span role="status" aria-live="polite" className="kk-edit__chip">
          <Lozenge appearance={chipTone}>{chipText}</Lozenge>
        </span>
        <Button onClick={() => void close()}>Close</Button>
        {autosave && (
          <Menu label="Page options" align="end" renderTrigger={(t) => <IconButton label="Page options" icon={<Icon name="more" />} {...t} />}>
            <MenuItem danger onSelect={() => setDiscardOpen(true)}>Discard draft</MenuItem>
          </Menu>
        )}
        <Button appearance="primary" disabled={!canPublish} onClick={() => { publish.reset(); setPublishOpen(true); }}>
          Publish
        </Button>
      </PageHeaderBar>
      <div ref={setToolbarHost} className="kk-toolbarhost" data-pinned={pinned} />
      <div className="kk-canvas">
        <textarea
          ref={titleEl}
          className="kk-canvas__title kk-canvas__title--input"
          aria-label="Title"
          placeholder="Give this page a title"
          rows={1}
          value={title}
          autoFocus={!page.title}
          onChange={(e) => {
            const t = e.target.value.replace(/\n/g, ' ');
            setTitle(t);
            latest.current.title = t;
            changed();
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              editor.current?.focus();
            }
          }}
        />
        {unsupported.length > 0 && (
          <p className="kk-edit__note" role="note">
            This page contains content the editor can't show yet ({unsupported.join(', ')}). Publishing turns it into plain text.
          </p>
        )}
        <Editor
          ref={editor}
          value={doc}
          toolbarHost={toolbarHost}
          upload={upload}
          resolveMedia={resolveMedia}
          searchUsers={searchUsers}
          onError={(m) => toast({ title: m, appearance: 'error' })}
          pinned={pinned}
          onPinnedChange={(v) => {
            setPinned(v);
            store.set('kk.toolbar.pinned', String(v));
          }}
          onChange={(d) => {
            latest.current.body = JSON.stringify(d);
            changed();
          }}
        />
      </div>
      <Dialog
        open={publishOpen}
        onClose={() => setPublishOpen(false)}
        title="Publish page"
        onSubmit={() => publish.mutate()}
        footer={
          <>
            <Button onClick={() => setPublishOpen(false)}>Cancel</Button>
            <Button type="submit" appearance="primary" loading={publish.isPending}>Publish</Button>
          </>
        }
      >
        <TextArea label="Version comment (optional)" rows={3} value={message} onChange={(e) => setMessage(e.target.value)} autoFocus />
        {publish.error && <div className="kk-error-box" role="alert">{publish.error.message}</div>}
      </Dialog>
      <Dialog
        open={leaveOpen}
        onClose={() => setLeaveOpen(false)}
        title="Leave without publishing?"
        footer={
          <>
            <Button onClick={() => setLeaveOpen(false)}>Keep editing</Button>
            <Button
              appearance="danger"
              onClick={() => {
                savedRev.current = rev.current;
                void close(true);
              }}
            >
              Leave
            </Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>Your changes to this published page are not saved until you publish them.</p>
      </Dialog>
      <Dialog
        open={discardOpen}
        onClose={() => setDiscardOpen(false)}
        title="Discard draft?"
        footer={
          <>
            <Button onClick={() => setDiscardOpen(false)}>Keep editing</Button>
            <Button appearance="danger" loading={discard.isPending} onClick={() => discard.mutate()}>Discard</Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>Unpublished changes will be lost.</p>
      </Dialog>
    </div>
  );
}
