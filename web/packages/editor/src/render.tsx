import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import type { AdfMark, AdfNode } from './adf';
import { formatDate, isImage, safeColor, type MediaAttrs } from './extensions';

const SAFE_HREF = /^(https?:|mailto:|tel:|\/|#)/i;

export interface Annotations {
  /** Ids of anchors drawn as highlights; any other annotation renders as plain text. */
  open: ReadonlySet<string>;
  onSelect: (id: string) => void;
}

interface ViewCtx {
  resolveMedia?: (a: MediaAttrs) => Promise<string>;
  headings: { n: number; level: number; text: string }[];
  annotations?: Annotations;
}
const Ctx = createContext<ViewCtx>({ headings: [] });

function applyMarks(text: ReactNode, marks: AdfMark[] = [], ann?: Annotations): ReactNode {
  return marks.reduce<ReactNode>((acc, m) => {
    switch (m.type) {
      case 'strong': return <strong>{acc}</strong>;
      case 'em': return <em>{acc}</em>;
      case 'underline': return <u>{acc}</u>;
      case 'strike': return <s>{acc}</s>;
      case 'code': return <code>{acc}</code>;
      case 'textColor': return <span style={{ color: safeColor(m.attrs?.color) }}>{acc}</span>;
      case 'backgroundColor': return <span className="kk-hl" style={{ backgroundColor: safeColor(m.attrs?.color) }}>{acc}</span>;
      case 'annotation': {
        const id = String(m.attrs?.id ?? '');
        if (!ann) return <span className="kk-annotation" data-annotation-id={id}>{acc}</span>;
        if (!ann.open.has(id)) return acc;
        return (
          <mark className="kk-annotation" data-annotation-id={id} role="button" tabIndex={0} title="Open comment" onClick={() => ann.onSelect(id)} onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), ann.onSelect(id))}>
            {acc}
          </mark>
        );
      }
      case 'link': {
        const href = String(m.attrs?.href ?? '');
        return SAFE_HREF.test(href) ? <a href={href} rel="noopener noreferrer nofollow">{acc}</a> : acc;
      }
      default: return acc;
    }
  }, text);
}

function blockProps(n: AdfNode) {
  const p: { 'data-align'?: string; 'data-indent'?: number } = {};
  for (const m of n.marks ?? []) {
    if (m.type === 'alignment' && (m.attrs?.align === 'center' || m.attrs?.align === 'end')) p['data-align'] = m.attrs.align;
    if (m.type === 'indentation') p['data-indent'] = Math.min(6, Math.max(0, Number(m.attrs?.level) || 0));
  }
  return p;
}

function kids(n: AdfNode): ReactNode {
  return n.content?.map((c, i) => <Node key={i} node={c} />);
}

const textOf = (n: AdfNode): string => n.text ?? (n.content ?? []).map(textOf).join('');
export function headingsOf(doc: AdfNode | null | undefined): ViewCtx['headings'] {
  const out: ViewCtx['headings'] = [];
  doc?.content?.forEach((n) => n.type === 'heading' && out.push({ n: out.length, level: Math.min(6, Math.max(1, Number(n.attrs?.level) || 1)), text: textOf(n) }));
  return out;
}

function MediaView({ a }: { a: MediaAttrs }) {
  const { resolveMedia } = useContext(Ctx);
  const [src, setSrc] = useState('');
  const image = isImage(a);
  useEffect(() => {
    let stale = false;
    if (image) void resolveMedia?.(a).then((u) => !stale && setSrc(u), () => undefined);
    return () => {
      stale = true;
    };
  }, [a, image, resolveMedia]);
  if (image) return src ? <img className="kk-media__img" src={src} alt={a.alt ?? a.__fileName ?? ''} /> : <span className="kk-media__img kk-media__img--pending" role="img" aria-label={a.__fileName ?? 'Image'} />;
  return <span className="kk-file" data-kk-atom="">{a.__fileName ?? 'File'}</span>;
}

function Toc() {
  const { headings } = useContext(Ctx);
  if (!headings.length) return <div className="kk-toc" data-kk-atom="">Table of contents: this page has no headings.</div>;
  return (
    <nav className="kk-toc" data-kk-atom="" aria-label="Table of contents">
      <ul>
        {headings.map((h) => (
          <li key={h.n} style={{ marginLeft: (h.level - 1) * 16 }}>
            <a href={`#kk-h-${h.n}`}>{h.text || 'Untitled heading'}</a>
          </li>
        ))}
      </ul>
    </nav>
  );
}

