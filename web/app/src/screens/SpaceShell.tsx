import { Skeleton } from '@kaijukit/ui';
import { Outlet, useParams } from 'react-router-dom';
import { useSpace } from '../hooks';
import { ErrorPanel } from './Errors';

// Loads the space once for every route under /wiki/spaces/:key; the left nav in Chrome shows its tree.
export function SpaceShell() {
  const space = useSpace(useParams().key);
  if (space.isError) return <ErrorPanel error={space.error} onRetry={() => void space.refetch()} />;
  if (!space.data) return <div className="kk-container" aria-busy="true"><Skeleton height={32} width="40%" /></div>;
  return <Outlet context={space.data} />;
}
