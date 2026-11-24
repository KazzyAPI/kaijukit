import { forwardRef, useId, type InputHTMLAttributes, type TextareaHTMLAttributes, type ReactNode } from 'react';

interface FieldProps {
  label: string;
  hint?: ReactNode;
  error?: ReactNode;
}

function useField({ hint, error }: Pick<FieldProps, 'hint' | 'error'>) {
  const id = useId();
  const describedBy = [error ? `${id}-e` : '', hint ? `${id}-h` : ''].filter(Boolean).join(' ') || undefined;
  return { id, describedBy };
}

function Wrap({ id, label, hint, error, children }: FieldProps & { id: string; children: ReactNode }) {
  return (
    <div className="kk-field">
      <label className="kk-field__label" htmlFor={id}>
        {label}
      </label>
      {children}
      {hint && (
        <span id={`${id}-h`} className="kk-field__hint">
          {hint}
        </span>
      )}
      {error && (
        <span id={`${id}-e`} className="kk-field__error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}

export const TextField = forwardRef<HTMLInputElement, FieldProps & InputHTMLAttributes<HTMLInputElement>>(function TextField({ label, hint, error, className, ...rest }, ref) {
  const { id, describedBy } = useField({ hint, error });
  return (
    <Wrap id={id} label={label} hint={hint} error={error}>
      <input ref={ref} id={id} className={`kk-input${className ? ` ${className}` : ''}`} aria-invalid={error ? true : undefined} aria-describedby={describedBy} {...rest} />
    </Wrap>
  );
});

export const TextArea = forwardRef<HTMLTextAreaElement, FieldProps & TextareaHTMLAttributes<HTMLTextAreaElement>>(function TextArea({ label, hint, error, className, ...rest }, ref) {
  const { id, describedBy } = useField({ hint, error });
  return (
    <Wrap id={id} label={label} hint={hint} error={error}>
      <textarea ref={ref} id={id} className={`kk-input${className ? ` ${className}` : ''}`} aria-invalid={error ? true : undefined} aria-describedby={describedBy} {...rest} />
    </Wrap>
  );
});
