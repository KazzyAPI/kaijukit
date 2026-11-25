import { Button, Dialog, Icon, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import { ApiError, authedFetch, resolveMedia, uploadAttachment, type Kind } from './api';
import { deleteAttachment, pageAttachments, type Attachment } from './contentApi';
import { relativeTime } from './format';
import { useUserNames } from './users';

const size = (n: number) => (n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(1)} MB`);

function Thumb({ a, pageId }: { a: Attachment; pageId: string }) {
  const [src, setSrc] = useState('');
  const image = /^image\/(png|jpeg|gif|webp)$/.test(a.mediaType);
  useEffect(() => {
    if (image) void resolveMedia({ id: a.id, type: 'file', collection: `contentId-${pageId}`, __fileName: a.title }).then(setSrc, () => undefined);
  }, [a.id, a.title, image, pageId]);
  return src ? <img className="kk-att__thumb" src={src} alt="" /> : <span className="kk-att__thumb kk-att__thumb--icon"><Icon name="page" /></span>;
}

async function download(a: Attachment) {
  const res = await authedFetch(`/wiki${a.downloadLink}`);
  if (!res.ok) throw new ApiError(res.status, 'The file could not be downloaded.');
  const url = URL.createObjectURL(await res.blob());
  const link = Object.assign(document.createElement('a'), { href: url, download: a.title });
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function Attachments({ pageId, of = 'page' }: { pageId: string; of?: Kind }) {
  const qc = useQueryClient();
  const toast = useToast();
  const input = useRef<HTMLInputElement>(null);
  const q = useQuery({ queryKey: ['attachments', pageId], queryFn: () => pageAttachments(pageId, of) });
  const name = useUserNames(q.data?.map((a) => a.version.authorId) ?? []);
  const [confirm, setConfirm] = useState<Attachment | null>(null);
  const [drag, setDrag] = useState(false);
  const refresh = () => qc.invalidateQueries({ queryKey: ['attachments', pageId] });

  const upload = useMutation({
    mutationFn: async (files: File[]) => {
      for (const f of files) await uploadAttachment(pageId, f);
    },
    onSuccess: refresh,
    onError: (e) => {
      toast({ title: "Couldn't upload the file", description: e.message, appearance: 'error' });
      void refresh();
    },
  });
  const remove = useMutation({
    mutationFn: (a: Attachment) => deleteAttachment(a.id),
    onSuccess: async () => {
      setConfirm(null);
      await refresh();
      toast({ title: 'Attachment deleted', appearance: 'success' });
    },
    onError: (e) => toast({ title: "Couldn't delete the attachment", description: e.message, appearance: 'error' }),
  });
  const pick = (files: FileList | null) => files?.length && upload.mutate(Array.from(files));

  return (
    <section
      className="kk-section kk-att"
      data-drag={drag || undefined}
      aria-labelledby="att-h"
      onDragOver={(e) => {
        e.preventDefault();
        setDrag(true);
      }}
      onDragLeave={() => setDrag(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDrag(false);
        pick(e.dataTransfer.files);
      }}
    >
      <div className="kk-row kk-row--between">
        <h2 id="att-h" className="kk-section__title">Attachments{q.data ? ` (${q.data.length})` : ''}</h2>
        <Button loading={upload.isPending} onClick={() => input.current?.click()}>
          <Icon name="upload" /> Upload files
        </Button>
        <input ref={input} type="file" multiple hidden aria-label="Choose files to attach" onChange={(e) => { pick(e.target.files); e.target.value = ''; }} />
      </div>
      {q.isPending && <Skeleton height={48} />}
      {q.isError && <p role="alert" className="kk-error-box">Couldn't load attachments.</p>}
      {q.data?.length === 0 && <p className="kk-muted">No attachments. Choose files or drop them here.</p>}
      {q.data && q.data.length > 0 && (
        <ul className="kk-att__list" aria-label="Attachments">
          {q.data.map((a) => (
            <li key={a.id} className="kk-att__row">
              <Thumb a={a} pageId={pageId} />
              <div className="kk-att__meta">
                <strong className="kk-att__name">{a.title}</strong>
                <span className="kk-muted">
                  {size(a.fileSize)} · v{a.version.number} · {name(a.version.authorId)} · {relativeTime(a.version.createdAt)}
                </span>
              </div>
              <Button aria-label={`Download ${a.title}`} onClick={() => download(a).catch((e: Error) => toast({ title: "Couldn't download the file", description: e.message, appearance: 'error' }))}>
                <Icon name="download" /> <span className="kk-att__btntext">Download</span>
              </Button>
              <Button appearance="subtle" aria-label={`Delete ${a.title}`} onClick={() => setConfirm(a)}>
                <Icon name="trash" />
              </Button>
            </li>
          ))}
        </ul>
      )}
      <Dialog
        open={!!confirm}
        onClose={() => setConfirm(null)}
        title="Delete this attachment?"
        footer={
          <>
            <Button onClick={() => setConfirm(null)}>Cancel</Button>
            <Button appearance="danger" loading={remove.isPending} onClick={() => confirm && remove.mutate(confirm)}>Delete</Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>"{confirm?.title}" is removed from this page. Images that use it in the page body stop showing.</p>
      </Dialog>
    </section>
  );
}
