import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode, type Ref } from 'react';

export interface TriggerProps {
  ref: Ref<HTMLButtonElement>;
  'aria-haspopup': 'menu';
  'aria-expanded': boolean;
  onClick: () => void;
  onKeyDown: (e: KeyboardEvent) => void;
}

export function Menu({ renderTrigger, children, align = 'start', label }: { renderTrigger: (p: TriggerProps) => ReactNode; children: ReactNode; align?: 'start' | 'end'; label: string }) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const root = useRef<HTMLDivElement>(null);
  const items = () => Array.from(list.current?.querySelectorAll<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])') ?? []);

  useEffect(() => {
    if (!open) return;
    items()[0]?.focus();
    const away = (e: MouseEvent) => !root.current?.contains(e.target as Node) && setOpen(false);
    document.addEventListener('mousedown', away);
    return () => document.removeEventListener('mousedown', away);
  }, [open]);

  const close = (refocus: boolean) => {
    setOpen(false);
    if (refocus) trigger.current?.focus();
  };

  const onListKey = (e: KeyboardEvent) => {
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
      {open && (
        <div ref={list} role="menu" aria-label={label} className={`kk-menu__list kk-menu__list--${align}`} onKeyDown={onListKey} onClick={(e) => (e.target as HTMLElement).closest('[role="menuitem"]:not([aria-disabled="true"])') && close(true)}>
          {children}
        </div>
      )}
    </div>
  );
}

export function MenuItem({ onSelect, disabled, danger, children }: { onSelect: () => void; disabled?: boolean; danger?: boolean; children: ReactNode }) {
  return (
    <button type="button" role="menuitem" tabIndex={-1} aria-disabled={disabled || undefined} className={`kk-menu__item${danger ? ' kk-menu__item--danger' : ''}`} onClick={() => !disabled && onSelect()}>
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
