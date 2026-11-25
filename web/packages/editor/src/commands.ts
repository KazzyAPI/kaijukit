import type { Editor } from '@tiptap/core';
import { NodeSelection } from '@tiptap/pm/state';
import { PANEL_TYPES } from './extensions';
import { expand, layoutSection, panel, toc } from './insert';

export interface Command {
  id: string;
  title: string;
  hint: string;
  keywords: string;
  run: (e: Editor) => void;
  can?: (e: Editor) => boolean;
  /** False when the host lacks what the command needs (uploads). */
  available?: boolean;
}
export interface Actions {
  status: () => void;
  date: () => void;
  file: (kind: 'image' | 'file') => void;
  canUpload: boolean;
}

const heading = (level: 1 | 2 | 3 | 4 | 5 | 6): Command => ({ id: `h${level}`, title: `Heading ${level}`, hint: `${'#'.repeat(level)} `, keywords: `h${level} heading title`, run: (e) => e.chain().focus().setHeading({ level }).run() });
// Inserting over a selected block node (a TOC, an image) would replace it, so go after it instead.
export const insert = (e: Editor, node: object | object[]) => {
  const { selection } = e.state;
  const chain = e.chain().focus();
  return (selection instanceof NodeSelection ? chain.insertContentAt(selection.to, node) : chain.insertContent(node)).run();
};

export function buildCommands(a: Actions): Command[] {
  return [
    { id: 'p', title: 'Paragraph', hint: 'Plain text', keywords: 'text paragraph', run: (e) => e.chain().focus().setParagraph().run() },
    heading(1), heading(2), heading(3), heading(4), heading(5), heading(6),
    { id: 'ul', title: 'Bulleted list', hint: '- ', keywords: 'bullet list unordered', run: (e) => e.chain().focus().toggleBulletList().run() },
    { id: 'ol', title: 'Numbered list', hint: '1. ', keywords: 'number ordered list', run: (e) => e.chain().focus().toggleOrderedList().run() },
    { id: 'task', title: 'Action item', hint: '[] ', keywords: 'task todo checkbox action', run: (e) => e.chain().focus().toggleList('taskList', 'taskItem').run() },
    { id: 'decision', title: 'Decision', hint: '<> ', keywords: 'decision list', run: (e) => e.chain().focus().toggleList('decisionList', 'decisionItem').run() },
    { id: 'quote', title: 'Quote', hint: '> ', keywords: 'blockquote quote', run: (e) => e.chain().focus().toggleBlockquote().run() },
    { id: 'code', title: 'Code snippet', hint: '```', keywords: 'code block snippet', run: (e) => e.chain().focus().toggleCodeBlock().run() },
    { id: 'rule', title: 'Divider', hint: '---', keywords: 'rule divider line hr', run: (e) => e.chain().focus().setHorizontalRule().run() },
    { id: 'table', title: 'Table', hint: '3 x 3', keywords: 'table grid', run: (e) => e.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(), can: (e) => !e.isActive('table') },
    ...PANEL_TYPES.map(
      (t): Command => ({
        id: `panel-${t}`,
        title: `${t[0]!.toUpperCase()}${t.slice(1)} panel`,
        hint: 'Highlighted block',
        keywords: `panel callout ${t}`,
        run: (e) => (e.isActive('panel') ? e.chain().focus().updateAttributes('panel', { panelType: t }).run() : insert(e, panel(t))),
      }),
    ),
    { id: 'expand', title: 'Expand', hint: 'Collapsible section', keywords: 'expand collapse details', run: (e) => insert(e, expand()) },
    { id: 'layout2', title: 'Two columns', hint: 'Layout', keywords: 'layout section columns two', run: (e) => insert(e, layoutSection(2)) },
    { id: 'layout3', title: 'Three columns', hint: 'Layout', keywords: 'layout section columns three', run: (e) => insert(e, layoutSection(3)) },
    { id: 'status', title: 'Status', hint: 'Colored lozenge', keywords: 'status lozenge label', run: a.status },
    { id: 'date', title: 'Date', hint: 'Pick a date', keywords: 'date calendar', run: a.date },
    { id: 'toc', title: 'Table of contents', hint: 'Heading list', keywords: 'toc table of contents macro', run: (e) => insert(e, toc()) },
    { id: 'image', title: 'Image', hint: 'Upload', keywords: 'image picture photo upload', run: () => a.file('image'), available: a.canUpload },
    { id: 'file', title: 'File', hint: 'Upload', keywords: 'file attachment upload', run: () => a.file('file'), available: a.canUpload },
  ];
}
