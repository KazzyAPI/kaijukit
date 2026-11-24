import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useMatch } from 'react-router-dom';
import { getSpaceByKey } from './api';

export const useSpace = (key: string | undefined) => useQuery({ queryKey: ['space', key], queryFn: () => getSpaceByKey(key!), enabled: !!key });

export function useDebounced<T>(value: T, ms = 250): T {
  const [v, setV] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setV(value), ms);
    return () => clearTimeout(t);
  }, [value, ms]);
  return v;
}

/** Id of the page being viewed or edited, if any (`pages/create` has none). */
export function useCurrentPageId(): string | undefined {
  const edit = useMatch('/wiki/spaces/:key/pages/edit-v2/:id')?.params.id;
  const view = useMatch('/wiki/spaces/:key/pages/:id/*')?.params.id;
  return edit ?? (view && view !== 'create' && view !== 'edit-v2' ? view : undefined);
}
