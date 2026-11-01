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
  primary:
    'border-orange-600 bg-orange-700 text-white hover:bg-orange-600 disabled:border-orange-900 disabled:bg-orange-950 disabled:text-orange-200',
  secondary:
    'border-neutral-700 bg-neutral-800 text-neutral-300 hover:border-orange-500 hover:text-white disabled:border-neutral-800 disabled:bg-neutral-900 disabled:text-neutral-500',
  ghost:
    'border-transparent bg-transparent text-neutral-300 hover:border-neutral-700 hover:bg-neutral-800 hover:text-white disabled:text-neutral-600 disabled:hover:border-transparent disabled:hover:bg-transparent',
  danger:
    'border-red-900 bg-red-950 text-red-200 hover:border-red-700 hover:bg-red-900 hover:text-white disabled:border-red-950 disabled:bg-neutral-900 disabled:text-red-900',
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'rounded-md px-2 py-1 text-xs',
  md: 'rounded-lg px-3 py-2 text-sm',
  lg: 'rounded-lg px-4 py-2.5 text-base',
  icon: 'rounded-lg p-2',
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
        'inline-flex items-center gap-2 border font-medium transition-colors focus:outline-2 focus:outline-offset-2 focus:outline-orange-500 disabled:cursor-not-allowed',
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
