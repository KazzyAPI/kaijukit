import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { Button } from './Button';
import { Icon } from './Icon';

export interface ToastInput {
  title: string;
  description?: string;
  appearance?: 'info' | 'success' | 'warning' | 'error';
  action?: { label: string; onClick: () => void };
}
interface Toast extends ToastInput {
  id: number;
}

const Ctx = createContext<(t: ToastInput) => void>(() => undefined);
export const useToast = () => useContext(Ctx);

const MAX = 4;

function ToastItem({ toast, onDismiss }: { toast: Toast; onDismiss: (id: number) => void }) {
  useEffect(() => {
    const t = setTimeout(() => onDismiss(toast.id), toast.appearance === 'error' ? 10000 : 5000);
    return () => clearTimeout(t);
  }, [toast, onDismiss]);
  return (
    <div className={`kk-toast kk-toast--${toast.appearance ?? 'info'}`}>
      <div className="kk-toast__body">
        <div className="kk-toast__title">{toast.title}</div>
        {toast.description && <div>{toast.description}</div>}
        {toast.action && (
          <Button appearance="subtle" onClick={toast.action.onClick}>
            {toast.action.label}
          </Button>
        )}
      </div>
      <Button appearance="subtle" className="kk-btn--icon" aria-label="Dismiss" onClick={() => onDismiss(toast.id)}>
        <Icon name="close" />
      </Button>
    </div>
  );
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const next = useRef(1);
  const dismiss = useCallback((id: number) => setToasts((l) => l.filter((t) => t.id !== id)), []);
  const push = useCallback((t: ToastInput) => setToasts((l) => [...l, { ...t, id: next.current++ }].slice(-MAX)), []);
  const value = useMemo(() => push, [push]);
  return (
    <Ctx.Provider value={value}>
      {children}
      <div className="kk-toasts" role="region" aria-label="Notifications" aria-live="polite">
        {toasts.map((t) => (
          <ToastItem key={t.id} toast={t} onDismiss={dismiss} />
        ))}
      </div>
    </Ctx.Provider>
  );
}
