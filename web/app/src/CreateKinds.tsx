import { Button, Dialog, TextField } from '@kaijukit/ui';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { type Space } from './api';
import { createFolder } from './contentApi';

/** Creates a folder under the space home page (no endpoint lists root-level folders). */
export function CreateFolderDialog({ open, space, onClose }: { open: boolean; space: Space; onClose: () => void }) {
  const qc = useQueryClient();
  const [title, setTitle] = useState('');
  const close = () => {
    setTitle('');
    create.reset();
    onClose();
  };
  const create = useMutation({
    mutationFn: () => createFolder(space.id, title.trim(), space.homepageId),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['tree'] });
      close();
    },
  });
  return (
    <Dialog
      open={open}
      onClose={close}
      title="Create folder"
      onSubmit={() => title.trim() && create.mutate()}
      footer={
        <>
          <Button onClick={close}>Cancel</Button>
          <Button type="submit" appearance="primary" loading={create.isPending} disabled={!title.trim()}>Create</Button>
        </>
      }
    >
      <TextField label="Title" autoFocus value={title} onChange={(e) => setTitle(e.target.value)} />
      {create.error && <div role="alert" className="kk-error-box">{create.error.message}</div>}
    </Dialog>
  );
}
