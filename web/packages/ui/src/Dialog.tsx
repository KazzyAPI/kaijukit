import { useEffect, useId, useRef, type ReactNode } from 'react';

export interface DialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
  onSubmit?: () => void;
}

// Native <dialog> gives the focus trap, inert background, Esc and focus return.
export function Dialog({ open, onClose, title, children, footer, onSubmit }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    if (!open && d.open) d.close();
  }, [open]);
  const body = (
    <>
      <div className="kk-dialog__head">
        <h2 id={titleId} className="kk-dialog__title">
          {title}
        </h2>
      </div>
      <div className="kk-dialog__body">{children}</div>
      {footer && <div className="kk-dialog__foot">{footer}</div>}
    </>
  );
  return (
    <dialog ref={ref} className="kk-dialog" aria-labelledby={titleId} onClose={onClose} onMouseDown={(e) => e.target === ref.current && onClose()}>
      {open &&
        (onSubmit ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              onSubmit();
            }}
          >
            {body}
          </form>
        ) : (
          body
        ))}
    </dialog>
  );
}