function Node({ node: n }: { node: AdfNode }): ReactNode {
  const a = n.attrs ?? {};
  const { annotations } = useContext(Ctx);
  switch (n.type) {
    case 'text': return applyMarks(n.text, n.marks, annotations);
    case 'paragraph': return <p {...blockProps(n)}>{kids(n)}</p>;
    case 'heading': {
      const H = `h${Math.min(6, Math.max(1, Number(a.level) || 1))}` as 'h1';
      return <H id={a.__anchor as string | undefined} {...blockProps(n)}>{kids(n)}</H>;
    }
    case 'bulletList': return <ul>{kids(n)}</ul>;
    case 'orderedList': return <ol start={typeof a.order === 'number' ? a.order : undefined}>{kids(n)}</ol>;
    case 'listItem': return <li>{kids(n)}</li>;
    case 'taskList': return <ul className="kk-items" data-type="taskList">{kids(n)}</ul>;
    case 'taskItem': return <li className="kk-task" data-state={String(a.state)}><input type="checkbox" checked={a.state === 'DONE'} readOnly disabled aria-label={a.state === 'DONE' ? 'Done' : 'Not done'} /><div className="kk-task__body">{kids(n)}</div></li>;
    case 'decisionList': return <ul className="kk-items" data-type="decisionList">{kids(n)}</ul>;
    case 'decisionItem': return <li data-type="decisionItem"><div>{kids(n)}</div></li>;
    case 'panel': return <div className="kk-panel" data-panel-type={String(a.panelType ?? 'info')} role="group" aria-label={`${String(a.panelType ?? 'info')} panel`}>{kids(n)}</div>;
    case 'expand': return <details className="kk-expand kk-expand--view" open><summary data-kk-atom="">{String(a.title || 'Expand')}</summary>{kids(n)}</details>;
    case 'layoutSection': return <div className="kk-layout">{kids(n)}</div>;
    case 'layoutColumn': return <div className="kk-layout__col" style={{ flex: `${Number(a.width) || 50} 1 0` }}>{kids(n)}</div>;
    case 'mediaSingle': return <figure className="kk-media" data-layout={String(a.layout ?? 'center')}>{kids(n)}</figure>;
    case 'mediaGroup': return <div className="kk-files">{kids(n)}</div>;
    case 'media': return <MediaView a={a as unknown as MediaAttrs} />;
    case 'mention': return <span className="kk-mention" data-kk-atom="" data-mention-id={String(a.id ?? '')}>{String(a.text ?? '@unknown')}</span>;
    case 'emoji': return <span className="kk-emoji" data-kk-atom="" role="img" aria-label={String(a.shortName ?? '').replace(/:/g, '')}>{String(a.text ?? a.shortName ?? '')}</span>;
    case 'status': return <span data-kk-atom="" className={`kk-status kk-status--${String(a.color ?? 'neutral')}`}>{String(a.text ?? '').toUpperCase()}</span>;
    case 'date': return <time className="kk-date" data-kk-atom="" dateTime={new Date(Number(a.timestamp)).toISOString().slice(0, 10)}>{formatDate(a.timestamp)}</time>;
    case 'extension': return a.extensionKey === 'toc' ? <Toc /> : <span data-kk-atom="" data-kk-node="extension">[extension]</span>;
    case 'codeBlock': return <pre tabIndex={0} data-language={a.language as string | undefined}><code>{kids(n)}</code></pre>;
    case 'blockquote': return <blockquote>{kids(n)}</blockquote>;
    case 'rule': return <hr />;
    case 'hardBreak': return <br />;
    case 'table': return <div className="kk-doc__table"><table><tbody>{kids(n)}</tbody></table></div>;
    case 'tableRow': return <tr>{kids(n)}</tr>;
    case 'tableHeader': return <th colSpan={a.colspan as number | undefined} rowSpan={a.rowspan as number | undefined}>{kids(n)}</th>;
    case 'tableCell': return <td colSpan={a.colspan as number | undefined} rowSpan={a.rowspan as number | undefined}>{kids(n)}</td>;
    default:
      if (n.content?.length) return <div data-kk-node={n.type}>{kids(n)}</div>;
      return <span data-kk-atom="" data-kk-node={n.type}>{INLINE(n)}</span>;
  }
}

const INLINE = (n: AdfNode) => (['text', 'shortName', 'title', 'url'].map((k) => n.attrs?.[k]).find((v) => typeof v === 'string') as string | undefined) ?? `[${n.type}]`;

export function AdfView({ doc, resolveMedia, annotations }: { doc: AdfNode | null | undefined; resolveMedia?: (a: MediaAttrs) => Promise<string>; annotations?: Annotations }) {
  const headings = headingsOf(doc);
  let h = 0;
  return (
    <Ctx.Provider value={{ resolveMedia, headings, annotations }}>
      <div className="kk-doc">
        {doc?.content?.map((c, i) => <Node key={i} node={c.type === 'heading' ? { ...c, attrs: { ...c.attrs, __anchor: `kk-h-${h++}` } } : c} />)}
      </div>
    </Ctx.Provider>
  );
}
