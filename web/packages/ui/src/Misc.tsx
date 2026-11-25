import { useState, type CSSProperties, type ReactNode } from 'react';

export function Spinner({ size, label }: { size?: 'lg'; label?: string }) {
  return <span className={`kk-spinner${size ? ` kk-spinner--${size}` : ''}`} role={label ? 'status' : undefined} aria-label={label} aria-hidden={label ? undefined : true} />;
}

export function Skeleton({ width = '100%', height = 16, style }: { width?: number | string; height?: number | string; style?: CSSProperties }) {
  return <span className="kk-skeleton" aria-hidden="true" style={{ width, height, ...style }} />;
}

export function Lozenge({ appearance = 'default', children }: { appearance?: 'default' | 'success' | 'danger' | 'warning' | 'info' | 'brand'; children: ReactNode }) {
  return <span className={`kk-lozenge${appearance === 'default' ? '' : ` kk-lozenge--${appearance}`}`}>{children}</span>;
}

export function Avatar({ name, size = 'md', shape = 'circle' }: { name: string; size?: 'sm' | 'md' | 'lg'; shape?: 'circle' | 'square' }) {
  const initials = name.replace(/@.*/, '').split(/[\s._-]+/).filter(Boolean).slice(0, 2).map((w) => w[0]).join('').toUpperCase() || '?';
  return (
    <span className={`kk-avatar kk-avatar--${size}${shape === 'square' ? ' kk-avatar--square' : ''}`} role="img" aria-label={name} title={name}>
      {initials}
    </span>
  );
}

export function EmptyState({ title, description, action, icon }: { title: string; description?: ReactNode; action?: ReactNode; icon?: ReactNode }) {
  return (
    <div className="kk-empty">
      {icon}
      <h2 className="kk-empty__title">{title}</h2>
      {description && <p className="kk-empty__text">{description}</p>}
      {action}
    </div>
  );
}

export interface Crumb {
  label: string;
  href?: string;
}

const COLLAPSE_OVER = 4;

export function Breadcrumbs({ items, renderLink }: { items: Crumb[]; renderLink?: (c: { href: string; children: ReactNode }) => ReactNode }) {
  const [expanded, setExpanded] = useState(false);
  const hidden = !expanded && items.length > COLLAPSE_OVER ? new Set(items.slice(1, -2).map((_, i) => i + 1)) : new Set<number>();
  return (
    <nav className="kk-breadcrumbs" aria-label="Breadcrumbs">
      <ol>
        {items.map((c, i) => {
          const last = i === items.length - 1;
          if (hidden.has(i)) {
            return i === 1 ? (
              <li key="more">
                <button type="button" className="kk-breadcrumbs__more" aria-label={`Show ${hidden.size} more levels`} onClick={() => setExpanded(true)}>
                  ...
                </button>
              </li>
            ) : null;
          }
          return (
            <li key={i}>
              {last || !c.href ? (
                <span aria-current={last ? 'page' : undefined}>{c.label}</span>
              ) : renderLink ? (
                renderLink({ href: c.href, children: c.label })
              ) : (
                <a href={c.href}>{c.label}</a>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
