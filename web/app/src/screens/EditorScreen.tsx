import { Breadcrumbs, Button, Dialog, Icon, IconButton, Lozenge, Menu, MenuItem, Skeleton, TextArea, useToast } from '@kaijukit/ui';
import { emptyDoc, parseAdf, unsupportedTypes, type AdfDoc } from '@kaijukit/editor/adf';
import { Editor, type EditorHandle } from '@kaijukit/editor';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useNavigate, useParams, useSearchParams } from 'react-router-dom';
import { ApiError, createDraft, deletePage, getPage, savePage, type Page, type Space } from '../api';
import { pageUrl } from '../format';
import { useSpace } from '../hooks';
import { QueryState } from '../QueryState';
import { ErrorPanel } from './Errors';

const SAVE_DELAY = 1500;

type SaveState = 'idle' | 'saving' | 'saved' | 'error' | 'offline';
const CHIP: Record<SaveState, [string, 'default' | 'info' | 'success' | 'danger' | 'warning']> = {
  idle: ['Draft', 'default'],
  saving: ['Saving...', 'info'],
  saved: ['Saved', 'success'],
  error: ['Not saved', 'danger'],
  offline: ['Offline', 'warning'],
};

export function CreateRoute() {
  const { key } = useParams();
  const [params] = useSearchParams();
  const space = useSpace(key);
  const navigate = useNavigate();
  const started = useRef(false);
  const [error, setError] = useState<unknown>(null);
  const [attempt, setAttempt] = useState(0);
  const parent = params.get('parentPageId');
  const s = space.data;

  useEffect(() => {
    if (!s || started.current) return;
    started.current = true;
    createDraft(s.id, parent ?? s.homepageId, JSON.stringify(emptyDoc()))
      .then((p) => navigate(`/wiki/spaces/${s.key}/pages/edit-v2/${p.id}`, { replace: true }))
      .catch((e) => {
        started.current = false;
        setError(e);
      });
  }, [s, parent, navigate, attempt]);

  if (space.isError) return <div className="kk-container"><ErrorPanel error={space.error} onRetry={() => void space.refetch()} /></div>;
  if (error) return <div className="kk-container"><ErrorPanel error={error} onRetry={() => { setError(null); setAttempt((a) => a + 1); }} /></div>;
  return <div className="kk-container" aria-busy="true"><Skeleton height={36} width="50%" /><Skeleton height={200} style={{ marginTop: 16 }} /></div>;
}

export function EditorRoute() {
  const { key, id } = useParams();
  const space = useSpace(key);
  const page = useQuery({ queryKey: ['page', id, 'draft'], queryFn: () => getPage(id!, true), staleTime: 0, gcTime: 0, refetchOnWindowFocus: false });
  return (
    <div className="kk-edit">
      <QueryState query={space} skeleton={<Skeleton height={48} />}>
        {(s) => (
          <QueryState query={page} skeleton={<div className="kk-edit__col"><Skeleton height={40} width="60%" /><Skeleton height={300} style={{ marginTop: 16 }} /></div>}>
            {(p) => (p.spaceId !== s.id ? <ErrorPanel error={new ApiError(404, '')} /> : <EditorScreen key={p.id} space={s} page={p} />)}
          </QueryState>
        )}
      </QueryState>
    </div>
  );
}

function EditorScreen({ space, page }: { space: Space; page: Page }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const doc = useMemo<AdfDoc>(() => parseAdf(page.body?.atlas_doc_format?.value) ?? emptyDoc(), [page]);
  const unsupported = useMemo(() => unsupportedTypes(doc), [doc]);
  const [title, setTitle] = useState(page.title);
  const [state, setState] = useState<SaveState>('idle');
  const [dirty, setDirty] = useState(false);
  const [publishOpen, setPublishOpen] = useState(false);
  const [discardOpen, setDiscardOpen] = useState(false);
  const [message, setMessage] = useState('');
  const editor = useRef<EditorHandle>(null);
  const titleEl = useRef<HTMLTextAreaElement>(null);
  const latest = useRef({ title: page.title, body: JSON.stringify(doc) });
  const rev = useRef(0);
  const savedRev = useRef(0);
  const draftVersion = useRef(page.version.number);
  const timer = useRef<ReturnType<typeof setTimeout>>();
  const chain = useRef<Promise<void>>(Promise.resolve());

  const flush = useCallback((): Promise<void> => {
    clearTimeout(timer.current);
    chain.current = chain.current.then(async () => {
      if (rev.current === savedRev.current) return;
      const at = rev.current;
      setState('saving');
      try {
        const p = await savePage({ id: page.id, status: 'draft', title: latest.current.title, body: latest.current.body, version: draftVersion.current });
        draftVersion.current = p.version.number;
        savedRev.current = at;
        setState(navigator.onLine ? 'saved' : 'offline');
        setDirty(rev.current !== savedRev.current);
      } catch {
        setState(navigator.onLine ? 'error' : 'offline');
      }
    });
    return chain.current;
  }, [page.id]);

  const changed = useCallback(() => {
    rev.current++;
    setDirty(true);
    setState('idle');
    clearTimeout(timer.current);
    timer.current = setTimeout(() => void flush(), SAVE_DELAY);
  }, [flush]);

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
      const live = await getPage(page.id, false).catch(() => null);
      const version = live?.status === 'current' ? live.version.number + 1 : draftVersion.current;
      return savePage({ id: page.id, status: 'current', title: latest.current.title.trim(), body: latest.current.body, version, message: message.trim() || undefined });
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

  const close = async () => {
    await flush();
    done();
    navigate(page.status === 'draft' && page.version.number === 1 ? pageUrl(space.key, page.id, latest.current.title) : pageUrl(space.key, page.id, page.title));
  };

  const [chipText, chipTone] = CHIP[state];
  const canPublish = title.trim().length > 0 && (dirty || page.status === 'draft' || state !== 'idle');

  return (
    <>
      <header className="kk-edit__head">
        <Breadcrumbs items={[{ label: space.name, href: `/wiki/spaces/${space.key}/overview` }, { label: title.trim() || 'New page' }]} renderLink={({ href, children }) => <Link to={href}>{children}</Link>} />
        <span role="status" aria-live="polite" className="kk-edit__chip">
          <Lozenge appearance={chipTone}>{chipText}</Lozenge>
        </span>
        <span className="kk-edit__spacer" />
        <Button onClick={() => void close()}>Close</Button>
        <Menu label="Page options" align="end" renderTrigger={(t) => <IconButton label="Page options" icon={<Icon name="more" />} {...t} />}>
          <MenuItem danger onSelect={() => setDiscardOpen(true)}>Discard draft</MenuItem>
        </Menu>
        <Button appearance="primary" disabled={!canPublish} onClick={() => { publish.reset(); setPublishOpen(true); }}>
          Publish
        </Button>
      </header>
      <main className="kk-edit__col" id="main">
        <textarea
          ref={titleEl}
          className="kk-edit__title"
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
          onChange={(d) => {
            latest.current.body = JSON.stringify(d);
            changed();
          }}
        />
      </main>
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
        <TextArea label="Version comment (optional)" rows={3} value={message} onChange={(e) => setMessage(e.target.value)} autoFocus error={publish.error?.message} />
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
    </>
  );
}
