import { cloneElement, useEffect, useId, useState, type ReactElement } from 'react';

export function Tooltip({ content, children }: { content: string; children: ReactElement<Record<string, unknown>> }) {
  const id = useId();
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (!open) return;
    const esc = (e: KeyboardEvent) => e.key === 'Escape' && setOpen(false);
    document.addEventListener('keydown', esc);
    return () => document.removeEventListener('keydown', esc);
  }, [open]);
  return (
    <span className="kk-tooltip" onMouseEnter={() => setOpen(true)} onMouseLeave={() => setOpen(false)} onFocus={() => setOpen(true)} onBlur={() => setOpen(false)}>
      {cloneElement(children, { 'aria-describedby': open ? id : undefined })}
      {open && (
        <span role="tooltip" id={id} className="kk-tooltip__tip">
          {content}
        </span>
      )}
    </span>
  );
}
