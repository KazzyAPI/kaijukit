import { useQuery } from '@tanstack/react-query';
import { createDraft, deletePage, getPage, listSpaces, request } from './api';

const V1 = '/wiki/rest/api';

export interface Template {
  templateId: string;
  name: string;
  description?: string;
  labels?: { name: string }[];
  space?: { key: string; name?: string };
  body?: { storage?: { value: string } };
}

export const listTemplates = (spaceKey?: string) => request<{ results: Template[] }>('GET', `${V1}/template/page`, { query: { spaceKey, limit: 250 } }).then((r) => r.results);
export const getTemplate = (id: string) => request<Template>('GET', `${V1}/template/${id}`, { query: { templateExpand: 'body' } });
export const deleteTemplate = (id: string) => request<void>('DELETE', `${V1}/template/${id}`);

/** `id` set means update. The body is ADF; the server keeps it and serves it back as storage only. */
export const saveTemplate = (t: { id?: string; name: string; description: string; labels: string[]; spaceKey?: string; adf: string }) =>
  request<Template>(t.id ? 'PUT' : 'POST', `${V1}/template`, {
    body: {
      templateId: t.id,
      name: t.name,
      templateType: 'page',
      description: t.description,
      labels: t.labels.map((name) => ({ prefix: 'global', name })),
      body: { atlas_doc_format: { representation: 'atlas_doc_format', value: t.adf } },
      space: t.spaceKey ? { key: t.spaceKey } : undefined,
    },
  });

/**
 * Templates come back as storage format only, but the editor takes ADF. A scratch draft in a space converts it
 * (the server parses storage on write and serves ADF on read) and is discarded right away.
 */
export async function templateAdf(storage: string, spaceId: string): Promise<string> {
  const scratch = await createDraft(spaceId, undefined, { representation: 'storage', value: storage });
  try {
    return (await getPage(scratch.id, true)).body?.atlas_doc_format?.value ?? '';
  } finally {
    await deletePage(scratch.id, true).catch(() => undefined);
  }
}

/** Global templates have no space, so any space the caller can see hosts the scratch draft. */
export const listSpacesForScratch = () => listSpaces().then((s) => s[0]?.id);

/** Global templates plus the space's own, space ones first. */
export const useSpaceTemplates = (spaceKey: string | undefined, enabled = true) =>
  useQuery({
    queryKey: ['templates', 'available', spaceKey],
    queryFn: async () => {
      const [space, global] = await Promise.all([listTemplates(spaceKey), listTemplates()]);
      return [...space, ...global];
    },
    enabled: enabled && !!spaceKey,
    retry: false,
  });
