import { Button, Dialog, TextField } from '@kaijukit/ui';
import { EditorContent, useEditor, type Editor as TiptapEditor } from '@tiptap/react';
import { createPortal } from 'react-dom';
import { forwardRef, useCallback, useEffect, useImperativeHandle, useMemo, useRef, useState } from 'react';
import { fromAdf, toAdf, type AdfDoc, type AdfNode } from './adf';
import { buildCommands, insert, type Command } from './commands';
import { STATUS_COLORS, type MediaAttrs } from './extensions';
import { date, emoji, EMOJIS, mediaBlock, mention, status } from './insert';
import { kkExtensions } from './schema';
import { Toolbar } from './Toolbar';

export interface MentionCandidate {
  id: string;
  name: string;
}
export interface EditorHandle {
  focus: () => void;
  /** Marks the selected text as the anchor of an inline comment; false when nothing is selected. */
  annotate: (id: string) => boolean;
  removeAnnotation: (id: string) => void;
}
export interface EditorProps {
  value: AdfNode | null;
  onChange: (doc: AdfDoc) => void;
  label?: string;
  /** Render the toolbar here (a full-width band owned by the page) instead of above the content. */
  toolbarHost?: HTMLElement | null;
  pinned?: boolean;
  onPinnedChange?: (pinned: boolean) => void;
  /** Without it the image and file entries are disabled. */
  upload?: (file: File) => Promise<MediaAttrs>;
  resolveMedia?: (a: MediaAttrs) => Promise<string>;
  searchUsers?: (query: string) => Promise<MentionCandidate[]>;
  onAnnotationClick?: (id: string) => void;
  onError?: (message: string) => void;
}

type Kind = '/' | '@' | ':';
interface Trigger {
  kind: Kind;
  from: number;
  to: number;
  query: string;
  left: number;
  top: number;
}
interface Item {
  id: string;
  title: string;
  hint?: string;
  run: (e: TiptapEditor) => void;
}

const PATTERNS: [Kind, RegExp][] = [
  ['/', /(?:^|\s)\/([\w-]*)$/],
  ['@', /(?:^|\s)@([\w.-]*)$/],
  [':', /(?:^|\s):([\w+-]+)$/],
];

function detectTrigger(editor: TiptapEditor): Trigger | null {
  const { selection } = editor.state;
  const { $from } = selection;
  if (!selection.empty || !$from.parent.isTextblock || $from.parent.type.name === 'codeBlock') return null;
  const before = $from.parent.textBetween(0, $from.parentOffset, undefined, '￼');
  for (const [kind, re] of PATTERNS) {
    const m = re.exec(before);
    if (!m) continue;
    const query = m[1] ?? '';
    const c = editor.view.coordsAtPos(selection.from);
    return { kind, from: selection.from - query.length - 1, to: selection.from, query, left: Math.min(c.left, window.innerWidth - 276), top: c.bottom + 4 };
  }
  return null;
}

const ALLOWED = /^(https?:|mailto:|tel:)/i;
function normalizeUrl(raw: string): string | null {
  const v = raw.trim();
  if (!v) return null;
  if (/^[a-z][a-z0-9+.-]*:/i.test(v)) return ALLOWED.test(v) ? v : null;
  return `https://${v}`;
}

