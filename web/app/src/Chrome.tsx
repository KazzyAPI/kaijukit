import { useCallback, useEffect, useState, type KeyboardEvent, type PointerEvent } from 'react';
import { Outlet, useLocation, useMatch } from 'react-router-dom';
import { useSpace } from './hooks';
import { LeftNav } from './LeftNav';
import { ErrorBoundary } from './screens/Errors';
import { getJson, store } from './storage';
import { TopBar } from './TopBar';

const MIN = 240;
const MAX = 420;
const clamp = (n: number) => Math.min(MAX, Math.max(MIN, n));
const wide = () => window.matchMedia('(min-width: 768px)').matches;

export function Chrome() {
  const key = useMatch('/wiki/spaces/:key/*')?.params.key;
  const space = useSpace(key);
  const [width, setWidth] = useState(() => clamp(getJson('kk.sidebar.w', 280)));
  const [open, setOpen] = useState(() => getJson('kk.sidebar.open', true) && wide());
  const { pathname } = useLocation();
  // On narrow screens the panel is an overlay: leave it once a destination is chosen.
  useEffect(() => {
    if (!wide()) setOpen(false);
  }, [pathname]);
  useEffect(() => store.set('kk.sidebar.w', String(width)), [width]);
  useEffect(() => {
    if (wide()) store.set('kk.sidebar.open', String(open));
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

  return (
    <>
      <a className="kk-skip" href="#main">Skip to content</a>
      <TopBar navOpen={open} onToggleNav={() => setOpen((o) => !o)} />
      <div className="kk-shell" style={{ ['--kk-sidebar-w' as string]: `${width}px` }} data-open={open}>
        {open && (
          <>
            <LeftNav space={key ? space.data : undefined} loading={!!key && space.isPending} />
            <div className="kk-scrim" aria-hidden="true" onClick={() => setOpen(false)} />
            <div role="separator" aria-orientation="vertical" aria-label="Resize sidebar" aria-valuenow={width} aria-valuemin={MIN} aria-valuemax={MAX} tabIndex={0} className="kk-resizer" onPointerDown={drag} onKeyDown={resizeKey} />
          </>
        )}
        <main id="main" className="kk-main">
          <ErrorBoundary>
            <Outlet />
          </ErrorBoundary>
        </main>
      </div>
    </>
  );
}
