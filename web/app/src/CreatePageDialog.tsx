import { Button, Dialog } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { listSpaces } from './api';

export function CreatePageDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces(), enabled: open });
  const [key, setKey] = useState('');
  const navigate = useNavigate();
  const chosen = key || spaces.data?.[0]?.key;
  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Create page"
      onSubmit={() => {
        if (!chosen) return;
        onClose();
        navigate(`/wiki/spaces/${chosen}/pages/create`);
      }}
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button type="submit" appearance="primary" disabled={!chosen}>
            Create
          </Button>
        </>
      }
    >
      {spaces.data?.length === 0 ? (
        <p style={{ margin: 0 }}>Create a space first. Pages live inside spaces.</p>
      ) : (
        <div className="kk-field">
          <label className="kk-field__label" htmlFor="kk-create-space">
            Space
          </label>
          <select id="kk-create-space" className="kk-input" value={chosen ?? ''} onChange={(e) => setKey(e.target.value)} disabled={!spaces.data}>
            {spaces.data?.map((s) => (
              <option key={s.id} value={s.key}>
                {s.name} ({s.key})
              </option>
            ))}
          </select>
        </div>
      )}
      <p className="kk-muted" style={{ margin: 0 }}>Page templates aren't available yet, so new pages start blank.</p>
    </Dialog>
  );
}
