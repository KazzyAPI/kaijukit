import { Button, Dialog, TextField } from '@kaijukit/ui';
import { EditorContent, useEditor, type Editor as TiptapEditor } from '@tiptap/react';
import Link from '@tiptap/extension-link';
import Placeholder from '@tiptap/extension-placeholder';
import { Table } from '@tiptap/extension-table';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import TableRow from '@tiptap/extension-table-row';
import Underline from '@tiptap/extension-underline';
import StarterKit from '@tiptap/starter-kit';
import { InputRule } from '@tiptap/core';
import { createPortal } from 'react-dom';
import { forwardRef, useCallback, useEffect, useImperativeHandle, useMemo, useRef, useState } from 'react';
import { fromAdf, toAdf, type AdfDoc, type AdfNode } from './adf';
import { Toolbar } from './Toolbar';

export interface EditorHandle {
  focus: () => void;
}
export interface EditorProps {
  value: AdfNode | null;
  onChange: (doc: AdfDoc) => void;
  label?: string;
  /** Render the toolbar here (a full-width band owned by the page) instead of above the content. */
  toolbarHost?: HTMLElement | null;
  pinned?: boolean;
  onPinnedChange?: (pinned: boolean) => void;
}

const MarkdownLink = Link.extend({
  addInputRules() {
    return [
      new InputRule({
        find: /\[([^\]]+)\]\((https?:\/\/[^\s)]+|mailto:[^\s)]+|tel:[^\s)]+)\)$/,
        handler: ({ range, match, chain }) => {
          const [, text = '', href = ''] = match;
          chain().insertContentAt(range, { type: 'text', text, marks: [{ type: 'link', attrs: { href } }] }).run();
        },
      }),
    ];
  },
});

interface Command {
  id: string;
  title: string;
  hint: string;
  keywords: string;
  run: (e: TiptapEditor) => void;
}
const heading = (level: 1 | 2 | 3 | 4 | 5 | 6): Command => ({ id: `h${level}`, title: `Heading ${level}`, hint: `${'#'.repeat(level)} `, keywords: `h${level} heading title`, run: (e) => e.chain().focus().setHeading({ level }).run() });
const COMMANDS: Command[] = [
  { id: 'p', title: 'Paragraph', hint: 'Plain text', keywords: 'text paragraph', run: (e) => e.chain().focus().setParagraph().run() },
  heading(1), heading(2), heading(3), heading(4), heading(5), heading(6),
  { id: 'ul', title: 'Bulleted list', hint: '- ', keywords: 'bullet list unordered', run: (e) => e.chain().focus().toggleBulletList().run() },
  { id: 'ol', title: 'Numbered list', hint: '1. ', keywords: 'number ordered list', run: (e) => e.chain().focus().toggleOrderedList().run() },
  { id: 'quote', title: 'Quote', hint: '> ', keywords: 'blockquote quote', run: (e) => e.chain().focus().toggleBlockquote().run() },
  { id: 'code', title: 'Code snippet', hint: '```', keywords: 'code block snippet', run: (e) => e.chain().focus().toggleCodeBlock().run() },
  { id: 'rule', title: 'Divider', hint: '---', keywords: 'rule divider line hr', run: (e) => e.chain().focus().setHorizontalRule().run() },
  { id: 'table', title: 'Table', hint: '3 x 3', keywords: 'table grid', run: (e) => e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run() },
];

interface Slash {
  from: number;
  to: number;
  query: string;
  left: number;
  top: number;
}

function detectSlash(editor: TiptapEditor): Slash | null {
  const { selection } = editor.state;
  const { $from } = selection;
  if (!selection.empty || !$from.parent.isTextblock || $from.parent.type.name === 'codeBlock') return null;
  const before = $from.parent.textBetween(0, $from.parentOffset, undefined, '￼');
  const m = /(?:^|\s)\/([\w-]*)$/.exec(before);
  if (!m) return null;
  const query = m[1] ?? '';
  const c = editor.view.coordsAtPos(selection.from);
  return { from: selection.from - query.length - 1, to: selection.from, query, left: Math.min(c.left, window.innerWidth - 276), top: c.bottom + 4 };
}

