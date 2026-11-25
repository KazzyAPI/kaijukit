import { Button, Icon, Menu, MenuItem, MenuSeparator, Tooltip, type IconName } from '@kaijukit/ui';
import type { Editor } from '@tiptap/react';
import { useRef, useState, type KeyboardEvent } from 'react';
import type { Command } from './commands';
import { changeIndent, COLORS, HIGHLIGHTS, setAlign, type Align } from './extensions';
import { emoji, EMOJIS } from './insert';

type Run = (e: Editor) => void;
interface Entry {
  label: string;
  run?: Run;
  active?: (e: Editor) => boolean;
  can?: (e: Editor) => boolean;
  /** Why the entry is unavailable; it is rendered disabled with this as its tooltip. */
  off?: string;
  swatch?: string;
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
const NO_UPLOAD = 'Uploading files is not available here';
const COLOR_NAMES: Record<string, string> = { '#172b4d': 'Dark blue', '#de350b': 'Red', '#ff8b00': 'Orange', '#ff991f': 'Amber', '#00875a': 'Green', '#006644': 'Dark green', '#0065ff': 'Blue', '#403294': 'Dark purple', '#6554c0': 'Purple', '#97a0af': 'Gray', '#ffebe6': 'Light red', '#fff0b3': 'Yellow', '#e3fcef': 'Light green', '#deebff': 'Light blue', '#eae6ff': 'Light purple', '#f4f5f7': 'Light gray' };
const colorEntries = (markName: string, none: string, palette: string[]): (Entry | 'sep')[] => [
  { label: none, run: (e) => c(e).unsetMark(markName).run() },
  'sep',
  ...palette.map((hex): Entry => ({ label: COLOR_NAMES[hex] ?? hex, swatch: hex, run: (e) => c(e).setMark(markName, { color: hex }).run(), active: (e) => e.isActive(markName, { color: hex }) })),
];
const alignOf = (e: Editor): Align => (e.state.selection.$from.parent.attrs.align as Align | undefined) ?? null;
const alignEntry = (label: string, a: Align): Entry => ({ label, run: (e) => setAlign(e, a), active: (e) => alignOf(e) === a });
const byId = (commands: Command[], id: string, label?: string): Entry => {
  const cmd = commands.find((x) => x.id === id)!;
  return { label: label ?? cmd.title, run: cmd.run, can: cmd.can, off: cmd.available === false ? NO_UPLOAD : undefined };
};

const groups = (commands: Command[], onLink: Run, onFile: (k: 'image' | 'file') => void, canUpload: boolean): Tool[][] => {
  const cmd = (id: string, label?: string) => byId(commands, id, label);
  return [
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
      { key: 'color', label: 'Text color', icon: 'textColor', items: colorEntries('textColor', 'Default color', COLORS) },
      { key: 'highlight', label: 'Highlight', icon: 'highlight', items: colorEntries('backgroundColor', 'No highlight', HIGHLIGHTS) },
      {
        key: 'align',
        label: 'Alignment',
        icon: 'alignLeft',
        drop: true,
        items: [alignEntry('Align left', null), alignEntry('Align center', 'center'), alignEntry('Align right', 'end'), 'sep', { label: 'Indent', run: (e) => changeIndent(e, 1) }, { label: 'Outdent', run: (e) => changeIndent(e, -1) }],
      },
    ],
    [
      { key: 'lists', label: 'Lists', icon: 'bullets', items: [bullets, numbers] },
      { key: 'task', ...mark('Action item', 'taskList', (e) => c(e).toggleList('taskList', 'taskItem').run()), icon: 'checkbox' },
    ],
    [
      { key: 'link', ...mark('Link', 'link', onLink), icon: 'link' },
      { key: 'image', label: 'Image', icon: 'image', run: () => onFile('image'), off: canUpload ? undefined : NO_UPLOAD },
      {
        key: 'mention',
        label: 'Mention',
        icon: 'at',
        run: (e) => {
          const before = e.state.doc.textBetween(Math.max(0, e.state.selection.from - 1), e.state.selection.from);
          c(e).insertContent(before && !/\s/.test(before) ? ' @' : '@').run();
        },
      },
      { key: 'emoji', label: 'Emoji', icon: 'smile', items: EMOJIS.slice(0, 16).map(([n, ch]): Entry => ({ label: `${ch} ${n.replace(/_/g, ' ')}`, run: (e) => c(e).insertContent([emoji(n, ch), { type: 'text', text: ' ' }]).run() })) },
      { key: 'layouts', label: 'Layouts', icon: 'columns', items: [cmd('layout2'), cmd('layout3'), 'sep', { label: 'Remove layout', run: (e) => c(e).deleteNode('layoutSection').run(), can: (e) => e.isActive('layoutSection') }] },
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
      {
        key: 'insert',
        label: 'Insert',
        icon: 'plus',
        items: [divider, quote, codeBlock, insertTable, 'sep', ...(['info', 'note', 'success', 'warning', 'error'] as const).map((t) => cmd(`panel-${t}`)), cmd('expand'), cmd('status'), cmd('date'), cmd('decision'), cmd('toc'), 'sep', cmd('image'), cmd('file')],
      },
    ],
  ];
};

export function Toolbar({ editor, commands, onLink, onFile, canUpload, pinned, onPinnedChange }: { editor: Editor; commands: Command[]; onLink: () => void; onFile: (k: 'image' | 'file') => void; canUpload: boolean; pinned?: boolean; onPinnedChange?: (p: boolean) => void }) {
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
    if (t.off)
      return (
        <Tooltip above key={t.key} content={t.off}>
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
            <Tooltip above content={t.label}>
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
              <MenuItem key={it.label} disabled={!!it.off || (it.can ? !it.can(editor) : false)} title={it.off} onSelect={() => it.run?.(editor)}>
                <span style={{ width: 16 }}>{it.active?.(editor) && <Icon name="check" />}</span>
                {it.swatch && <span className="kk-swatch" style={{ background: it.swatch }} aria-hidden="true" />}
                {it.label}
              </MenuItem>
            ),
          )}
        </Menu>
      );
    }
    const active = t.active?.(editor);
    return (
      <Tooltip above key={t.key} content={t.label}>
        <Button {...common} aria-pressed={t.active ? !!active : undefined} disabled={t.can ? !t.can(editor) : false} onClick={() => t.run?.(editor)}>
          {face(t)}
        </Button>
      </Tooltip>
    );
  };

  return (
    <div ref={bar} className="kk-toolbar" role="toolbar" aria-label="Formatting" onKeyDown={onKey}>
      {groups(commands, onLink, onFile, canUpload).map((g, gi) => (
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
              <Tooltip above content={pinned ? 'Unpin toolbar' : 'Pin toolbar'}>
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
