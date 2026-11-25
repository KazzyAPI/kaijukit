import { Button, Dialog } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { listSpaces } from './api';
import { useSpaceTemplates } from './templatesApi';

/** Pick a space (unless one is fixed) and a template, then open the editor on a new draft. */
export function CreatePageDialog({ open, onClose, spaceKey }: { open: boolean; onClose: () => void; spaceKey?: string }) {
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces(), enabled: open && !spaceKey });
  const [key, setKey] = useState('');
  const [template, setTemplate] = useState('');
  const navigate = useNavigate();
  const chosen = spaceKey ?? (key || spaces.data?.[0]?.key);
  const templates = useSpaceTemplates(chosen, open);
  // A template from another space must not stay selected after the space changes.
  useEffect(() => setTemplate(''), [chosen, open]);
  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Create page"
      onSubmit={() => {
        if (!chosen) return;
        onClose();
        navigate(`/wiki/spaces/${chosen}/pages/create${template ? `?templateId=${template}` : ''}`);
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
      {!spaceKey && spaces.data?.length === 0 ? (
        <p style={{ margin: 0 }}>Create a space first. Pages live inside spaces.</p>
      ) : (
        !spaceKey && (
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
        )
      )}
      {!!templates.data?.length && (
      <fieldset className="kk-tpl">
        <legend className="kk-field__label">Start from</legend>
        <label className="kk-tpl__opt">
          <input type="radio" name="kk-template" checked={template === ''} onChange={() => setTemplate('')} />
          <span>
            <strong>Blank page</strong>
          </span>
        </label>
        {templates.data?.map((t) => (
          <label key={t.templateId} className="kk-tpl__opt">
            <input type="radio" name="kk-template" checked={template === t.templateId} onChange={() => setTemplate(t.templateId)} />
            <span>
              <strong>{t.name}</strong>
              <span className="kk-muted"> {t.space ? 'Space template' : 'Global template'}{t.description ? `. ${t.description}` : ''}</span>
            </span>
          </label>
        ))}
      </fieldset>
      )}
    </Dialog>
  );
}