const ALLOWED = /^(https?:|mailto:|tel:)/i;
function normalizeUrl(raw: string): string | null {
  const v = raw.trim();
  if (!v) return null;
  if (/^[a-z][a-z0-9+.-]*:/i.test(v)) return ALLOWED.test(v) ? v : null;
  return `https://${v}`;
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ value, onChange, label = 'Page content', toolbarHost, pinned, onPinnedChange }, ref) {
  const [slash, setSlash] = useState<Slash | null>(null);
  const [index, setIndex] = useState(0);
  const [linkOpen, setLinkOpen] = useState(false);
  const [url, setUrl] = useState('');
  const [urlError, setUrlError] = useState('');
  const slashRef = useRef<Slash | null>(null);
  const indexRef = useRef(0);
  const dismissed = useRef<number | null>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const editorRef = useRef<TiptapEditor | null>(null);

  const matches = useMemo(() => {
    const q = slash?.query.toLowerCase() ?? '';
    return COMMANDS.filter((c) => !q || c.title.toLowerCase().includes(q) || c.keywords.includes(q));
  }, [slash?.query]);
  const matchesRef = useRef(matches);
  matchesRef.current = matches;

  const setSlashState = useCallback((s: Slash | null) => {
    slashRef.current = s;
    setSlash(s);
  }, []);

  const runCommand = useCallback((c: Command) => {
    const e = editorRef.current;
    const s = slashRef.current;
    if (!e || !s) return;
    e.chain().focus().deleteRange({ from: s.from, to: s.to }).run();
    setSlashState(null);
    c.run(e);
  }, [setSlashState]);

  const refreshSlash = useCallback((e: TiptapEditor) => {
    const s = detectSlash(e);
    if (s && dismissed.current === s.from) return setSlashState(null);
    if (!s) dismissed.current = null;
    if (s?.query !== slashRef.current?.query) {
      indexRef.current = 0;
      setIndex(0);
    }
    setSlashState(s);
  }, [setSlashState]);

  const editor = useEditor({
    extensions: [
      StarterKit,
      Underline,
      MarkdownLink.configure({ openOnClick: false, autolink: true, protocols: ['mailto', 'tel'], HTMLAttributes: { rel: 'noopener noreferrer nofollow' } }),
      Table,
      TableRow,
      TableHeader,
      TableCell,
      Placeholder.configure({ placeholder: "Type '/' for commands" }),
    ],
    content: fromAdf(value),
    editorProps: {
      attributes: { class: 'kk-doc', role: 'textbox', 'aria-multiline': 'true', 'aria-label': label },
      handleKeyDown: (_view, event) => {
        if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
          event.preventDefault();
          openLink();
          return true;
        }
        const s = slashRef.current;
        const list = matchesRef.current;
        if (!s) return false;
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
          runCommand(c);
          return true;
        }
        if (event.key === 'Escape') {
          dismissed.current = s.from;
          setSlashState(null);
          return true;
        }
        return false;
      },
    },
    onUpdate: ({ editor: e }) => {
      onChangeRef.current(toAdf(e.getJSON() as Parameters<typeof toAdf>[0]));
      refreshSlash(e);
    },
    onSelectionUpdate: ({ editor: e }) => refreshSlash(e),
  });
  editorRef.current = editor;

  useImperativeHandle(ref, () => ({ focus: () => {
      editor?.view.dom.focus();
      editor?.commands.setTextSelection(1);
    } }), [editor]);

  useEffect(() => {
    const dom = editor?.view.dom;
    if (!dom) return;
    const c = matches[index];
    if (slash && c) {
      dom.setAttribute('aria-activedescendant', `kk-slash-${c.id}`);
      dom.setAttribute('aria-controls', 'kk-slash');
    } else {
      dom.removeAttribute('aria-activedescendant');
      dom.removeAttribute('aria-controls');
    }
  }, [editor, slash, matches, index]);

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

  if (!editor) return null;
  const toolbar = <Toolbar editor={editor} onLink={openLink} pinned={pinned} onPinnedChange={onPinnedChange} />;
  return (
    <div className="kk-editor">
      {toolbarHost ? createPortal(toolbar, toolbarHost) : toolbar}
      <div className="kk-editor__content" onClick={(e) => e.target === e.currentTarget && editor.commands.focus('end')}>
        <EditorContent editor={editor} />
      </div>
      {slash && (
        <ul id="kk-slash" role="listbox" tabIndex={0} aria-label="Insert" className="kk-slash" style={{ left: slash.left, top: slash.top }} onMouseDown={(e) => e.preventDefault()}>
          {matches.length === 0 && <li className="kk-slash__empty" role="presentation">No matching items</li>}
          {matches.map((c, i) => (
            <li key={c.id} id={`kk-slash-${c.id}`} role="option" aria-selected={i === index} className="kk-slash__item" onMouseEnter={() => { indexRef.current = i; setIndex(i); }} onClick={() => runCommand(c)}>
              <span>{c.title}</span>
              <span className="kk-slash__hint">{c.hint}</span>
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
    </div>
  );
});
