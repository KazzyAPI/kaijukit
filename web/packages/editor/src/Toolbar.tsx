import { Button, Icon, Menu, MenuItem, MenuSeparator, Tooltip, type IconName } from '@kaijukit/ui';
import type { Editor } from '@tiptap/react';
import { useRef, useState, type KeyboardEvent } from 'react';

type Run = (e: Editor) => void;
interface Entry {
  label: string;
  run?: Run;
  active?: (e: Editor) => boolean;
  can?: (e: Editor) => boolean;
  /** Not implemented: rendered disabled with a tooltip instead of faking behavior. */
  todo?: boolean;
}
interface Tool extends Entry {
  key: string;
  icon?: IconName;
  text?: string;
  items?: (Entry | 'sep')[];
  drop?: boolean;
}

const c = (e: Editor) => e.chain().focus();
const mark = (label: string, name: string, run: Run): Entry => ({ label, run, active: (e) => e.isActive(name) });
const heading = (level: 1 | 2 | 3 | 4 | 5 | 6): Entry => ({ label: `Heading ${level}`, run: (e) => c(e).toggleHeading({ level }).run(), active: (e) => e.isActive('heading', { level }) });
const bullets = mark('Bulleted list', 'bulletList', (e) => c(e).toggleBulletList().run());
const numbers = mark('Numbered list', 'orderedList', (e) => c(e).toggleOrderedList().run());
const quote = mark('Quote', 'blockquote', (e) => c(e).toggleBlockquote().run());
const codeBlock = mark('Code block', 'codeBlock', (e) => c(e).toggleCodeBlock().run());
const divider: Entry = { label: 'Divider', run: (e) => c(e).setHorizontalRule().run() };
const insertTable: Entry = { label: 'Insert table', run: (e) => c(e).insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(), can: (e) => !e.isActive('table') };
const inTable = (e: Editor) => e.isActive('table');
const todo = (label: string): Entry => ({ label, todo: true });

const groups = (onLink: Run): Tool[][] => [
  [
    { key: 'undo', label: 'Undo', icon: 'undo', run: (e) => c(e).undo().run(), can: (e) => e.can().undo() },
    { key: 'redo', label: 'Redo', icon: 'redo', run: (e) => c(e).redo().run(), can: (e) => e.can().redo() },
  ],
  [{ key: 'style', label: 'Text style', icon: 'textStyle', items: [{ label: 'Normal text', run: (e) => c(e).setParagraph().run(), active: (e) => e.isActive('paragraph') }, ...([1, 2, 3, 4, 5, 6] as const).map(heading), 'sep', quote, codeBlock] }],
  [
    { key: 'bold', ...mark('Bold', 'bold', (e) => c(e).toggleBold().run()), icon: 'bold' },
    { key: 'italic', ...mark('Italic', 'italic', (e) => c(e).toggleItalic().run()), icon: 'italic' },
    {
      key: 'more-format',
      label: 'More text formatting',
      icon: 'chevronDown',
      items: [
        mark('Underline', 'underline', (e) => c(e).toggleUnderline().run()),
        mark('Strikethrough', 'strike', (e) => c(e).toggleStrike().run()),
        mark('Inline code', 'code', (e) => c(e).toggleCode().run()),
        'sep',
        { label: 'Clear formatting', run: (e) => c(e).unsetAllMarks().run() },
      ],
    },
  ],
  [
    { key: 'color', ...todo('Text color'), icon: 'textColor' },
    { key: 'highlight', ...todo('Highlight'), icon: 'highlight' },
    { key: 'align', ...todo('Alignment'), icon: 'alignLeft', drop: true },
  ],
  [
    { key: 'lists', label: 'Lists', icon: 'bullets', items: [bullets, numbers] },
    { key: 'task', ...todo('Action item'), icon: 'checkbox' },
  ],
  [
    { key: 'link', ...mark('Link', 'link', onLink), icon: 'link' },
    { key: 'image', ...todo('Image'), icon: 'image' },
    { key: 'mention', ...todo('Mention'), icon: 'at' },
    { key: 'emoji', ...todo('Emoji'), icon: 'smile' },
    { key: 'layouts', ...todo('Layouts'), icon: 'columns' },
  ],
  [
    {
      key: 'table',
      label: 'Table',
      icon: 'table',
      items: [
        insertTable,
        'sep',
        { label: 'Add row below', run: (e) => c(e).addRowAfter().run(), can: inTable },
        { label: 'Add column right', run: (e) => c(e).addColumnAfter().run(), can: inTable },
        { label: 'Delete row', run: (e) => c(e).deleteRow().run(), can: inTable },
        { label: 'Delete column', run: (e) => c(e).deleteColumn().run(), can: inTable },
        { label: 'Delete table', run: (e) => c(e).deleteTable().run(), can: inTable },
      ],
    },
    { key: 'insert', label: 'Insert', icon: 'plus', items: [divider, quote, codeBlock, insertTable, 'sep', todo('Panel'), todo('Expand'), todo('Status'), todo('Date'), todo('Decision')] },
  ],
];

