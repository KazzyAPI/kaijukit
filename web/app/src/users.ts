import { useQuery } from '@tanstack/react-query';
import { lookupUsers } from './api';
import { useAuth } from './auth';

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

// There is no "current user" endpoint, so the account id is found by email among the first users.
// shortcut: scans account ids 1..100, add GET /user/current when the server has it.
export function useMyId(): string | undefined {
  const { user } = useAuth();
  const ids = Array.from({ length: 100 }, (_, i) => String(i + 1));
  const q = useQuery({ queryKey: ['me', user?.email], queryFn: () => lookupUsers(ids), enabled: !!user, retry: false, staleTime: Infinity });
  return q.data?.find((u) => u.email?.toLowerCase() === user?.email.toLowerCase())?.accountId;
}
