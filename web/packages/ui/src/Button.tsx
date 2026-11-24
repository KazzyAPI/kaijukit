import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { Spinner } from './Misc';
import { Tooltip } from './Tooltip';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  appearance?: 'default' | 'primary' | 'subtle' | 'danger';
  loading?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { appearance = 'default', loading, className, children, onClick, type = 'button', ...rest },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={`kk-btn${appearance === 'default' ? '' : ` kk-btn--${appearance}`}${className ? ` ${className}` : ''}`}
      aria-busy={loading || undefined}
      onClick={loading ? undefined : onClick}
      {...rest}
    >
      {loading && <Spinner />}
      {children}
    </button>
  );
});

export interface IconButtonProps extends Omit<ButtonProps, 'children' | 'aria-label'> {
  label: string;
  icon: ReactNode;
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton({ label, icon, className, appearance = 'subtle', ...rest }, ref) {
  return (
    <Tooltip content={label}>
      <Button ref={ref} appearance={appearance} aria-label={label} className={`kk-btn--icon${className ? ` ${className}` : ''}`} {...rest}>
        {icon}
      </Button>
    </Tooltip>
  );
});
