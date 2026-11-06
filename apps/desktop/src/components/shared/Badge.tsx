import type { FC, HTMLAttributes, ReactNode } from 'react';
import { cn } from './cn';

export type BadgeTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger';

export type BadgeProps = Omit<HTMLAttributes<HTMLSpanElement>, 'children'> & {
  children: ReactNode;
  tone?: BadgeTone;
  dot?: boolean;
  dotTone?: BadgeTone;
  dotClassName?: string;
};

const toneClasses: Record<BadgeTone, string> = {
  neutral: 'text-neutral-400',
  accent: 'text-orange-300',
  success: 'text-green-400',
  warning: 'text-orange-300',
  danger: 'text-red-300',
};

const dotToneClasses: Record<BadgeTone, string> = {
  neutral: 'bg-neutral-500',
  accent: 'bg-orange-500',
  success: 'bg-green-400',
  warning: 'bg-orange-400',
  danger: 'bg-red-400',
};

const Badge: FC<BadgeProps> = ({
  children,
  tone = 'neutral',
  dot = false,
  dotTone,
  dotClassName,
  className,
  ...props
}) => (
  <span
    data-component='Badge'
    className={cn('inline-flex items-center gap-2 text-xs', toneClasses[tone], className)}
    {...props}>
    {dot ? (
      <span
        aria-hidden='true'
        className={cn(
          'h-2 w-2 shrink-0 rounded-full',
          dotToneClasses[dotTone ?? tone],
          dotClassName,
        )}
      />
    ) : null}
    {children}
  </span>
);

Badge.displayName = 'Badge';

export default Badge;
