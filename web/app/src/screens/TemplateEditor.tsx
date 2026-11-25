import { Editor } from '@kaijukit/editor';
import { emptyDoc, parseAdf, type AdfDoc } from '@kaijukit/editor/adf';
import { Button, Skeleton, TextArea, TextField, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import { errorTitle, getSpaceByKey } from '../api';
import { QueryState } from '../QueryState';
import { getTemplate, listSpacesForScratch, saveTemplate, templateAdf, type Template } from '../templatesApi';

/** Create (`:tid` = new) or edit one template, global (no space key) or a space's. The body uses the page editor. */
export function TemplateEditorRoute({ spaceKey }: { spaceKey?: string }) {
  const { tid } = useParams();
  const isNew = tid === 'new';
  const load = useQuery({
    queryKey: ['template', tid, spaceKey],
    queryFn: async (): Promise<{ t?: Template; adf: string }> => {
      if (isNew) return { adf: JSON.stringify(emptyDoc()) };
      const t = await getTemplate(tid!);
      const spaceId = spaceKey ? (await getSpaceByKey(spaceKey)).id : await listSpacesForScratch();
      return { t, adf: spaceId ? await templateAdf(t.body?.storage?.value ?? '', spaceId) : JSON.stringify(emptyDoc()) };
    },
    staleTime: 0,
    gcTime: 0,
    refetchOnWindowFocus: false,
  });
  return (
    <QueryState query={load} skeleton={<div className="kk-container"><Skeleton height={36} width="50%" /><Skeleton height={240} style={{ marginTop: 16 }} /></div>}>
      {({ t, adf }) => <TemplateForm key={tid} template={t} adf={adf} spaceKey={spaceKey} />}
    </QueryState>
  );
}

function TemplateForm({ template, adf, spaceKey }: { template?: Template; adf: string; spaceKey?: string }) {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const toast = useToast();
  const [name, setName] = useState(template?.name ?? '');
  const [description, setDescription] = useState(template?.description ?? '');
  const doc = useRef<AdfDoc>(parseAdf(adf) ?? emptyDoc());
  const back = spaceKey ? `/wiki/spaces/${spaceKey}/settings?tab=templates` : '/wiki/admin/templates';
  const save = useMutation({
    mutationFn: () => saveTemplate({ id: template?.templateId, name: name.trim(), description: description.trim(), labels: template?.labels?.map((l) => l.name) ?? [], spaceKey, adf: JSON.stringify(doc.current) }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['templates'] });
      toast({ title: 'Template saved', appearance: 'success' });
      navigate(back);
    },
    onError: (e) => toast({ title: errorTitle(e, "Couldn't save the template"), description: e.message, appearance: 'error' }),
  });
  return (
    <form
      className="kk-container kk-tpled"
      onSubmit={(e) => {
        e.preventDefault();
        if (name.trim()) save.mutate();
      }}
    >
      <h1 className="kk-h1">{template ? 'Edit template' : 'Create template'}</h1>
      <TextField label="Template name" value={name} maxLength={255} onChange={(e) => setName(e.target.value)} required autoFocus />
      <TextArea label="Description (optional)" rows={2} maxLength={255} value={description} onChange={(e) => setDescription(e.target.value)} />
      <Editor label="Template content" value={doc.current} onChange={(d) => (doc.current = d)} onError={(m) => toast({ title: m, appearance: 'error' })} />
      <div className="kk-row">
        <Button type="submit" appearance="primary" loading={save.isPending} disabled={!name.trim()}>Save template</Button>
        <Link className="kk-btn" to={back}>Cancel</Link>
      </div>
    </form>
  );
}
