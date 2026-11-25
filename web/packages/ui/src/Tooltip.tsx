import { cloneElement, useEffect, useId, useState, type ReactElement } from 'react';

export function Tooltip({ content, children, above }: { content: string; children: ReactElement<Record<string, unknown>>; above?: boolean }) {
  const id = useId();
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (!open) return;
    const esc = (e: KeyboardEvent) => e.key === 'Escape' && setOpen(false);
    document.addEventListener('keydown', esc);
    return () => document.removeEventListener('keydown', esc);
  }, [open]);
  return (
    <span className="kk-tooltip" onPointerDown={() => setOpen(false)} onMouseEnter={() => setOpen(true)} onMouseLeave={() => setOpen(false)} onFocus={() => setOpen(true)} onBlur={() => setOpen(false)}>
      {cloneElement(children, { 'aria-describedby': open ? id : undefined })}
      {open && (
        <span role="tooltip" id={id} className={`kk-tooltip__tip${above ? ' kk-tooltip__tip--above' : ''}`}>
          {content}
        </span>
      )}
    </span>
  );
}