type NodeDialog = { kind: 'status'; pos?: number; text: string; color: string } | { kind: 'date'; pos?: number; value: string } | null;

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ value, onChange, label = 'Page content', toolbarHost, pinned, onPinnedChange, upload, resolveMedia, searchUsers, onAnnotationClick, onError }, ref) {
  const [trigger, setTriggerState] = useState<Trigger | null>(null);
  const [index, setIndex] = useState(0);
  const [people, setPeople] = useState<MentionCandidate[]>([]);
  const [linkOpen, setLinkOpen] = useState(false);
  const [url, setUrl] = useState('');
  const [urlError, setUrlError] = useState('');
  const [nodeDialog, setNodeDialog] = useState<NodeDialog>(null);
  const [uploading, setUploading] = useState<string[]>([]);
  const triggerRef = useRef<Trigger | null>(null);
  const indexRef = useRef(0);
  const dismissed = useRef<number | null>(null);
  const editorRef = useRef<TiptapEditor | null>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const fileKind = useRef<'image' | 'file'>('file');
  const live = useRef({ onChange, upload, resolveMedia, onAnnotationClick, onError });
  live.current = { onChange, upload, resolveMedia, onAnnotationClick, onError };

  const setTrigger = useCallback((t: Trigger | null) => {
    triggerRef.current = t;
    setTriggerState(t);
  }, []);

  const handleFiles = useCallback(async (files: File[]) => {
    const e = editorRef.current;
    const up = live.current.upload;
    if (!e || !up) return;
    for (const f of files) {
      setUploading((u) => [...u, f.name]);
      try {
        insert(e, mediaBlock(await up(f)));
      } catch (err) {
        live.current.onError?.(`Couldn't upload ${f.name}. ${err instanceof Error ? err.message : ''}`.trim());
      } finally {
        setUploading((u) => u.filter((n) => n !== f.name));
      }
    }
  }, []);

  const commands = useMemo(
    () =>
      buildCommands({
        canUpload: !!upload,
        status: () => setNodeDialog({ kind: 'status', text: '', color: 'neutral' }),
        date: () => setNodeDialog({ kind: 'date', value: new Date().toISOString().slice(0, 10) }),
        file: (kind) => {
          fileKind.current = kind;
          if (fileInput.current) fileInput.current.accept = kind === 'image' ? 'image/*' : '';
          fileInput.current?.click();
        },
      }),
    [upload],
  );

  const items = useMemo<Item[]>(() => {
    const q = trigger?.query.toLowerCase() ?? '';
    if (trigger?.kind === '@') return people.map((p) => ({ id: p.id, title: p.name, run: (e) => e.chain().focus().insertContent([mention(p.id, p.name), { type: 'text', text: ' ' }]).run() }));
    if (trigger?.kind === ':')
      return EMOJIS.filter(([n]) => n.includes(q))
        .slice(0, 8)
        .map(([n, c]) => ({ id: n, title: `${c} :${n}:`, run: (e) => e.chain().focus().insertContent([emoji(n, c), { type: 'text', text: ' ' }]).run() }));
    return commands.filter((c: Command) => c.available !== false && (!q || c.title.toLowerCase().includes(q) || c.keywords.includes(q))).map((c) => ({ id: c.id, title: c.title, hint: c.hint, run: c.run }));
  }, [trigger?.kind, trigger?.query, people, commands]);
  const itemsRef = useRef(items);
  itemsRef.current = items;

  useEffect(() => {
    if (trigger?.kind !== '@' || !searchUsers) return setPeople([]);
    let stale = false;
    void searchUsers(trigger.query).then((r) => !stale && setPeople(r.slice(0, 10)), () => !stale && setPeople([]));
    return () => {
      stale = true;
    };
  }, [trigger?.kind, trigger?.query, searchUsers]);

  const runItem = useCallback((item: Item) => {
    const e = editorRef.current;
    const t = triggerRef.current;
    if (!e || !t) return;
    e.chain().focus().deleteRange({ from: t.from, to: t.to }).run();
    setTrigger(null);
    item.run(e);
  }, [setTrigger]);

  const refresh = useCallback((e: TiptapEditor) => {
    const t = detectTrigger(e);
    if (t && dismissed.current === t.from) return setTrigger(null);
    if (!t) dismissed.current = null;
    if (t?.query !== triggerRef.current?.query || t?.kind !== triggerRef.current?.kind) {
      indexRef.current = 0;
      setIndex(0);
    }
    setTrigger(t);
  }, [setTrigger]);

  const editor = useEditor({
    extensions: kkExtensions((a) => (live.current.resolveMedia ? live.current.resolveMedia(a) : Promise.resolve(''))),
    content: fromAdf(value),
    editorProps: {
      attributes: { class: 'kk-doc', role: 'textbox', 'aria-multiline': 'true', 'aria-label': label },
      handleClick: (_v, _p, event) => {
        const id = (event.target as HTMLElement).closest('[data-annotation-id]')?.getAttribute('data-annotation-id');
        if (id) live.current.onAnnotationClick?.(id);
        return false;
      },
      handleClickOn: (_v, _p, node, nodePos) => {
        if (node.type.name === 'status') setNodeDialog({ kind: 'status', pos: nodePos, text: node.attrs.text, color: node.attrs.color });
        else if (node.type.name === 'date') setNodeDialog({ kind: 'date', pos: nodePos, value: new Date(Number(node.attrs.timestamp)).toISOString().slice(0, 10) });
        return false;
      },
      handleDrop: (_v, event) => {
        const files = Array.from(event.dataTransfer?.files ?? []);
        if (!files.length || !live.current.upload) return false;
        event.preventDefault();
        void handleFiles(files);
        return true;
      },
      handlePaste: (_v, event) => {
        const files = Array.from(event.clipboardData?.files ?? []);
        if (!files.length || !live.current.upload) return false;
        event.preventDefault();
        void handleFiles(files);
        return true;
      },
      handleKeyDown: (_view, event) => {
        if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
          event.preventDefault();
          openLink();
          return true;
        }
        const t = triggerRef.current;
        const list = itemsRef.current;
        if (!t) return false;
        const move = (n: number) => {
          event.preventDefault();
          indexRef.current = list.length ? (n + list.length) % list.length : 0;
          setIndex(indexRef.current);
          return true;
        };
        if (event.key === 'ArrowDown') return move(indexRef.current + 1);
        if (event.key === 'ArrowUp') return move(indexRef.current - 1);
        if (event.key === 'Enter' || event.key === 'Tab') {
          const c = list[indexRef.current];
          if (!c) return false;
          event.preventDefault();
          runItem(c);
          return true;
        }
        if (event.key === 'Escape') {
          dismissed.current = t.from;
          setTrigger(null);
          return true;
        }
        return false;
      },
    },
    onUpdate: ({ editor: e }) => {
      live.current.onChange(toAdf(e.getJSON() as Parameters<typeof toAdf>[0]));
      refresh(e);
    },
    onSelectionUpdate: ({ editor: e }) => refresh(e),
  });
  editorRef.current = editor;

  useImperativeHandle(ref, () => ({
    focus: () => {
      editor?.view.dom.focus();
      editor?.commands.setTextSelection(1);
    },
    annotate: (id) => !!editor && !editor.state.selection.empty && editor.chain().focus().setMark('annotation', { id }).run(),
    removeAnnotation: (id) => {
      editor?.state.doc.descendants((node, pos) => {
        const m = node.marks.find((x) => x.type.name === 'annotation' && x.attrs.id === id);
        if (m) editor.view.dispatch(editor.state.tr.removeMark(pos, pos + node.nodeSize, m));
      });
    },
  }), [editor]);

  useEffect(() => {
    const dom = editor?.view.dom;
    if (!dom) return;
    const c = items[index];
    if (trigger && c) {
      dom.setAttribute('aria-activedescendant', `kk-slash-${c.id}`);
      dom.setAttribute('aria-controls', 'kk-slash');
    } else {
      dom.removeAttribute('aria-activedescendant');
      dom.removeAttribute('aria-controls');
    }
  }, [editor, trigger, items, index]);

  function openLink() {
    const e = editorRef.current;
    if (!e) return;
    setUrl((e.getAttributes('link').href as string | undefined) ?? '');
    setUrlError('');
    setLinkOpen(true);
  }
  const applyLink = () => {
    const href = normalizeUrl(url);
    if (!href) return setUrlError(url.trim() ? 'Use an http, https, mailto or tel address.' : 'Enter an address.');
    editor?.chain().focus().extendMarkRange('link').setLink({ href }).run();
    setLinkOpen(false);
  };

  const applyNode = () => {
    const d = nodeDialog;
    if (!editor || !d) return;
    const node = d.kind === 'status' ? status(d.text.trim() || 'Status', d.color) : date(d.value);
    if (d.kind === 'date' && !d.value) return;
    const chain = editor.chain().focus();
    if (d.pos === undefined) chain.insertContent([node, { type: 'text', text: ' ' }]).run();
    else chain.command(({ tr }) => !!tr.setNodeMarkup(d.pos!, undefined, { ...tr.doc.nodeAt(d.pos!)?.attrs, ...node.attrs })).run();
    setNodeDialog(null);
  };

  if (!editor) return null;
  const toolbar = <Toolbar editor={editor} commands={commands} onLink={openLink} onFile={(k) => commands.find((x) => x.id === k)?.run(editor)} canUpload={!!upload} pinned={pinned} onPinnedChange={onPinnedChange} />;
  const popupLabel = trigger?.kind === '@' ? 'Mention' : trigger?.kind === ':' ? 'Emoji' : 'Insert';
  const emptyText = trigger?.kind === '@' ? (searchUsers ? 'No matching people' : 'People search is not available') : 'No matching items';
  return (
    <div className="kk-editor">
      {toolbarHost ? createPortal(toolbar, toolbarHost) : toolbar}
      <div className="kk-editor__content" onClick={(e) => e.target === e.currentTarget && editor.commands.focus('end')}>
        <EditorContent editor={editor} />
      </div>
      <div aria-live="polite" className="kk-editor__uploads">
        {uploading.map((n) => <span key={n}>Uploading {n}...</span>)}
      </div>
      <input
        ref={fileInput}
        type="file"
        multiple
        hidden
        aria-hidden="true"
        tabIndex={-1}
        onChange={(e) => {
          const files = Array.from(e.target.files ?? []);
          e.target.value = '';
          void handleFiles(fileKind.current === 'image' ? files.filter((f) => f.type.startsWith('image/')) : files);
        }}
      />
      {trigger && (
        <ul id="kk-slash" role="listbox" tabIndex={0} aria-label={popupLabel} className="kk-slash" style={{ left: trigger.left, top: trigger.top }} onMouseDown={(e) => e.preventDefault()}>
          {items.length === 0 && <li className="kk-slash__empty" role="presentation">{emptyText}</li>}
          {items.map((c, i) => (
            <li key={c.id} id={`kk-slash-${c.id}`} role="option" aria-selected={i === index} className="kk-slash__item" onMouseEnter={() => { indexRef.current = i; setIndex(i); }} onClick={() => runItem(c)}>
              <span>{c.title}</span>
              {c.hint && <span className="kk-slash__hint">{c.hint}</span>}
            </li>
          ))}
        </ul>
      )}
      <Dialog
        open={linkOpen}
        onClose={() => setLinkOpen(false)}
        title="Link"
        onSubmit={applyLink}
        footer={
          <>
            {editor.isActive('link') && (
              <Button onClick={() => { editor.chain().focus().extendMarkRange('link').unsetLink().run(); setLinkOpen(false); }}>Remove link</Button>
            )}
            <Button onClick={() => setLinkOpen(false)}>Cancel</Button>
            <Button type="submit" appearance="primary">Apply</Button>
          </>
        }
      >
        <TextField label="Address" value={url} onChange={(e) => { setUrl(e.target.value); setUrlError(''); }} error={urlError} autoFocus placeholder="https://" />
      </Dialog>
      <Dialog
        open={nodeDialog?.kind === 'status'}
        onClose={() => setNodeDialog(null)}
        title="Status"
        onSubmit={applyNode}
        footer={
          <>
            <Button onClick={() => setNodeDialog(null)}>Cancel</Button>
            <Button type="submit" appearance="primary">{nodeDialog?.pos === undefined ? 'Insert' : 'Apply'}</Button>
          </>
        }
      >
        {nodeDialog?.kind === 'status' && (
          <>
            <TextField label="Text" value={nodeDialog.text} placeholder="Status" autoFocus onChange={(e) => setNodeDialog({ ...nodeDialog, text: e.target.value })} />
            <div className="kk-field">
              <label className="kk-field__label" htmlFor="kk-status-color">Color</label>
              <select id="kk-status-color" className="kk-input" value={nodeDialog.color} onChange={(e) => setNodeDialog({ ...nodeDialog, color: e.target.value })}>
                {STATUS_COLORS.map((c) => <option key={c} value={c}>{c[0]!.toUpperCase() + c.slice(1)}</option>)}
              </select>
            </div>
          </>
        )}
      </Dialog>
      <Dialog
        open={nodeDialog?.kind === 'date'}
        onClose={() => setNodeDialog(null)}
        title="Date"
        onSubmit={applyNode}
        footer={
          <>
            <Button onClick={() => setNodeDialog(null)}>Cancel</Button>
            <Button type="submit" appearance="primary">{nodeDialog?.pos === undefined ? 'Insert' : 'Apply'}</Button>
          </>
        }
      >
        {nodeDialog?.kind === 'date' && <TextField label="Date" type="date" value={nodeDialog.value} autoFocus onChange={(e) => setNodeDialog({ ...nodeDialog, value: e.target.value })} />}
      </Dialog>
    </div>
  );
});
