import { Icon, IconButton, Menu, MenuItem, Skeleton, Tree, useToast, type TreeItem } from '@kaijukit/ui';
import { useQueries, useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { childPages, getAncestors, pagesByTitle, rootPages, type ChildPage, type Space } from './api';
import { ContentIcon } from './ContentIcon';
import { useDebounced } from './hooks';
import { pageUrl } from './format';
import { getJson, store } from './storage';

export function flattenTree(roots: ChildPage[], children: Record<string, ChildPage[] | undefined>, expanded: ReadonlySet<string>, actions?: (p: ChildPage) => TreeItem['actions']): TreeItem[] {
  const out: TreeItem[] = [];
  const walk = (list: ChildPage[], level: number) =>
    list.forEach((p) => {
      const kids = children[p.id];
      const open = expanded.has(p.id);
      out.push({ id: p.id, label: p.title || 'Untitled', icon: <ContentIcon type={p.type} />, level, expanded: kids && kids.length === 0 ? undefined : open, actions: actions?.(p) });
      if (open && kids) walk(kids, level + 1);
    });
  walk(roots, 1);
  return out;
}

export function PageTree({ space, currentId }: { space: Space; currentId?: string }) {
  const [text, setText] = useState('');
  const q = useDebounced(text.trim());
  return (
    <>
      <input
        type="search"
        className="kk-input kk-nav__search"
        aria-label="Search by title"
        placeholder="Search by title"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => e.key === 'Escape' && setText('')}
      />
      {q ? <TitleHits space={space} q={q} /> : <TreeRows space={space} currentId={currentId} />}
    </>
  );
}

function TitleHits({ space, q }: { space: Space; q: string }) {
  const hits = useQuery({ queryKey: ['title-search', space.id, q], queryFn: () => pagesByTitle(q, space.id) });
  if (hits.isPending) return <div aria-busy="true"><Skeleton height={24} style={{ margin: '8px 0' }} /></div>;
  if (hits.isError) return <p className="kk-muted" role="alert">Search failed.</p>;
  if (!hits.data.length) return <p className="kk-muted kk-nav__empty" role="status">No pages match "{q}".</p>;
  return (
    <ul className="kk-nav__hits" aria-label="Matching pages">
      {hits.data.map((p) => (
        <li key={p.id}>
          <Link className="kk-nav__row" to={pageUrl(space.key, p.id, p.title)}>
            <ContentIcon />
            <span className="kk-nav__label">{p.title || 'Untitled'}</span>
          </Link>
        </li>
      ))}
    </ul>
  );
}

function TreeRows({ space, currentId }: { space: Space; currentId?: string }) {
  const navigate = useNavigate();
  const toast = useToast();
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
    <>
      <IconButton label={`Create child page under ${p.title || 'Untitled'}`} icon={<Icon name="plus" />} tabIndex={-1} onClick={() => navigate(`/wiki/spaces/${space.key}/pages/create?parentPageId=${p.id}`)} />
      <Menu label={`Actions for ${p.title || 'Untitled'}`} align="end" renderTrigger={(t) => <IconButton label={`More actions for ${p.title || 'Untitled'}`} icon={<Icon name="more" />} tabIndex={-1} {...t} />}>
        <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/pages/edit-v2/${p.id}`)}>Edit</MenuItem>
        <MenuItem
          onSelect={() =>
            void navigator.clipboard?.writeText(`${location.origin}${pageUrl(space.key, p.id, p.title)}`).then(
              () => toast({ title: 'Link copied', appearance: 'success' }),
              () => toast({ title: "Couldn't copy the link", appearance: 'warning' }),
            )
          }
        >
          Copy link
        </MenuItem>
      </Menu>
    </>
  ));

  if (roots.isPending) return <div aria-busy="true">{[0, 1, 2].map((i) => <Skeleton key={i} height={24} style={{ margin: '8px 0' }} />)}</div>;
  if (roots.isError) return <p className="kk-muted" role="alert">Couldn't load pages.</p>;
  if (items.length === 0) return <p className="kk-muted kk-nav__empty">No pages yet.</p>;

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
