export interface AdfMark {
  type: string;
  attrs?: Record<string, unknown>;
}
export interface AdfNode {
  type: string;
  attrs?: Record<string, unknown>;
  content?: AdfNode[];
  marks?: AdfMark[];
  text?: string;
}
export interface AdfDoc extends AdfNode {
  version: 1;
  type: 'doc';
  content: AdfNode[];
}

export const emptyDoc = (): AdfDoc => ({ version: 1, type: 'doc', content: [{ type: 'paragraph' }] });

const markToAdf: Record<string, string> = { bold: 'strong', italic: 'em', underline: 'underline', strike: 'strike', code: 'code', link: 'link' };
const markFromAdf = Object.fromEntries(Object.entries(markToAdf).map(([a, b]) => [b, a]));
const nodeToAdf: Record<string, string> = { horizontalRule: 'rule' };
const nodeFromAdf: Record<string, string> = { rule: 'horizontalRule' };
const KNOWN = new Set(['doc', 'paragraph', 'heading', 'text', 'bulletList', 'orderedList', 'listItem', 'codeBlock', 'blockquote', 'rule', 'hardBreak', 'table', 'tableRow', 'tableHeader', 'tableCell']);
const INLINE_TEXT_ATTRS = ['text', 'shortName', 'title', 'url'];

type PM = { type: string; attrs?: Record<string, unknown>; content?: PM[]; marks?: PM[]; text?: string };

function convertAttrs(pm: PM): Record<string, unknown> | undefined {
  const a = pm.attrs ?? {};
  switch (pm.type) {
    case 'heading': return { level: a.level };
    case 'orderedList': return typeof a.start === 'number' && a.start !== 1 ? { order: a.start } : undefined;
    case 'codeBlock': return a.language ? { language: a.language } : undefined;
    case 'tableHeader':
    case 'tableCell': {
      const o: Record<string, unknown> = {};
      if (Number(a.colspan) > 1) o.colspan = a.colspan;
      if (Number(a.rowspan) > 1) o.rowspan = a.rowspan;
      return Object.keys(o).length ? o : undefined;
    }
  }
}

function nodeToAdfNode(pm: PM): AdfNode {
  const out: AdfNode = { type: nodeToAdf[pm.type] ?? pm.type };
  const attrs = convertAttrs(pm);
  if (attrs) out.attrs = attrs;
  if (pm.type === 'text') {
    out.text = pm.text ?? '';
    if (pm.marks?.length) out.marks = pm.marks.map((m) => (m.type === 'link' ? { type: 'link', attrs: { href: m.attrs?.href } } : { type: markToAdf[m.type] ?? m.type }));
  } else if (pm.content?.length) out.content = pm.content.map(nodeToAdfNode);
  return out;
}

export function toAdf(pmDoc: PM): AdfDoc {
  const n = nodeToAdfNode(pmDoc);
  return { version: 1, type: 'doc', content: n.content ?? [{ type: 'paragraph' }] };
}

const isInline = (n: AdfNode) => ['text', 'hardBreak', 'mention', 'emoji', 'status', 'date', 'inlineCard', 'mediaInline', 'inlineExtension', 'placeholder'].includes(n.type);

function nodeFromAdfNode(n: AdfNode): PM[] {
  if (n.type === 'text') {
    const marks = (n.marks ?? []).flatMap((m): PM[] => {
      const t = markFromAdf[m.type];
      return t ? [t === 'link' ? { type: t, attrs: { href: m.attrs?.href } } : { type: t }] : [];
    });
    return n.text ? [{ type: 'text', text: n.text, ...(marks.length ? { marks } : {}) }] : [];
  }
  if (!KNOWN.has(n.type)) {
    const kids = (n.content ?? []).flatMap(nodeFromAdfNode);
    if (kids.length) return kids;
    const label = INLINE_TEXT_ATTRS.map((k) => n.attrs?.[k]).find((v) => typeof v === 'string') as string | undefined;
    const text = label ?? `[${n.type}]`;
    const t: PM = { type: 'text', text };
    return isInline(n) ? [t] : [{ type: 'paragraph', content: [t] }];
  }
  const out: PM = { type: nodeFromAdf[n.type] ?? n.type };
  const a = n.attrs ?? {};
  if (n.type === 'heading') out.attrs = { level: a.level };
  if (n.type === 'orderedList') out.attrs = { start: a.order ?? 1 };
  if (n.type === 'codeBlock') out.attrs = { language: a.language ?? null };
  if (n.type === 'tableHeader' || n.type === 'tableCell') out.attrs = { colspan: a.colspan ?? 1, rowspan: a.rowspan ?? 1 };
  if (n.content?.length) out.content = n.content.flatMap(nodeFromAdfNode);
  return [out];
}

export function fromAdf(doc: AdfNode | null | undefined): PM {
  const content = (doc?.content ?? []).flatMap(nodeFromAdfNode);
  return { type: 'doc', content: content.length ? content : [{ type: 'paragraph' }] };
}

/** Node types the editor cannot represent yet; they are flattened on edit. */
export function unsupportedTypes(doc: AdfNode | null | undefined): string[] {
  const seen = new Set<string>();
  const walk = (n: AdfNode) => {
    if (!KNOWN.has(n.type) && n.type !== 'rule') seen.add(n.type);
    n.content?.forEach(walk);
  };
  doc?.content?.forEach(walk);
  return [...seen];
}

export function parseAdf(value: string | undefined | null): AdfDoc | null {
  if (!value) return null;
  try {
    const d = JSON.parse(value) as AdfDoc;
    return d?.type === 'doc' ? d : null;
  } catch {
    return null;
  }
}

/** Words in the visible text of a document; block boundaries separate words. */
export function wordCount(doc: AdfNode | null | undefined): number {
  const parts: string[] = [];
  const walk = (n: AdfNode) => {
    if (n.text) parts.push(n.text);
    else if (n.content) {
      n.content.forEach(walk);
      parts.push(' ');
    }
  };
  if (doc) walk(doc);
  return parts.join('').split(/\s+/).filter(Boolean).length;
}
