import type { FC, ReactNode } from 'react';
import { cn } from './cn';

export type DialogProps = {
  labelledBy: string;
  children: ReactNode;
  className?: string;
  component?: string;
};

const Dialog: FC<DialogProps> = ({ labelledBy, children, className, component = 'Dialog' }) => (
  <div
    data-component={component}
    className='fixed inset-0 grid place-items-center bg-black/65'
    role='presentation'>
    <section
      className={cn(
        'w-full max-w-md rounded-xl border border-orange-950 bg-neutral-900 p-6 shadow-2xl',
        className,
      )}
      role='dialog'
      aria-modal='true'
      aria-labelledby={labelledBy}>
      {children}
    </section>
  </div>
);

Dialog.displayName = 'Dialog';

export default Dialog;
