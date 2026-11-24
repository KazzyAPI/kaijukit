import { Icon, IconButton, Skeleton, Tree, type TreeItem } from '@kaijukit/ui';
import { useQueries, useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { childPages, getAncestors, rootPages, type ChildPage, type Space } from './api';
import { pageUrl } from './format';
import { getJson, store } from './storage';

export function flattenTree(roots: ChildPage[], children: Record<string, ChildPage[] | undefined>, expanded: ReadonlySet<string>, actions?: (p: ChildPage) => TreeItem['actions']): TreeItem[] {
  const out: TreeItem[] = [];
  const walk = (list: ChildPage[], level: number) =>
    list.forEach((p) => {
      const kids = children[p.id];
      const open = expanded.has(p.id);
      out.push({ id: p.id, label: p.title || 'Untitled', level, expanded: kids && kids.length === 0 ? undefined : open, actions: actions?.(p) });
      if (open && kids) walk(kids, level + 1);
    });
  walk(roots, 1);
  return out;
}

export function PageTree({ space, currentId }: { space: Space; currentId?: string }) {
  const navigate = useNavigate();
  const storeKey = `kk.tree.${space.key}`;
  const [expanded, setExpanded] = useState<string[]>(() => getJson(storeKey, []));
  useEffect(() => store.set(storeKey, JSON.stringify(expanded)), [storeKey, expanded]);

  const ancestors = useQuery({ queryKey: ['ancestors', currentId], queryFn: () => getAncestors(currentId!), enabled: !!currentId });
  useEffect(() => {
    const ids = ancestors.data?.map((a) => a.id) ?? [];
    setExpanded((e) => (ids.every((i) => e.includes(i)) ? e : [...new Set([...e, ...ids])]));
  }, [ancestors.data]);

  const roots = useQuery({ queryKey: ['tree', 'root', space.id], queryFn: () => rootPages(space.id) });
  const loadIds = useMemo(() => [...new Set([...(space.homepageId ? [space.homepageId] : []), ...expanded])], [space.homepageId, expanded]);
  const kids = useQueries({ queries: loadIds.map((id) => ({ queryKey: ['tree', 'children', id], queryFn: () => childPages(id) })) });
  const childMap = Object.fromEntries(loadIds.map((id, i) => [id, kids[i]?.data]));

  const top = [...(space.homepageId ? (childMap[space.homepageId] ?? []) : []), ...(roots.data ?? []).filter((r) => r.id !== space.homepageId)];
  const titles = Object.fromEntries([...top, ...Object.values(childMap).flatMap((c) => c ?? [])].map((p) => [p.id, p.title]));
  const set = useMemo(() => new Set(expanded), [expanded]);
  const items = flattenTree(top, childMap, set, (p) => (
    <IconButton label={`Create child page under ${p.title || 'Untitled'}`} icon={<Icon name="plus" />} tabIndex={-1} onClick={() => navigate(`/wiki/spaces/${space.key}/pages/create?parentPageId=${p.id}`)} />
  ));

  if (roots.isPending) return <div aria-busy="true">{[0, 1, 2].map((i) => <Skeleton key={i} height={24} style={{ margin: '8px 0' }} />)}</div>;
  if (roots.isError) return <p className="kk-muted" role="alert">Couldn't load pages.</p>;
  if (items.length === 0) return <p className="kk-muted" style={{ padding: '0 8px' }}>No pages yet.</p>;

  return (
    <Tree
      label="Pages"
      items={items}
      selectedId={currentId}
      onSelect={(id) => navigate(pageUrl(space.key, id, titles[id] ?? ''))}
      onToggle={(id, open) => setExpanded((e) => (open ? [...new Set([...e, id])] : e.filter((x) => x !== id)))}
    />
  );
}