export function Toolbar({ editor, onLink, pinned, onPinnedChange }: { editor: Editor; onLink: () => void; pinned?: boolean; onPinnedChange?: (p: boolean) => void }) {
  const bar = useRef<HTMLDivElement>(null);
  const [tab, setTab] = useState(0);
  let idx = -1;

  const onKey = (e: KeyboardEvent) => {
    if ((e.target as HTMLElement).closest('[role="menu"]')) return;
    const els = Array.from(bar.current?.querySelectorAll<HTMLButtonElement>('button[data-idx]') ?? []);
    const i = els.indexOf(document.activeElement as HTMLButtonElement);
    const n = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : e.key === 'Home' ? 0 : e.key === 'End' ? els.length - 1 : null;
    if (n === null || i < 0) return;
    e.preventDefault();
    const t = els[(n + els.length) % els.length];
    t?.focus();
    setTab(Number(t?.dataset.idx));
  };

  const face = (t: Tool) => (
    <>
      {t.icon ? <Icon name={t.icon} /> : <span aria-hidden="true">{t.text}</span>}
      {(t.items || t.drop) && t.icon !== 'chevronDown' && t.icon !== 'plus' && <Icon name="chevronDown" size={10} />}
    </>
  );

  const render = (t: Tool) => {
    const i = ++idx;
    const common = {
      appearance: 'subtle' as const,
      'aria-label': t.label,
      'data-idx': i,
      tabIndex: i === tab ? 0 : -1,
      onMouseDown: (e: React.MouseEvent) => e.preventDefault(),
      onFocus: () => setTab(i),
    };
    if (t.todo)
      return (
        <Tooltip key={t.key} content={`${t.label} is not built yet`}>
          <Button {...common} aria-disabled="true" onClick={(e) => e.preventDefault()}>
            {face(t)}
          </Button>
        </Tooltip>
      );
    if (t.items) {
      const items = t.items;
      return (
        <Menu
          key={t.key}
          label={t.label}
          renderTrigger={(p) => (
            <Tooltip content={t.label}>
              <Button {...common} {...p}>
                {face(t)}
              </Button>
            </Tooltip>
          )}
        >
          {items.map((it, n) =>
            it === 'sep' ? (
              <MenuSeparator key={n} />
            ) : (
              <MenuItem key={it.label} disabled={it.todo || (it.can ? !it.can(editor) : false)} title={it.todo ? 'Not built yet' : undefined} onSelect={() => it.run?.(editor)}>
                <span style={{ width: 16 }}>{it.active?.(editor) && <Icon name="check" />}</span>
                {it.label}
              </MenuItem>
            ),
          )}
        </Menu>
      );
    }
    const active = t.active?.(editor);
    return (
      <Tooltip key={t.key} content={t.label}>
        <Button {...common} aria-pressed={t.active ? !!active : undefined} disabled={t.can ? !t.can(editor) : false} onClick={() => t.run?.(editor)}>
          {face(t)}
        </Button>
      </Tooltip>
    );
  };

  return (
    <div ref={bar} className="kk-toolbar" role="toolbar" aria-label="Formatting" onKeyDown={onKey}>
      {groups(onLink).map((g, gi) => (
        <span key={gi} role="group" style={{ display: 'contents' }}>
          {gi > 0 && <span className="kk-toolbar__sep" aria-hidden="true" />}
          {g.map(render)}
        </span>
      ))}
      {onPinnedChange && (
        <span className="kk-toolbar__end">
          {(() => {
            const i = ++idx;
            return (
              <Tooltip content={pinned ? 'Unpin toolbar' : 'Pin toolbar'}>
                <Button appearance="subtle" aria-label="Pin toolbar" aria-pressed={!!pinned} data-idx={i} tabIndex={i === tab ? 0 : -1} onMouseDown={(e) => e.preventDefault()} onFocus={() => setTab(i)} onClick={() => onPinnedChange(!pinned)}>
                  <Icon name="pin" />
                </Button>
              </Tooltip>
            );
          })()}
        </span>
      )}
    </div>
  );
}
