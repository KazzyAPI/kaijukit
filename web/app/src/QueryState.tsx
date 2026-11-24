import type { UseQueryResult } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { ErrorPanel } from './screens/Errors';

export function QueryState<T>({ query, skeleton, children }: { query: UseQueryResult<T>; skeleton: ReactNode; children: (data: T) => ReactNode }) {
  if (query.isPending) return <div aria-busy="true" aria-live="polite">{skeleton}</div>;
  if (query.isError) return <ErrorPanel error={query.error} onRetry={() => void query.refetch()} />;
  return <>{children(query.data)}</>;
}
