import { useId, useRef, type KeyboardEvent, type ReactNode } from 'react';

export interface TabDef {
  id: string;
  label: string;
  disabled?: boolean;
}

export function Tabs({ tabs, value, onChange, label, children }: { tabs: TabDef[]; value: string; onChange: (id: string) => void; label: string; children: ReactNode }) {
  const uid = useId();
  const list = useRef<HTMLDivElement>(null);
  const enabled = tabs.filter((t) => !t.disabled);
  const onKey = (e: KeyboardEvent) => {
    const i = enabled.findIndex((t) => t.id === value);
    const n = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : e.key === 'Home' ? 0 : e.key === 'End' ? enabled.length - 1 : null;
    if (n === null) return;
    e.preventDefault();
    const t = enabled[(n + enabled.length) % enabled.length];
    if (!t) return;
    onChange(t.id);
    list.current?.querySelector<HTMLElement>(`[data-tab="${t.id}"]`)?.focus();
  };
  return (
    <div className="kk-tabs">
      <div ref={list} role="tablist" aria-label={label} className="kk-tabs__list" onKeyDown={onKey}>
        {tabs.map((t) => (
          <button key={t.id} type="button" role="tab" data-tab={t.id} id={`${uid}-t-${t.id}`} aria-controls={`${uid}-p`} aria-selected={t.id === value} tabIndex={t.id === value ? 0 : -1} disabled={t.disabled} className="kk-tab" onClick={() => onChange(t.id)}>
            {t.label}
          </button>
        ))}
      </div>
      <div role="tabpanel" id={`${uid}-p`} aria-labelledby={`${uid}-t-${value}`} className="kk-tabs__panel">
        {children}
      </div>
    </div>
  );
}
