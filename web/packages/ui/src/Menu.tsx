import { createPortal } from 'react-dom';
import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent, type ReactNode, type Ref } from 'react';

export interface TriggerProps {
  ref: Ref<HTMLButtonElement>;
  'aria-haspopup': 'menu';
  'aria-expanded': boolean;
  onClick: () => void;
  onKeyDown: (e: KeyboardEvent) => void;
}

// `side="right"` opens a flyout beside the trigger (left nav rows) instead of below it.
export function Menu({ renderTrigger, children, align = 'start', side = 'bottom', label }: { renderTrigger: (p: TriggerProps) => ReactNode; children: ReactNode; align?: 'start' | 'end'; side?: 'bottom' | 'right'; label: string }) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState<CSSProperties>();
  const trigger = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const root = useRef<HTMLDivElement>(null);
  const items = () => Array.from(list.current?.querySelectorAll<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])') ?? []);

  useEffect(() => {
    if (!open) return;
    (items()[0] ?? list.current)?.focus();
    // Fixed so a menu inside a scrolling panel (left nav) is not clipped; the CSS min() keeps it on screen.
    const r = root.current!.getBoundingClientRect();
    setPos(
      side === 'right'
        ? { top: r.top, left: `min(${r.right + 4}px, calc(100vw - 216px))` }
        : align === 'end'
          ? { top: r.bottom + 4, right: Math.max(8, window.innerWidth - r.right) }
          : { top: r.bottom + 4, left: `min(${r.left}px, calc(100vw - 216px))` },
    );
    const away = (e: MouseEvent) => !root.current?.contains(e.target as Node) && !list.current?.contains(e.target as Node) && setOpen(false);
    const scrolled = (e: Event) => !list.current?.contains(e.target as Node) && setOpen(false);
    document.addEventListener('mousedown', away);
    window.addEventListener('scroll', scrolled, true);
    window.addEventListener('resize', scrolled);
    return () => {
      document.removeEventListener('mousedown', away);
      window.removeEventListener('scroll', scrolled, true);
      window.removeEventListener('resize', scrolled);
    };
  }, [open, side, align]);

  const close = (refocus: boolean) => {
    setOpen(false);
    if (refocus) trigger.current?.focus();
  };

  const onListKey = (e: KeyboardEvent) => {
    e.stopPropagation();
    const els = items();
    const i = els.indexOf(document.activeElement as HTMLElement);
    const go = (n: number) => {
      e.preventDefault();
      els[(n + els.length) % els.length]?.focus();
    };
    if (e.key === 'ArrowDown') go(i + 1);
    else if (e.key === 'ArrowUp') go(i - 1);
    else if (e.key === 'Home') go(0);
    else if (e.key === 'End') go(els.length - 1);
    else if (e.key === 'Escape') {
      e.preventDefault();
      close(true);
    } else if (e.key === 'Tab') close(false);
  };

  return (
    <div className="kk-menu" ref={root}>
      {renderTrigger({
        ref: trigger,
        'aria-haspopup': 'menu',
        'aria-expanded': open,
        onClick: () => setOpen((o) => !o),
        onKeyDown: (e) => {
          if (e.key === 'ArrowDown' && !open) {
            e.preventDefault();
            setOpen(true);
          }
        },
      })}
      {open &&
        createPortal(
          <div ref={list} role="menu" tabIndex={-1} aria-label={label} className="kk-menu__list" style={pos} onKeyDown={onListKey} onClick={(e) => (e.target as HTMLElement).closest('[role="menuitem"]:not([aria-disabled="true"])') && close(true)}>
            {children}
          </div>,
          document.body,
        )}
    </div>
  );
}

export function MenuItem({ onSelect, disabled, danger, title, children }: { onSelect: () => void; disabled?: boolean; danger?: boolean; title?: string; children: ReactNode }) {
  return (
    <button type="button" role="menuitem" tabIndex={-1} aria-disabled={disabled || undefined} title={title} className={`kk-menu__item${danger ? ' kk-menu__item--danger' : ''}`} onClick={() => !disabled && onSelect()}>
      {children}
    </button>
  );
}

export const MenuSeparator = () => <hr className="kk-menu__sep" role="separator" />;
export const MenuLabel = ({ children }: { children: ReactNode }) => (
  <div className="kk-menu__label" role="presentation">
    {children}
  </div>
);
