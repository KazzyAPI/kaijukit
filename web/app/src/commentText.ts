import type { AdfNode } from '@kaijukit/editor/adf';

export const toCommentAdf = (text: string): string =>
  JSON.stringify({
    version: 1,
    type: 'doc',
    content: text.split('\n').filter((l) => l.trim()).map((l) => ({ type: 'paragraph', content: [{ type: 'text', text: l }] })),
  });

/** True when the comment is plain paragraphs, so a textarea can edit it without losing formatting. */
export const isPlainComment = (doc: AdfNode | null | undefined): boolean =>
  !!doc?.content?.every((p) => p.type === 'paragraph' && (p.content ?? []).every((t) => t.type === 'text' && !t.marks?.length));

export const commentText = (doc: AdfNode | null | undefined): string =>
  (doc?.content ?? []).map((p) => (p.content ?? []).map((t) => t.text ?? '').join('')).join('\n');
