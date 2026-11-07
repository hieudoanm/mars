import type { FC, HTMLAttributes, ReactNode } from 'react';
import { cn } from './cn';

export type KbdProps = Omit<HTMLAttributes<HTMLElement>, 'children'> & {
  children: ReactNode;
};

const Kbd: FC<KbdProps> = ({ children, className, ...props }) => (
  <kbd data-component='Kbd' className={cn('text-xs text-neutral-500', className)} {...props}>
    {children}
  </kbd>
);

Kbd.displayName = 'Kbd';

export default Kbd;
