import { type Kind } from './api';
import { parseAdf } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { Avatar, Button, Dialog, type ButtonProps, Icon, Lozenge, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState, type ReactNode } from 'react';
import { useSearchParams } from 'react-router-dom';
import { commentReplies, createComment, deleteComment, editComment, pageComments, resolveComment, type Comment, type CommentKind } from './contentApi';
import { commentText, isPlainComment, toCommentAdf } from './commentText';
import { exactTime, relativeTime } from './format';
import { useMyId, useUserNames } from './users';

const MAX_REPLY_DEPTH = 2;

export function useCommentRefresh(pageId: string) {
  const qc = useQueryClient();
  return (kind: CommentKind) =>
    Promise.all([
      qc.invalidateQueries({ queryKey: ['comments', pageId] }),
      qc.invalidateQueries({ queryKey: ['replies'] }),
      ...(kind === 'inline' ? [qc.invalidateQueries({ queryKey: ['page', pageId] })] : []),
    ]);
}

export function Composer({ label, initial = '', submitLabel, submit, onCancel, autoFocus }: { label: string; initial?: string; submitLabel: string; submit: (text: string) => Promise<unknown>; onCancel: () => void; autoFocus?: boolean }) {
  const [text, setText] = useState(initial);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const send = async () => {
    if (!text.trim() || busy) return;
    setBusy(true);
    setError('');
    try {
      await submit(text);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Something went wrong.');
    } finally {
      setBusy(false);
    }
  };
  return (
    <form
      className="kk-composer"
      onSubmit={(e) => {
        e.preventDefault();
        void send();
      }}
    >
      <textarea
        className="kk-input kk-composer__input"
        aria-label={label}
        rows={3}
        autoFocus={autoFocus}
        value={text}
        placeholder="Write a comment..."
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            void send();
          } else if (e.key === 'Escape') {
            e.stopPropagation();
            onCancel();
          }
        }}
      />
      {error && <p role="alert" className="kk-error-box">{error}</p>}
      <div className="kk-row">
        <Button type="submit" appearance="primary" loading={busy} disabled={!text.trim()}>{submitLabel}</Button>
        <Button onClick={onCancel}>Cancel</Button>
      </div>
    </form>
  );
}

function Action({ icon, children, ...rest }: { icon: 'reply' | 'edit' | 'trash' | 'check'; children: ReactNode } & Omit<ButtonProps, 'children'>) {
  return (
    <Button appearance="subtle" className="kk-comment__action" {...rest}>
      <Icon name={icon} size={14} />
      {children}
    </Button>
  );
}

