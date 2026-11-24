import { Button, Dialog, Icon, TextField, Tooltip, type IconName } from '@kaijukit/ui';
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
import { forwardRef, useCallback, useEffect, useImperativeHandle, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { fromAdf, toAdf, type AdfDoc, type AdfNode } from './adf';

export interface EditorHandle {
  focus: () => void;
}
export interface EditorProps {
  value: AdfNode | null;
  onChange: (doc: AdfDoc) => void;
  label?: string;
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

interface ToolItem {
  key: string;
  label: string;
  icon?: IconName;
  text?: string;
  active?: (e: TiptapEditor) => boolean;
  run: (e: TiptapEditor) => void;
  can?: (e: TiptapEditor) => boolean;
}
const toggle = (name: string): Pick<ToolItem, 'active'> => ({ active: (e) => e.isActive(name) });
const GROUPS: ToolItem[][] = [
  [
    { key: 'undo', label: 'Undo', icon: 'undo', run: (e) => e.chain().focus().undo().run(), can: (e) => e.can().undo() },
    { key: 'redo', label: 'Redo', icon: 'redo', run: (e) => e.chain().focus().redo().run(), can: (e) => e.can().redo() },
  ],
  [1, 2, 3].map((level) => ({ key: `h${level}`, label: `Heading ${level}`, text: `H${level}`, active: (e: TiptapEditor) => e.isActive('heading', { level }), run: (e: TiptapEditor) => e.chain().focus().toggleHeading({ level: level as 1 }).run() })),
  [
    { key: 'bold', label: 'Bold', icon: 'bold', ...toggle('bold'), run: (e) => e.chain().focus().toggleBold().run() },
    { key: 'italic', label: 'Italic', icon: 'italic', ...toggle('italic'), run: (e) => e.chain().focus().toggleItalic().run() },
    { key: 'underline', label: 'Underline', icon: 'underline', ...toggle('underline'), run: (e) => e.chain().focus().toggleUnderline().run() },
    { key: 'strike', label: 'Strikethrough', icon: 'strike', ...toggle('strike'), run: (e) => e.chain().focus().toggleStrike().run() },
    { key: 'code', label: 'Inline code', icon: 'code', ...toggle('code'), run: (e) => e.chain().focus().toggleCode().run() },
  ],
  [
    { key: 'bullets', label: 'Bulleted list', icon: 'bullets', ...toggle('bulletList'), run: (e) => e.chain().focus().toggleBulletList().run() },
    { key: 'numbers', label: 'Numbered list', icon: 'numbers', ...toggle('orderedList'), run: (e) => e.chain().focus().toggleOrderedList().run() },
    { key: 'quote', label: 'Quote', icon: 'quote', ...toggle('blockquote'), run: (e) => e.chain().focus().toggleBlockquote().run() },
    { key: 'codeblock', label: 'Code block', text: '{ }', ...toggle('codeBlock'), run: (e) => e.chain().focus().toggleCodeBlock().run() },
    { key: 'rule', label: 'Divider', icon: 'rule', run: (e) => e.chain().focus().setHorizontalRule().run() },
    { key: 'table', label: 'Insert table', icon: 'table', run: (e) => e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(), can: (e) => !e.isActive('table') },
  ],
];
const TABLE_TOOLS: ToolItem[] = [
  { key: 'addRow', label: 'Add row below', text: '+ Row', run: (e) => e.chain().focus().addRowAfter().run() },
  { key: 'addCol', label: 'Add column right', text: '+ Col', run: (e) => e.chain().focus().addColumnAfter().run() },
  { key: 'delRow', label: 'Delete row', text: '- Row', run: (e) => e.chain().focus().deleteRow().run() },
  { key: 'delCol', label: 'Delete column', text: '- Col', run: (e) => e.chain().focus().deleteColumn().run() },
  { key: 'delTable', label: 'Delete table', icon: 'trash', run: (e) => e.chain().focus().deleteTable().run() },
];

function Toolbar({ editor, onLink }: { editor: TiptapEditor; onLink: () => void }) {
  const bar = useRef<HTMLDivElement>(null);
  const [tab, setTab] = useState(0);
  const inTable = editor.isActive('table');
  const linkItem: ToolItem = { key: 'link', label: 'Link', icon: 'link', ...toggle('link'), run: onLink };
  const groups = [...GROUPS.slice(0, 3), [linkItem], GROUPS[3]!, ...(inTable ? [TABLE_TOOLS] : [])];
  let idx = -1;

  const onKey = (e: KeyboardEvent) => {
    const els = Array.from(bar.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? []);
    const i = els.indexOf(document.activeElement as HTMLButtonElement);
    const n = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : e.key === 'Home' ? 0 : e.key === 'End' ? els.length - 1 : null;
    if (n === null || i < 0) return;
    e.preventDefault();
    const t = els[(n + els.length) % els.length];
    t?.focus();
    setTab(Number(t?.dataset.idx));
  };

  return (
    <div ref={bar} className="kk-toolbar" role="toolbar" aria-label="Formatting" onKeyDown={onKey}>
      {groups.map((g, gi) => (
        <span key={gi} role="group" style={{ display: 'contents' }}>
          {gi > 0 && <span className="kk-toolbar__sep" aria-hidden="true" />}
          {g.map((t) => {
            idx++;
            const active = t.active?.(editor) ?? false;
            return (
              <Tooltip key={t.key} content={t.label}>
                <Button
                  appearance="subtle"
                  aria-label={t.label}
                  aria-pressed={t.active ? active : undefined}
                  data-idx={idx}
                  tabIndex={idx === tab ? 0 : -1}
                  disabled={t.can ? !t.can(editor) : false}
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => t.run(editor)}
                  onFocus={(e) => setTab(Number(e.currentTarget.dataset.idx))}
                >
                  {t.icon ? <Icon name={t.icon} /> : t.text}
                </Button>
              </Tooltip>
            );
          })}
        </span>
      ))}
    </div>
  );
}

const ALLOWED = /^(https?:|mailto:|tel:)/i;
function normalizeUrl(raw: string): string | null {
  const v = raw.trim();
  if (!v) return null;
  if (/^[a-z][a-z0-9+.-]*:/i.test(v)) return ALLOWED.test(v) ? v : null;
  return `https://${v}`;
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ value, onChange, label = 'Page content' }, ref) {
  const [slash, setSlash] = useState<Slash | null>(null);
  const [index, setIndex] = useState(0);
  const [linkOpen, setLinkOpen] = useState(false);
  const [url, setUrl] = useState('');
  const [urlError, setUrlError] = useState('');
  const [, force] = useState(0);
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
    onTransaction: () => force((n) => n + 1),
  });
  editorRef.current = editor;

  useImperativeHandle(ref, () => ({ focus: () => editor?.commands.focus('start') }), [editor]);

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
  return (
    <div className="kk-editor">
      <Toolbar editor={editor} onLink={openLink} />
      <div className="kk-editor__content" onClick={(e) => e.target === e.currentTarget && editor.commands.focus('end')}>
        <EditorContent editor={editor} />
      </div>
      {slash && (
        <ul id="kk-slash" role="listbox" aria-label="Insert" className="kk-slash" style={{ left: slash.left, top: slash.top }} onMouseDown={(e) => e.preventDefault()}>
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
