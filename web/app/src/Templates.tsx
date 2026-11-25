import { Button, Dialog, EmptyState, Icon, IconButton, Skeleton, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { errorTitle } from './api';
import { deleteTemplate, listTemplates, type Template } from './templatesApi';
import { QueryState } from './QueryState';

/** List, create, edit and delete page templates: a space's own (`spaceKey`) or the global ones. Writes need space admin or site admin; the server answers 403 otherwise. */
export function TemplateManager({ spaceKey }: { spaceKey?: string }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const [deleting, setDeleting] = useState<Template | null>(null);
  const base = spaceKey ? `/wiki/spaces/${spaceKey}/templates` : '/wiki/admin/templates';
  const list = useQuery({ queryKey: ['templates', 'list', spaceKey ?? 'global'], queryFn: () => listTemplates(spaceKey) });
  const del = useMutation({
    mutationFn: (t: Template) => deleteTemplate(t.templateId),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['templates'] });
      setDeleting(null);
      toast({ title: 'Template deleted', appearance: 'success' });
    },
    onError: (e) => {
      setDeleting(null);
      toast({ title: errorTitle(e, "Couldn't delete the template"), description: e.message, appearance: 'error' });
    },
  });
  return (
    <div className="kk-settings">
      <div className="kk-row kk-row--between kk-tplhead">
        <p className="kk-muted" style={{ margin: 0 }}>
          {spaceKey ? 'Templates in this space appear next to the global ones when you create a page here.' : 'Global templates are offered in every space.'}
        </p>
        <Button appearance="primary" onClick={() => navigate(`${base}/new`)}>
          <Icon name="plus" /> Create template
        </Button>
      </div>
      <QueryState query={list} skeleton={<Skeleton height={56} style={{ marginBottom: 8 }} />}>
        {(items) =>
          items.length === 0 ? (
            <EmptyState title="No templates yet" description="A template is a page body that new pages can start from." />
          ) : (
            <ul className="kk-results" aria-label={spaceKey ? 'Space templates' : 'Global templates'}>
              {items.map((t) => (
                <li key={t.templateId} className="kk-result kk-tplrow">
                  <div className="kk-result__text">
                    <span className="kk-result__title">{t.name}</span>
                    {t.description && <span className="kk-muted">{t.description}</span>}
                  </div>
                  <span className="kk-row">
                    <IconButton label={`Edit ${t.name}`} icon={<Icon name="edit" />} onClick={() => navigate(`${base}/${t.templateId}`)} />
                    <IconButton label={`Delete ${t.name}`} icon={<Icon name="trash" />} onClick={() => setDeleting(t)} />
                  </span>
                </li>
              ))}
            </ul>
          )
        }
      </QueryState>
      <Dialog
        open={!!deleting}
        onClose={() => setDeleting(null)}
        title="Delete template?"
        footer={
          <>
            <Button onClick={() => setDeleting(null)}>Cancel</Button>
            <Button appearance="danger" loading={del.isPending} onClick={() => deleting && del.mutate(deleting)}>Delete</Button>
          </>
        }
      >
        <p style={{ margin: 0 }}>"{deleting?.name}" is deleted for good. Pages already made from it are not changed.</p>
      </Dialog>
    </div>
  );
}
