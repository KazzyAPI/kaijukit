import type { ReactNode } from 'react';
import type { AdfMark, AdfNode } from './adf';

const SAFE_HREF = /^(https?:|mailto:|tel:|\/|#)/i;

function applyMarks(text: ReactNode, marks: AdfMark[] = []): ReactNode {
  return marks.reduce<ReactNode>((acc, m) => {
    switch (m.type) {
      case 'strong': return <strong>{acc}</strong>;
      case 'em': return <em>{acc}</em>;
      case 'underline': return <u>{acc}</u>;
      case 'strike': return <s>{acc}</s>;
      case 'code': return <code>{acc}</code>;
      case 'link': {
        const href = String(m.attrs?.href ?? '');
        return SAFE_HREF.test(href) ? <a href={href} rel="noopener noreferrer nofollow">{acc}</a> : acc;
      }
      default: return acc;
    }
  }, text);
}

function kids(n: AdfNode): ReactNode {
  return n.content?.map((c, i) => <Node key={i} node={c} />);
}

function Node({ node: n }: { node: AdfNode }): ReactNode {
  const a = n.attrs ?? {};
  switch (n.type) {
    case 'text': return applyMarks(n.text, n.marks);
    case 'paragraph': return <p>{kids(n)}</p>;
    case 'heading': {
      const H = `h${Math.min(6, Math.max(1, Number(a.level) || 1))}` as 'h1';
      return <H>{kids(n)}</H>;
    }
    case 'bulletList': return <ul>{kids(n)}</ul>;
    case 'orderedList': return <ol start={typeof a.order === 'number' ? a.order : undefined}>{kids(n)}</ol>;
    case 'listItem': return <li>{kids(n)}</li>;
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
      return <span data-kk-node={n.type}>{INLINE(n)}</span>;
  }
}

const INLINE = (n: AdfNode) => (['text', 'shortName', 'title', 'url'].map((k) => n.attrs?.[k]).find((v) => typeof v === 'string') as string | undefined) ?? `[${n.type}]`;

export function AdfView({ doc }: { doc: AdfNode | null | undefined }) {
  return <div className="kk-doc">{doc?.content?.map((c, i) => <Node key={i} node={c} />)}</div>;
}
