import { Button, Dialog, TextField } from '@kaijukit/ui';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { createSpace } from './api';
import { suggestKey } from './format';

export function CreateSpaceDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [name, setName] = useState('');
  const [key, setKey] = useState('');
  const [keyTouched, setKeyTouched] = useState(false);
  const navigate = useNavigate();
  const qc = useQueryClient();
  const m = useMutation({
    mutationFn: () => createSpace(name.trim(), key.trim()),
    onSuccess: (s) => {
      void qc.invalidateQueries({ queryKey: ['spaces'] });
      onClose();
      navigate(`/wiki/spaces/${s.key}/overview`);
    },
  });
  useEffect(() => {
    if (open) {
      setName('');
      setKey('');
      setKeyTouched(false);
      m.reset();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- reset only when the dialog opens
  }, [open]);

  const keyInvalid = key !== '' && !/^[A-Za-z0-9]+$/.test(key);
  const err = m.error?.message;
  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Create space"
      onSubmit={() => name.trim() && key && !keyInvalid && m.mutate()}
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button type="submit" appearance="primary" loading={m.isPending} disabled={!name.trim() || !key || keyInvalid}>
            Create
          </Button>
        </>
      }
    >
      <TextField
        label="Space name"
        value={name}
        autoFocus
        required
        onChange={(e) => {
          setName(e.target.value);
          if (!keyTouched) setKey(suggestKey(e.target.value));
        }}
      />
      <TextField
        label="Space key"
        value={key}
        hint="Letters and numbers only. Used in the space address."
        error={keyInvalid ? 'Use letters and numbers only.' : err}
        onChange={(e) => {
          setKeyTouched(true);
          setKey(e.target.value.toUpperCase());
        }}
      />
    </Dialog>
  );
}
