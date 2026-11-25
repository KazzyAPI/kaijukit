import { Button, Dialog, TextArea, TextField } from '@kaijukit/ui';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { type Space } from './api';
import { createBlog, createFolder } from './contentApi';

/** Creates a blog post (title and plain text; no rich editor for blogs yet) or a folder under the space home page. */
export function CreateKindDialog({ kind, space, onClose }: { kind: 'blog' | 'folder' | null; space: Space; onClose: () => void }) {
  const qc = useQueryClient();
  const navigate = useNavigate();
  const [title, setTitle] = useState('');
  const [text, setText] = useState('');
  const create = useMutation({
    mutationFn: () => (kind === 'blog' ? createBlog(space.id, title.trim(), text) : createFolder(space.id, title.trim(), space.homepageId)),
    onSuccess: (r) => {
      void qc.invalidateQueries({ queryKey: [kind === 'blog' ? 'blogs' : 'tree'] });
      close();
      if (kind === 'blog') navigate(`/wiki/spaces/${space.key}/blog/${r.id}`);
    },
  });
  const close = () => {
    setTitle('');
    setText('');
    create.reset();
    onClose();
  };
  const label = kind === 'blog' ? 'blog post' : 'folder';
  return (
    <Dialog
      open={!!kind}
      onClose={close}
      title={`Create ${label}`}
      onSubmit={() => title.trim() && create.mutate()}
      footer={
        <>
          <Button onClick={close}>Cancel</Button>
          <Button type="submit" appearance="primary" loading={create.isPending} disabled={!title.trim()}>Create</Button>
        </>
      }
    >
      <TextField label="Title" autoFocus value={title} onChange={(e) => setTitle(e.target.value)} />
      {kind === 'blog' && <TextArea label="Text" rows={6} hint="Plain text; separate paragraphs with a blank line." value={text} onChange={(e) => setText(e.target.value)} />}
      {create.error && <div role="alert" className="kk-error-box">{create.error.message}</div>}
    </Dialog>
  );
}
