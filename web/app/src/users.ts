import { useQuery } from '@tanstack/react-query';
import { lookupUsers } from './api';

const fallback = (id?: string) => (id ? `User ${id.slice(0, 8)}` : 'Unknown user');

// POST /users-bulk may not exist on older servers: a failed lookup (no retry) leaves the short-id fallback in place.
export function useUserNames(ids: (string | undefined)[]): (id?: string) => string {
  const list = [...new Set(ids.filter((i): i is string => !!i))].sort();
  const q = useQuery({ queryKey: ['users', list], queryFn: () => lookupUsers(list), enabled: list.length > 0, retry: false, staleTime: Infinity });
  return (id) => {
    const u = q.data?.find((x) => x.accountId === id);
    return u?.displayName || u?.publicName || fallback(id);
  };
}
