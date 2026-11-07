import type { ButtonHTMLAttributes, FC, ReactNode } from 'react';
import { cn } from './cn';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg' | 'icon';
export type ButtonAlign = 'center' | 'start';

export type ButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> & {
  children?: ReactNode;
  variant?: ButtonVariant;
  size?: ButtonSize;
  align?: ButtonAlign;
  loading?: boolean;
};

const variantClasses: Record<ButtonVariant, string> = {
  primary: 'btn-primary',
  secondary: 'btn-neutral',
  ghost: 'btn-ghost',
  danger: 'btn-soft btn-error',
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'h-6 rounded-md px-2.5 text-xs',
  md: 'h-7 rounded-lg px-3 text-sm',
  lg: 'h-8 rounded-lg px-4 text-base',
  icon: 'size-7 rounded-lg p-0',
};

const alignClasses: Record<ButtonAlign, string> = {
  center: 'justify-center',
  start: 'justify-start',
};

const Button: FC<ButtonProps> = ({
  children,
  variant = 'secondary',
  size = 'md',
  align = 'center',
  loading = false,
  disabled = false,
  type = 'button',
  className,
  ...props
}) => {
  const isDisabled = disabled || loading;

  return (
    <button
      data-component='Button'
      type={type}
      className={cn(
        'btn inline-flex items-center gap-2 font-medium transition-colors disabled:cursor-not-allowed',
        variantClasses[variant],
        sizeClasses[size],
        alignClasses[align],
        className,
      )}
      disabled={isDisabled}
      aria-busy={loading || undefined}
      {...props}>
      {loading ? <span aria-hidden='true'>Loading...</span> : null}
      {children}
    </button>
  );
};

Button.displayName = 'Button';

export default Button;
