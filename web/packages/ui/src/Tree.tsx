import { useEffect, useId, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { Icon } from './Icon';

export interface TreeItem {
  id: string;
  label: ReactNode;
  icon?: ReactNode;
  level: number;
  /** undefined = leaf */
  expanded?: boolean;
  actions?: ReactNode;
}

export interface TreeProps {
  items: TreeItem[];
  label: string;
  selectedId?: string;
  onSelect: (id: string) => void;
  onToggle: (id: string, expand: boolean) => void;
  /** Enables pointer drag and drop; keyboard users need another route (a move dialog). */
  onDrop?: (id: string, targetId: string, position: DropPosition) => void;
}

export type DropPosition = 'before' | 'after' | 'append';

// Flat visible-rows model: the caller owns expansion/lazy loading, the tree owns keyboard and ARIA.
export function Tree({ items, label, selectedId, onSelect, onToggle, onDrop }: TreeProps) {
  const [focusId, setFocusId] = useState<string | undefined>();
  const [drag, setDrag] = useState<string>();
  const [hint, setHint] = useState<{ id: string; position: DropPosition }>();
  const ul = useRef<HTMLUListElement>(null);
  const uid = useId();
  const active = items.some((i) => i.id === focusId) ? focusId : items.some((i) => i.id === selectedId) ? selectedId : items[0]?.id;

  useEffect(() => {
    if (focusId && ul.current?.contains(document.activeElement)) (Array.from(ul.current.children) as HTMLElement[]).find((el) => el.dataset.id === focusId)?.focus();
  }, [focusId]);

  const onKeyDown = (e: KeyboardEvent) => {
    const i = items.findIndex((it) => it.id === active);
    const it = items[i];
    if (!it) return;
    const focus = (n: number) => {
      const t = items[n];
      if (t) {
        e.preventDefault();
        setFocusId(t.id);
      }
    };
    switch (e.key) {
      case 'ArrowDown': return focus(i + 1);
      case 'ArrowUp': return focus(i - 1);
      case 'Home': return focus(0);
      case 'End': return focus(items.length - 1);
      case 'ArrowRight':
        if (it.expanded === false) {
          e.preventDefault();
          onToggle(it.id, true);
        } else if (it.expanded) focus(i + 1);
        return;
      case 'ArrowLeft':
        if (it.expanded) {
          e.preventDefault();
          onToggle(it.id, false);
        } else {
          for (let n = i - 1; n >= 0; n--) if ((items[n]?.level ?? 0) < it.level) return focus(n);
        }
        return;
      case 'Enter':
      case ' ':
        e.preventDefault();
        onSelect(it.id);
    }
  };

  return (
    <ul ref={ul} role="tree" aria-label={label} className="kk-tree" onKeyDown={onKeyDown}>
      {items.map((it, n) => (
        <li
          key={it.id}
          role="treeitem"
          data-id={it.id}
          aria-labelledby={`${uid}-${it.id}`}
          aria-level={it.level}
          aria-expanded={it.expanded}
          aria-selected={it.id === selectedId}
          {...siblingInfo(items, n)}
          tabIndex={it.id === active ? 0 : -1}
          className="kk-tree__item"
          onFocus={(e) => e.target === e.currentTarget && setFocusId(it.id)}
        >
          <div
            className="kk-tree__row"
            style={{ paddingLeft: (it.level - 1) * 16 }}
            data-drop={hint?.id === it.id ? hint.position : undefined}
            draggable={onDrop ? true : undefined}
            onDragStart={(e) => {
              setDrag(it.id);
              e.dataTransfer.effectAllowed = 'move';
              e.dataTransfer.setData('text/plain', it.id);
            }}
            onDragEnd={() => {
              setDrag(undefined);
              setHint(undefined);
            }}
            onDragOver={(e) => {
              if (!drag || drag === it.id) return;
              e.preventDefault();
              const r = e.currentTarget.getBoundingClientRect();
              const y = (e.clientY - r.top) / r.height;
              const position = y < 0.25 ? 'before' : y > 0.75 ? 'after' : 'append';
              if (hint?.id !== it.id || hint.position !== position) setHint({ id: it.id, position });
            }}
            onDragLeave={(e) => !e.currentTarget.contains(e.relatedTarget as Node | null) && setHint(undefined)}
            onDrop={(e) => {
              e.preventDefault();
              if (drag && hint && drag !== it.id) onDrop?.(drag, it.id, hint.position);
              setDrag(undefined);
              setHint(undefined);
            }}
            onClick={() => onSelect(it.id)}
          >
            <span
              className="kk-tree__chev"
              data-open={it.expanded || undefined}
              onClick={(e) => {
                if (it.expanded === undefined) return;
                e.stopPropagation();
                onToggle(it.id, !it.expanded);
              }}
            >
              {it.expanded === undefined ? <span className="kk-tree__dot" /> : <Icon name="chevronRight" size={14} />}
            </span>
            {it.icon && <span className="kk-tree__icon">{it.icon}</span>}
            <span id={`${uid}-${it.id}`} className="kk-tree__label">{it.label}</span>
            {it.actions && (
              <span className="kk-tree__actions" onClick={(e) => e.stopPropagation()}>
                {it.actions}
              </span>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}

function siblingInfo(items: TreeItem[], n: number) {
  const level = items[n]?.level ?? 1;
  let a = n;
  while (a > 0 && (items[a - 1]?.level ?? 0) >= level) a--;
  let b = n;
  while (b < items.length - 1 && (items[b + 1]?.level ?? 0) >= level) b++;
  const sibs = items.slice(a, b + 1).filter((x) => x.level === level);
  return { 'aria-setsize': sibs.length, 'aria-posinset': sibs.findIndex((x) => x.id === items[n]?.id) + 1 };
}
