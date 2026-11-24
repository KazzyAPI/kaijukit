import { Avatar, IconButton, Icon, Skeleton, Button } from '@kaijukit/ui';
import { useCallback, useEffect, useState, type KeyboardEvent, type PointerEvent } from 'react';
import { Link, NavLink, Outlet, useLocation, useMatch, useNavigate, useParams } from 'react-router-dom';
import { useSpace } from '../hooks';
import { PageTree } from '../PageTree';
import { getJson, store } from '../storage';
import { ErrorPanel } from './Errors';

const MIN = 240;
const MAX = 420;
const clamp = (n: number) => Math.min(MAX, Math.max(MIN, n));

export function SpaceShell() {
  const { key } = useParams();
  const space = useSpace(key);
  const navigate = useNavigate();
  const currentId = (useMatch('/wiki/spaces/:key/pages/:id/*')?.params.id ?? undefined) as string | undefined;
  const [width, setWidth] = useState(() => clamp(getJson('kk.sidebar.w', 280)));
  const [open, setOpen] = useState(() => getJson('kk.sidebar.open', true) && window.matchMedia('(min-width: 768px)').matches);
  const { pathname } = useLocation();
  // On narrow screens the sidebar is an overlay: leave it once a destination is chosen.
  useEffect(() => {
    if (!window.matchMedia('(min-width: 768px)').matches) setOpen(false);
  }, [pathname]);
  useEffect(() => store.set('kk.sidebar.w', String(width)), [width]);
  useEffect(() => {
    if (window.matchMedia('(min-width: 768px)').matches) store.set('kk.sidebar.open', String(open));
  }, [open]);

  const drag = useCallback((e: PointerEvent<HTMLDivElement>) => {
    const el = e.currentTarget;
    el.setPointerCapture(e.pointerId);
    const left = el.parentElement!.getBoundingClientRect().left;
    const move = (m: globalThis.PointerEvent) => setWidth(clamp(m.clientX - left));
    const up = () => {
      el.removeEventListener('pointermove', move);
      el.removeEventListener('pointerup', up);
    };
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerup', up);
  }, []);
  const resizeKey = (e: KeyboardEvent) => {
    if (e.key === 'ArrowLeft') setWidth((w) => clamp(w - 16));
    else if (e.key === 'ArrowRight') setWidth((w) => clamp(w + 16));
  };

  if (space.isError) return <ErrorPanel error={space.error} onRetry={() => void space.refetch()} />;
  const s = space.data;

  return (
    <div className="kk-shell" style={{ ['--kk-sidebar-w' as string]: `${width}px` }} data-open={open}>
      {open && (
        <>
          <nav className="kk-sidebar" aria-label="Space">
            {s ? (
              <>
                <div className="kk-sidebar__head">
                  <Avatar name={s.name} shape="square" />
                  <div className="kk-sidebar__name">
                    <strong>{s.name}</strong>
                    <span className="kk-muted">{s.key}</span>
                  </div>
                  <IconButton label="Collapse sidebar" icon={<Icon name="panel" />} onClick={() => setOpen(false)} />
                </div>
                <NavLink to={`/wiki/spaces/${s.key}/overview`} className="kk-sidebar__link">
                  Overview
                </NavLink>
                <div className="kk-sidebar__section">
                  <h2 className="kk-sidebar__label">Pages</h2>
                  <IconButton label="Create page" icon={<Icon name="plus" />} onClick={() => navigate(`/wiki/spaces/${s.key}/pages/create`)} />
                </div>
                <PageTree space={s} currentId={currentId} />
              </>
            ) : (
              <div aria-busy="true">
                <Skeleton height={32} style={{ margin: '8px 0' }} />
                <Skeleton height={20} style={{ margin: '8px 0' }} />
                <Skeleton height={20} style={{ margin: '8px 0' }} />
              </div>
            )}
          </nav>
          <div
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize sidebar"
            aria-valuenow={width}
            aria-valuemin={MIN}
            aria-valuemax={MAX}
            tabIndex={0}
            className="kk-resizer"
            onPointerDown={drag}
            onKeyDown={resizeKey}
          />
        </>
      )}
      <div className="kk-shell__main">
        {!open && (
          <div className="kk-shell__reopen">
            <Button onClick={() => setOpen(true)}>
              <Icon name="panel" /> Show sidebar
            </Button>
          </div>
        )}
        {s ? <Outlet context={s} /> : space.isPending ? <div className="kk-container"><Skeleton height={32} width="40%" /></div> : <Link to="/wiki/spaces">Spaces</Link>}
      </div>
    </div>
  );
}
