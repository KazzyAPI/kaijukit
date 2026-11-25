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

const markToAdf: Record<string, string> = { bold: 'strong', italic: 'em', underline: 'underline', strike: 'strike', code: 'code', link: 'link', textColor: 'textColor', backgroundColor: 'backgroundColor', annotation: 'annotation' };
const markFromAdf = Object.fromEntries(Object.entries(markToAdf).map(([a, b]) => [b, a]));
const markAttrs: Record<string, string[]> = { link: ['href'], textColor: ['color'], backgroundColor: ['color'], annotation: ['id', 'annotationType'] };
const nodeToAdf: Record<string, string> = { horizontalRule: 'rule' };
const nodeFromAdf: Record<string, string> = { rule: 'horizontalRule' };
// Nodes whose editor type and attributes carry the same names as ADF.
const PASS: Record<string, string[]> = {
  taskList: ['localId'],
  taskItem: ['localId', 'state'],
  decisionList: ['localId'],
  decisionItem: ['localId', 'state'],
  panel: ['panelType'],
  expand: ['title'],
  layoutColumn: ['width'],
  mediaSingle: ['layout'],
  media: ['id', 'type', 'collection', 'alt', 'width', 'height', '__fileName'],
  mention: ['id', 'text', 'accessLevel'],
  emoji: ['shortName', 'id', 'text'],
  status: ['text', 'color', 'localId'],
  date: ['timestamp'],
  extension: ['extensionType', 'extensionKey', 'parameters', 'layout'],
};
const ITEMS = new Set(['taskItem', 'decisionItem']);
const KNOWN = new Set(['doc', 'paragraph', 'heading', 'text', 'bulletList', 'orderedList', 'listItem', 'codeBlock', 'blockquote', 'rule', 'hardBreak', 'table', 'tableRow', 'tableHeader', 'tableCell', 'layoutSection', 'mediaGroup', ...Object.keys(PASS)]);
const known = (n: AdfNode) => KNOWN.has(n.type) && (n.type !== 'extension' || n.attrs?.extensionKey === 'toc');
const INLINE_TEXT_ATTRS = ['text', 'shortName', 'title', 'url'];

type PM = { type: string; attrs?: Record<string, unknown>; content?: PM[]; marks?: PM[]; text?: string };

const pick = (a: Record<string, unknown>, keys: string[]) => {
  const o: Record<string, unknown> = {};
  for (const k of keys) if (a[k] !== undefined && a[k] !== null) o[k] = a[k];
  return o;
};

function convertAttrs(pm: PM): Record<string, unknown> | undefined {
  const a = pm.attrs ?? {};
  const pass = PASS[pm.type];
  if (pass) return pick(a, pass);
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

function blockMarks(pm: PM): AdfMark[] | undefined {
  const a = pm.attrs ?? {};
  const marks: AdfMark[] = [];
  if (a.align) marks.push({ type: 'alignment', attrs: { align: a.align } });
  if (Number(a.indent) > 0) marks.push({ type: 'indentation', attrs: { level: a.indent } });
  return marks.length ? marks : undefined;
}

function nodeToAdfNode(pm: PM): AdfNode {
  const out: AdfNode = { type: nodeToAdf[pm.type] ?? pm.type };
  const attrs = convertAttrs(pm);
  if (attrs) out.attrs = attrs;
  if (pm.type === 'text') {
    out.text = pm.text ?? '';
    if (pm.marks?.length) out.marks = pm.marks.map((m) => ({ type: markToAdf[m.type] ?? m.type, ...(markAttrs[m.type] ? { attrs: pick(m.attrs ?? {}, markAttrs[m.type]!) } : {}) }));
    return out;
  }
  if (pm.type === 'paragraph' || pm.type === 'heading') {
    const marks = blockMarks(pm);
    if (marks) out.marks = marks;
  }
  const kids = ITEMS.has(pm.type) ? pm.content?.[0]?.content : pm.content;
  if (kids?.length) out.content = kids.map(nodeToAdfNode);
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
      if (!t) return [];
      const attrs = markAttrs[t] ? pick(m.attrs ?? {}, markAttrs[t]!) : undefined;
      return [attrs ? { type: t, attrs } : { type: t }];
    });
    return n.text ? [{ type: 'text', text: n.text, ...(marks.length ? { marks } : {}) }] : [];
  }
  if (!known(n)) {
    const kids = (n.content ?? []).flatMap(nodeFromAdfNode);
    if (kids.length) return kids;
    const label = INLINE_TEXT_ATTRS.map((k) => n.attrs?.[k]).find((v) => typeof v === 'string') as string | undefined;
    const text = label ?? `[${n.type}]`;
    const t: PM = { type: 'text', text };
    return isInline(n) ? [t] : [{ type: 'paragraph', content: [t] }];
  }
  const out: PM = { type: nodeFromAdf[n.type] ?? n.type };
  const a = n.attrs ?? {};
  if (PASS[n.type]) out.attrs = pick(a, PASS[n.type]!);
  if (n.type === 'heading') out.attrs = { level: a.level };
  if (n.type === 'orderedList') out.attrs = { start: a.order ?? 1 };
  if (n.type === 'codeBlock') out.attrs = { language: a.language ?? null };
  if (n.type === 'tableHeader' || n.type === 'tableCell') out.attrs = { colspan: a.colspan ?? 1, rowspan: a.rowspan ?? 1 };
  if (n.type === 'paragraph' || n.type === 'heading') {
    for (const m of n.marks ?? []) {
      if (m.type === 'alignment' && m.attrs?.align) out.attrs = { ...out.attrs, align: m.attrs.align };
      if (m.type === 'indentation' && m.attrs?.level) out.attrs = { ...out.attrs, indent: m.attrs.level };
    }
  }
  const kids = (n.type === 'taskList' ? (n.content ?? []).flatMap((c) => (c.type === 'taskList' ? (c.content ?? []) : [c])) : n.content ?? []).flatMap(nodeFromAdfNode);
  if (ITEMS.has(n.type)) out.content = [{ type: 'paragraph', ...(kids.length ? { content: kids } : {}) }];
  else if (kids.length) out.content = kids;
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
    if (!known(n)) seen.add(n.type);
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
