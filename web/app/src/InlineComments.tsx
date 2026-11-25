import { Button, EmptyState, IconButton, Icon, Skeleton, useToast } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useEffect, useRef, useState, type RefObject } from 'react';
import { Composer, Thread, useCommentRefresh } from './Comments';
import { toCommentAdf } from './commentText';
import { createComment, pageComments, type Comment, type InlineAnchor } from './contentApi';

export const isOpen = (c: Comment) => c.resolutionStatus !== 'resolved';
export const useInlineComments = (pageId: string) => useQuery({ queryKey: ['comments', pageId, 'inline'], queryFn: () => pageComments(pageId, 'inline') });

export interface Pending {
  anchor: InlineAnchor;
}
type Found = { anchor: InlineAnchor } | { error: string };

const SINGLE_RUN = 'Select text inside one paragraph and one formatting style (for example, not across bold and plain text).';

function starts(text: string, needle: string): number[] {
  const out: number[] = [];
  for (let i = text.indexOf(needle); i !== -1; i = text.indexOf(needle, i + needle.length)) out.push(i);
  return out;
}

/** The server anchors by occurrence inside one stored text node; the rendered text nodes are those nodes in document order. */
export function anchorFor(root: HTMLElement, range: Range): Found {
  const { startContainer: node, endContainer: end } = range;
  const text = range.toString();
  if (!text.trim()) return { error: 'Select some text to comment on.' };
  if (node !== end || node.nodeType !== Node.TEXT_NODE || !root.contains(node) || (node.parentElement as HTMLElement).closest('[data-kk-atom]')) return { error: SINGLE_RUN };
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let total = 0;
  let before = 0;
  for (let t = walker.nextNode(); t; t = walker.nextNode()) {
    if ((t.parentElement as HTMLElement).closest('[data-kk-atom]')) continue;
    const data = (t as Text).data;
    const at = starts(data, text);
    if (t === node) {
      const i = at.indexOf(range.startOffset);
      if (i === -1) return { error: SINGLE_RUN };
      before = total + i;
    }
    total += at.length;
  }
  return { anchor: { textSelection: text, textSelectionMatchCount: total, textSelectionMatchIndex: before } };
}

/** Floating "Comment" button next to a text selection inside `root`, also on Ctrl/Cmd+Alt+M. */
export function SelectionButton({ root, onStart }: { root: RefObject<HTMLElement>; onStart: (p: Pending) => void }) {
  const toast = useToast();
  const [at, setAt] = useState<{ top: number; left: number; found: Found } | null>(null);
  useEffect(() => {
    const read = (): { range: Range; found: Found } | null => {
      const sel = getSelection();
      const el = root.current;
      if (!el || !sel || sel.isCollapsed || !sel.rangeCount) return null;
      const range = sel.getRangeAt(0);
      return el.contains(range.commonAncestorContainer) ? { range, found: anchorFor(el, range) } : null;
    };
    const place = () => {
      const r = read();
      if (!r) return setAt(null);
      const box = r.range.getBoundingClientRect();
      setAt({ top: box.top < 120 ? box.bottom + 8 : box.top - 44, left: Math.min(Math.max(8, box.left), window.innerWidth - 140), found: r.found });
    };
    const key = (e: KeyboardEvent) => {
      if (!(e.key.toLowerCase() === 'm' || e.code === 'KeyM') || !e.altKey || !(e.metaKey || e.ctrlKey)) return;
      const r = read();
      if (!r) return;
      e.preventDefault();
      if ('anchor' in r.found) onStart({ anchor: r.found.anchor });
      else toast({ title: "Can't comment on this selection", description: r.found.error, appearance: 'warning' });
    };
    document.addEventListener('selectionchange', place);
    window.addEventListener('scroll', place, true);
    window.addEventListener('resize', place);
    document.addEventListener('keydown', key);
    return () => {
      document.removeEventListener('selectionchange', place);
      window.removeEventListener('scroll', place, true);
      window.removeEventListener('resize', place);
      document.removeEventListener('keydown', key);
    };
  }, [root, onStart, toast]);
  if (!at) return null;
  return (
    <button
      type="button"
      className="kk-selbtn"
      style={{ top: at.top, left: at.left }}
      aria-keyshortcuts="Control+Alt+M Meta+Alt+M"
      onMouseDown={(e) => e.preventDefault()}
      onClick={() => {
        if ('anchor' in at.found) onStart({ anchor: at.found.anchor });
        else toast({ title: "Can't comment on this selection", description: at.found.error, appearance: 'warning' });
      }}
    >
      <Icon name="comment" size={14} /> Comment
    </button>
  );
}

export function InlinePanel({ pageId, pending, onDone, focusedId, onClose }: { pageId: string; pending: Pending | null; onDone: () => void; focusedId?: string; onClose: () => void }) {
  const q = useInlineComments(pageId);
  const refresh = useCommentRefresh(pageId);
  const [filter, setFilter] = useState<'open' | 'resolved'>('open');
  const followed = useRef<string>();
  useEffect(() => {
    const c = q.data?.find((x) => x.id === focusedId);
    if (c && followed.current !== c.id) {
      followed.current = c.id;
      setFilter(isOpen(c) ? 'open' : 'resolved');
      requestAnimationFrame(() => document.getElementById(`comment-${c.id}`)?.scrollIntoView({ block: 'nearest' }));
    }
  }, [focusedId, q.data]);
  const shown = (q.data ?? []).filter((c) => isOpen(c) === (filter === 'open'));
  const openCount = (q.data ?? []).filter(isOpen).length;

  return (
    <aside className="kk-sidepanel" aria-label="Inline comments">
      <div className="kk-sidepanel__head">
        <h2 className="kk-sidepanel__title">Inline comments</h2>
        <IconButton label="Close inline comments" icon={<Icon name="close" />} onClick={onClose} />
      </div>
      <div className="kk-segment" role="group" aria-label="Show">
        <Button aria-pressed={filter === 'open'} onClick={() => setFilter('open')}>Open ({openCount})</Button>
        <Button aria-pressed={filter === 'resolved'} onClick={() => setFilter('resolved')}>Resolved ({(q.data?.length ?? 0) - openCount})</Button>
      </div>
      {pending && (
        <div className="kk-sidepanel__new">
          <blockquote className="kk-quote">{pending.anchor.textSelection}</blockquote>
          <Composer
            label="New inline comment"
            autoFocus
            submitLabel="Comment"
            onCancel={onDone}
            submit={async (t) => {
              await createComment('inline', { pageId, body: toCommentAdf(t), anchor: pending.anchor });
              await refresh('inline');
              setFilter('open');
              onDone();
            }}
          />
        </div>
      )}
      {q.isPending && <Skeleton height={64} />}
      {q.isError && <p role="alert" className="kk-error-box">Couldn't load inline comments.</p>}
      {q.data && shown.length === 0 && !pending && (
        <EmptyState title={filter === 'open' ? 'No open inline comments' : 'Nothing resolved yet'} description={filter === 'open' ? 'Select text on the page and choose Comment.' : undefined} />
      )}
      <ul className="kk-comments" aria-label={filter === 'open' ? 'Open inline comments' : 'Resolved inline comments'}>
        {shown.map((c) => (
          <li key={c.id} className="kk-inline">
            <blockquote className="kk-quote">{c.properties?.inlineOriginalSelection ?? 'Selected text'}</blockquote>
            <ul className="kk-comments"><Thread kind="inline" comment={c} pageId={pageId} focusedId={focusedId} /></ul>
          </li>
        ))}
      </ul>
    </aside>
  );
}
