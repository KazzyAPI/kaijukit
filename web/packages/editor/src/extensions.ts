import { Extension, Mark, Node as TNode, wrappingInputRule, type Editor } from '@tiptap/core';
import { Plugin } from '@tiptap/pm/state';

export const COLORS = ['#172b4d', '#de350b', '#ff8b00', '#ff991f', '#00875a', '#006644', '#0065ff', '#403294', '#6554c0', '#97a0af'];
export const HIGHLIGHTS = ['#ffebe6', '#fff0b3', '#e3fcef', '#deebff', '#eae6ff', '#f4f5f7'];
export const PANEL_TYPES = ['info', 'note', 'success', 'warning', 'error'] as const;
export const STATUS_COLORS = ['neutral', 'purple', 'blue', 'red', 'yellow', 'green'] as const;

// Colors are written into style attributes, so only hex values are accepted from documents.
export const safeColor = (v: unknown): string | undefined => (typeof v === 'string' && /^#[0-9a-f]{3,8}$/i.test(v) ? v : undefined);

export const formatDate = (timestamp: unknown) => {
  const d = new Date(Number(timestamp));
  return Number.isNaN(d.getTime()) ? 'Invalid date' : d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric', timeZone: 'UTC' });
};

const colorMark = (name: string, css: 'color' | 'background-color') =>
  Mark.create({
    name,
    addAttributes: () => ({ color: { default: null } }),
    parseHTML: () => [{ tag: `span[data-kk-${name}]`, getAttrs: (el) => ({ color: (el as HTMLElement).getAttribute(`data-kk-${name}`) }) }],
    renderHTML: ({ HTMLAttributes: a }) => {
      const c = safeColor(a.color);
      return ['span', { [`data-kk-${name}`]: c ?? '', class: css === 'color' ? undefined : 'kk-hl', style: c ? `${css}:${c}` : undefined }, 0];
    },
  });
export const TextColor = colorMark('textColor', 'color');
export const BackgroundColor = colorMark('backgroundColor', 'background-color');

export const Annotation = Mark.create({
  name: 'annotation',
  inclusive: false,
  excludes: '',
  addAttributes: () => ({ id: { default: null }, annotationType: { default: 'inlineComment' } }),
  parseHTML: () => [{ tag: 'span[data-annotation-id]', getAttrs: (el) => ({ id: (el as HTMLElement).getAttribute('data-annotation-id') }) }],
  renderHTML: ({ HTMLAttributes: a }) => ['span', { class: 'kk-annotation', 'data-annotation-id': a.id }, 0],
});

const NO_BLOCK_FORMAT = new Set(['listItem', 'taskItem', 'decisionItem']);

export const BlockFormat = Extension.create({
  name: 'blockFormat',
  addGlobalAttributes: () => [
    {
      types: ['paragraph', 'heading'],
      attributes: {
        align: { default: null, parseHTML: (el) => el.getAttribute('data-align'), renderHTML: (a) => (a.align ? { 'data-align': a.align } : {}) },
        indent: { default: 0, parseHTML: (el) => Number(el.getAttribute('data-indent')) || 0, renderHTML: (a) => (a.indent ? { 'data-indent': a.indent } : {}) },
      },
    },
  ],
});

/** ADF has no left alignment: `null` is left, `end` is right. */
export type Align = 'center' | 'end' | null;
export function formatBlocks(editor: Editor, change: (a: { align: Align; indent: number }) => { align?: Align; indent?: number }) {
  return editor
    .chain()
    .focus()
    .command(({ tr, state }) => {
      state.doc.nodesBetween(state.selection.from, state.selection.to, (node, pos, parent) => {
        if ((node.type.name !== 'paragraph' && node.type.name !== 'heading') || (parent && NO_BLOCK_FORMAT.has(parent.type.name))) return;
        const next = change({ align: node.attrs.align, indent: node.attrs.indent });
        tr.setNodeMarkup(pos, undefined, { ...node.attrs, ...next });
      });
      return true;
    })
    .run();
}
export const setAlign = (e: Editor, align: Align) => formatBlocks(e, () => ({ align }));
export const changeIndent = (e: Editor, delta: number) => formatBlocks(e, (a) => ({ indent: Math.min(6, Math.max(0, a.indent + delta)) }));

const uuid = () => globalThis.crypto?.randomUUID?.() ?? `${Date.now().toString(16)}-${Math.random().toString(16).slice(2)}`;
const WITH_LOCAL_ID = new Set(['taskList', 'taskItem', 'decisionList', 'decisionItem', 'status']);

export const LocalIds = Extension.create({
  name: 'localIds',
  addProseMirrorPlugins: () => [
    new Plugin({
      appendTransaction: (trs, _old, state) => {
        if (!trs.some((t) => t.docChanged)) return null;
        const tr = state.tr;
        state.doc.descendants((node, pos) => {
          if (WITH_LOCAL_ID.has(node.type.name) && !node.attrs.localId) tr.setNodeMarkup(pos, undefined, { ...node.attrs, localId: uuid() });
        });
        return tr.docChanged ? tr : null;
      },
    }),
  ],
});

const NEEDS_TAIL = new Set(['extension', 'mediaSingle', 'mediaGroup', 'layoutSection', 'expand', 'table', 'horizontalRule']);

// Blocks the cursor cannot leave by typing need a paragraph after them.
export const TrailingParagraph = Extension.create({
  name: 'trailingParagraph',
  addProseMirrorPlugins: () => [
    new Plugin({
      appendTransaction: (trs, _old, state) => {
        const last = state.doc.lastChild;
        if (!trs.some((t) => t.docChanged) || !last || !NEEDS_TAIL.has(last.type.name)) return null;
        return state.tr.insert(state.doc.content.size, state.schema.nodes.paragraph!.create());
      },
    }),
  ],
});

const localId = { localId: { default: null } };

const itemList = (name: string, item: string, marker: string) =>
  TNode.create({
    name,
    group: 'block',
    content: `${item}+`,
    addAttributes: () => localId,
    parseHTML: () => [{ tag: `ul[data-type="${name}"]` }],
    renderHTML: () => ['ul', { 'data-type': name, class: 'kk-items' }, 0],
    addInputRules() {
      return [wrappingInputRule({ find: new RegExp(`^\\s*${marker}\\s$`), type: this.type })];
    },
  });

export const TaskList = itemList('taskList', 'taskItem', '\\[\\]');
export const DecisionList = itemList('decisionList', 'decisionItem', '<>');

const itemKeys = (name: string) => ({
  Enter: ({ editor }: { editor: Editor }) => editor.commands.splitListItem(name),
  'Shift-Tab': ({ editor }: { editor: Editor }) => editor.commands.liftListItem(name),
});

export const TaskItem = TNode.create({
  name: 'taskItem',
  content: 'paragraph',
  defining: true,
  addAttributes: () => ({ ...localId, state: { default: 'TODO' } }),
  parseHTML: () => [{ tag: 'li[data-type="taskItem"]' }],
  renderHTML: ({ node }) => ['li', { 'data-type': 'taskItem', 'data-state': node.attrs.state }, 0],
  addKeyboardShortcuts() {
    return itemKeys(this.name);
  },
  addNodeView() {
    return ({ node, getPos, editor }) => {
      let current = node;
      const dom = document.createElement('li');
      const box = document.createElement('input');
      const body = document.createElement('div');
      dom.className = 'kk-task';
      box.type = 'checkbox';
      box.contentEditable = 'false';
      box.setAttribute('aria-label', 'Mark as done');
      body.className = 'kk-task__body';
      dom.append(box, body);
      const sync = () => {
        box.checked = current.attrs.state === 'DONE';
        dom.dataset.state = current.attrs.state;
      };
      sync();
      box.addEventListener('change', () => {
        const pos = getPos();
        if (typeof pos !== 'number') return;
        editor.chain().command(({ tr }) => !!tr.setNodeMarkup(pos, undefined, { ...current.attrs, state: box.checked ? 'DONE' : 'TODO' })).run();
      });
      return {
        dom,
        contentDOM: body,
        update: (n) => {
          if (n.type !== current.type) return false;
          current = n;
          sync();
          return true;
        },
      };
    };
  },
});

export const DecisionItem = TNode.create({
  name: 'decisionItem',
  content: 'paragraph',
  defining: true,
  addAttributes: () => ({ ...localId, state: { default: 'DECIDED' } }),
  parseHTML: () => [{ tag: 'li[data-type="decisionItem"]' }],
  renderHTML: () => ['li', { 'data-type': 'decisionItem' }, 0],
  addKeyboardShortcuts() {
    return itemKeys(this.name);
  },
});

export const Panel = TNode.create({
  name: 'panel',
  group: 'block',
  content: 'block+',
  defining: true,
  addAttributes: () => ({ panelType: { default: 'info' } }),
  parseHTML: () => [{ tag: 'div[data-panel-type]', getAttrs: (el) => ({ panelType: (el as HTMLElement).getAttribute('data-panel-type') }) }],
  renderHTML: ({ node }) => ['div', { class: 'kk-panel', 'data-panel-type': node.attrs.panelType, role: 'group', 'aria-label': `${node.attrs.panelType} panel` }, 0],
});

export const Expand = TNode.create({
  name: 'expand',
  group: 'block',
  content: 'block+',
  defining: true,
  isolating: true,
  addAttributes: () => ({ title: { default: '' } }),
  parseHTML: () => [{ tag: 'div[data-expand]', getAttrs: (el) => ({ title: (el as HTMLElement).getAttribute('data-expand') }) }],
  renderHTML: ({ node }) => ['div', { class: 'kk-expand', 'data-expand': node.attrs.title }, 0],
  addNodeView() {
    return ({ node, getPos, editor }) => {
      const dom = document.createElement('div');
      const head = document.createElement('div');
      const toggle = document.createElement('button');
      const input = document.createElement('input');
      const body = document.createElement('div');
      dom.className = 'kk-expand';
      head.className = 'kk-expand__head';
      head.contentEditable = 'false';
      toggle.type = 'button';
      toggle.className = 'kk-expand__toggle';
      toggle.setAttribute('aria-label', 'Collapse');
      toggle.setAttribute('aria-expanded', 'true');
      input.className = 'kk-expand__title';
      input.placeholder = 'Give this expand a title';
      input.setAttribute('aria-label', 'Expand title');
      input.value = node.attrs.title;
      toggle.addEventListener('click', () => {
        const open = body.hidden;
        body.hidden = !open;
        toggle.setAttribute('aria-expanded', String(open));
        toggle.setAttribute('aria-label', open ? 'Collapse' : 'Expand');
        dom.dataset.collapsed = String(!open);
      });
      input.addEventListener('input', () => {
        const pos = getPos();
        if (typeof pos === 'number') editor.chain().command(({ tr }) => !!tr.setNodeMarkup(pos, undefined, { title: input.value })).run();
      });
      head.append(toggle, input);
      dom.append(head, body);
      return {
        dom,
        contentDOM: body,
        stopEvent: (e) => head.contains(e.target as Node),
        ignoreMutation: (m) => head.contains(m.target),
        update: (n) => {
          if (n.type !== node.type) return false;
          if (input.value !== n.attrs.title) input.value = n.attrs.title;
          return true;
        },
      };
    };
  },
});

export const LayoutSection = TNode.create({
  name: 'layoutSection',
  group: 'block',
  content: 'layoutColumn{2,3}',
  isolating: true,
  parseHTML: () => [{ tag: 'div[data-layout-section]' }],
  renderHTML: () => ['div', { class: 'kk-layout', 'data-layout-section': '' }, 0],
});

export const LayoutColumn = TNode.create({
  name: 'layoutColumn',
  content: 'block+',
  isolating: true,
  addAttributes: () => ({ width: { default: 50 } }),
  parseHTML: () => [{ tag: 'div[data-layout-column]' }],
  renderHTML: ({ node }) => ['div', { class: 'kk-layout__col', 'data-layout-column': '', style: `flex: ${node.attrs.width} 1 0` }, 0],
});

export const Status = TNode.create({
  name: 'status',
  group: 'inline',
  inline: true,
  atom: true,
  addAttributes: () => ({ ...localId, text: { default: 'Status' }, color: { default: 'neutral' } }),
  parseHTML: () => [{ tag: 'span[data-status]' }],
  renderHTML: ({ node }) => ['span', { class: `kk-status kk-status--${node.attrs.color}`, 'data-status': '' }, String(node.attrs.text).toUpperCase()],
});

export const DateNode = TNode.create({
  name: 'date',
  group: 'inline',
  inline: true,
  atom: true,
  addAttributes: () => ({ timestamp: { default: null } }),
  parseHTML: () => [{ tag: 'time[data-timestamp]', getAttrs: (el) => ({ timestamp: (el as HTMLElement).getAttribute('data-timestamp') }) }],
  renderHTML: ({ node }) => ['time', { class: 'kk-date', 'data-timestamp': node.attrs.timestamp }, formatDate(node.attrs.timestamp)],
});

export const Mention = TNode.create({
  name: 'mention',
  group: 'inline',
  inline: true,
  atom: true,
  addAttributes: () => ({ id: { default: null }, text: { default: '@unknown' }, accessLevel: { default: null } }),
  parseHTML: () => [{ tag: 'span[data-mention-id]', getAttrs: (el) => ({ id: (el as HTMLElement).getAttribute('data-mention-id'), text: el.textContent }) }],
  renderHTML: ({ node }) => ['span', { class: 'kk-mention', 'data-mention-id': node.attrs.id }, node.attrs.text],
});

export const Emoji = TNode.create({
  name: 'emoji',
  group: 'inline',
  inline: true,
  atom: true,
  addAttributes: () => ({ shortName: { default: null }, id: { default: null }, text: { default: null } }),
  parseHTML: () => [{ tag: 'span[data-emoji]', getAttrs: (el) => ({ shortName: (el as HTMLElement).getAttribute('data-emoji'), text: el.textContent }) }],
  renderHTML: ({ node }) => ['span', { class: 'kk-emoji', 'data-emoji': node.attrs.shortName, role: 'img', 'aria-label': String(node.attrs.shortName).replace(/:/g, '') }, node.attrs.text ?? node.attrs.shortName],
});

export const TOC_ATTRS = { extensionType: 'kk.macro', extensionKey: 'toc', parameters: { macroParams: {} }, layout: 'default' };

export const TocExtension = TNode.create({
  name: 'extension',
  group: 'block',
  atom: true,
  selectable: true,
  addAttributes: () => ({ extensionType: { default: TOC_ATTRS.extensionType }, extensionKey: { default: 'toc' }, parameters: { default: TOC_ATTRS.parameters }, layout: { default: 'default' } }),
  parseHTML: () => [{ tag: 'div[data-extension-key="toc"]' }],
  renderHTML: () => ['div', { class: 'kk-toc', 'data-extension-key': 'toc' }, 'Table of contents (the heading list appears when the page is viewed)'],
});

export interface MediaAttrs {
  id: string;
  type: 'file';
  collection: string;
  __fileName?: string;
  alt?: string;
  width?: number;
  height?: number;
}
export const isImage = (a: Partial<MediaAttrs>) => /\.(png|jpe?g|gif|webp|svg|avif|bmp)$/i.test(a.__fileName ?? '');

export interface MediaOptions {
  resolve: (a: MediaAttrs) => Promise<string>;
}

export const MediaSingle = TNode.create({
  name: 'mediaSingle',
  group: 'block',
  content: 'media',
  selectable: true,
  addAttributes: () => ({ layout: { default: 'center' } }),
  parseHTML: () => [{ tag: 'figure[data-media-single]' }],
  renderHTML: ({ node }) => ['figure', { class: 'kk-media', 'data-media-single': '', 'data-layout': node.attrs.layout }, 0],
});

export const MediaGroup = TNode.create({
  name: 'mediaGroup',
  group: 'block',
  content: 'media+',
  parseHTML: () => [{ tag: 'div[data-media-group]' }],
  renderHTML: () => ['div', { class: 'kk-files', 'data-media-group': '' }, 0],
});

export const Media = TNode.create<MediaOptions>({
  name: 'media',
  atom: true,
  selectable: true,
  addOptions: () => ({ resolve: () => Promise.resolve('') }),
  addAttributes: () => ({ id: { default: null }, type: { default: 'file' }, collection: { default: '' }, alt: { default: null }, width: { default: null }, height: { default: null }, __fileName: { default: null } }),
  parseHTML: () => [{ tag: 'div[data-media-id]' }],
  renderHTML: ({ node }) => ['div', { 'data-media-id': node.attrs.id }, node.attrs.__fileName ?? 'File'],
  addNodeView() {
    return ({ node }) => {
      const a = node.attrs as MediaAttrs;
      const dom = document.createElement('div');
      dom.contentEditable = 'false';
      if (isImage(a)) {
        const img = document.createElement('img');
        img.className = 'kk-media__img';
        img.alt = a.alt ?? a.__fileName ?? '';
        void this.options.resolve(a).then((src) => src && (img.src = src));
        dom.className = 'kk-media__frame';
        dom.append(img);
      } else {
        dom.className = 'kk-file';
        dom.textContent = a.__fileName ?? 'File';
      }
      return { dom };
    };
  },
});