export function Thread({ kind, comment, pageId, depth = 0, focusedId }: { kind: CommentKind; comment: Comment; pageId: string; depth?: number; focusedId?: string }) {
  const refresh = useCommentRefresh(pageId);
  const toast = useToast();
  const me = useMyId();
  const authorId = comment.authorId ?? comment.version.authorId;
  const name = useUserNames([authorId]);
  const replies = useQuery({ queryKey: ['replies', kind, comment.id], queryFn: () => commentReplies(kind, comment.id) });
  const [mode, setMode] = useState<'view' | 'edit' | 'reply'>('view');
  const [confirm, setConfirm] = useState(false);
  const doc = parseAdf(comment.body?.atlas_doc_format?.value);
  const plain = isPlainComment(doc);
  const status = comment.resolutionStatus;
  const resolved = status === 'resolved';
  const own = !me || authorId === me;
  const fail = (title: string) => (e: Error) => toast({ title, description: e.message, appearance: 'error' });

  const resolve = useMutation({
    mutationFn: () => resolveComment(comment.id, !resolved),
    onSuccess: () => refresh('inline'),
    onError: fail(resolved ? "Couldn't reopen the comment" : "Couldn't resolve the comment"),
  });
  const remove = useMutation({
    mutationFn: () => deleteComment(kind, comment.id),
    onSuccess: async () => {
      setConfirm(false);
      await refresh(kind);
      toast({ title: 'Comment deleted', appearance: 'success' });
    },
    onError: fail("Couldn't delete the comment"),
  });
  const kids = replies.data ?? [];

  return (
    <li className="kk-comment" id={`comment-${comment.id}`} data-focused={focusedId === comment.id || undefined}>
      <div className="kk-comment__head">
        <Avatar name={name(authorId)} size="sm" />
        <strong>{name(authorId)}</strong>
        <time className="kk-muted" dateTime={comment.version.createdAt} title={exactTime(comment.version.createdAt)}>{relativeTime(comment.version.createdAt)}</time>
        {comment.version.number > 1 && <span className="kk-muted">edited</span>}
        {resolved && <Lozenge appearance="success">Resolved</Lozenge>}
        {status === 'dangling' && <Lozenge appearance="warning">Text was deleted</Lozenge>}
      </div>
      {mode === 'edit' ? (
        <Composer
          label="Edit comment"
          autoFocus
          initial={commentText(doc)}
          submitLabel="Save"
          onCancel={() => setMode('view')}
          submit={async (t) => {
            await editComment(kind, comment, toCommentAdf(t));
            await refresh(kind);
            setMode('view');
          }}
        />
      ) : (
        <div className="kk-comment__body"><AdfView doc={doc} /></div>
      )}
      {mode === 'view' && (
        <div className="kk-comment__actions">
          {depth < MAX_REPLY_DEPTH && <Action icon="reply" onClick={() => setMode('reply')} aria-label={`Reply to ${name(authorId)}`}>Reply</Action>}
          {own && <Action icon="edit" disabled={!plain} title={plain ? undefined : 'This comment has formatting that the comment box cannot edit yet'} onClick={() => setMode('edit')}>Edit</Action>}
          {own && <Action icon="trash" onClick={() => setConfirm(true)}>Delete</Action>}
          {kind === 'inline' && depth === 0 && status !== 'dangling' && (
            <Action icon="check" loading={resolve.isPending} onClick={() => resolve.mutate()}>{resolved ? 'Reopen' : 'Resolve'}</Action>
          )}
        </div>
      )}
      {mode === 'reply' && (
        <Composer
          label="Reply"
          autoFocus
          submitLabel="Reply"
          onCancel={() => setMode('view')}
          submit={async (t) => {
            await createComment(kind, { parentCommentId: comment.id, body: toCommentAdf(t) });
            await refresh(kind);
            setMode('view');
          }}
        />
      )}
      {replies.isError && <p role="alert" className="kk-muted">Couldn't load replies.</p>}
      {kids.length > 0 && (
        <ul className="kk-comments kk-comments--replies" aria-label={`Replies to ${name(authorId)}`}>
          {kids.map((c) => <Thread key={c.id} kind={kind} comment={c} pageId={pageId} depth={depth + 1} focusedId={focusedId} />)}
        </ul>
      )}
      <Dialog
        open={confirm}
        onClose={() => setConfirm(false)}
        title="Delete this comment?"
        footer={
          <>
            <Button onClick={() => setConfirm(false)}>Cancel</Button>
            <Button appearance="danger" loading={remove.isPending} onClick={() => remove.mutate()}>Delete</Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>{kids.length ? `Its ${kids.length} ${kids.length === 1 ? 'reply is' : 'replies are'} deleted with it.` : 'This cannot be undone.'}</p>
      </Dialog>
    </li>
  );
}

export function FooterComments({ pageId, of = 'page' }: { pageId: string; of?: Kind }) {
  const refresh = useCommentRefresh(pageId);
  const q = useQuery({ queryKey: ['comments', pageId, 'footer'], queryFn: () => pageComments(pageId, 'footer', of) });
  const [open, setOpen] = useState(false);
  const focus = useSearchParams()[0].get('focusedCommentId');
  useEffect(() => {
    if (q.data && focus) requestAnimationFrame(() => document.getElementById(`comment-${focus}`)?.scrollIntoView({ block: 'center' }));
  }, [q.data, focus]);
  return (
    <section className="kk-section" aria-labelledby="comments-h">
      <h2 id="comments-h" className="kk-section__title">Comments{q.data ? ` (${q.data.length})` : ''}</h2>
      {q.isPending && <Skeleton height={48} />}
      {q.isError && (
        <p role="alert" className="kk-error-box">
          Couldn't load comments. <Button onClick={() => void q.refetch()}>Retry</Button>
        </p>
      )}
      {q.data?.length === 0 && <p className="kk-muted">No comments yet.</p>}
      {q.data && q.data.length > 0 && (
        <ul className="kk-comments" aria-label="Comments">
          {q.data.map((c) => <Thread key={c.id} kind="footer" comment={c} pageId={pageId} />)}
        </ul>
      )}
      {open ? (
        <Composer
          label="Add a comment"
          autoFocus
          submitLabel="Save"
          onCancel={() => setOpen(false)}
          submit={async (t) => {
            await createComment('footer', { pageId, of, body: toCommentAdf(t) });
            await refresh('footer');
            setOpen(false);
          }}
        />
      ) : (
        <button type="button" className="kk-input kk-composer__stub" onClick={() => setOpen(true)}>Write a comment...</button>
      )}
    </section>
  );
}
