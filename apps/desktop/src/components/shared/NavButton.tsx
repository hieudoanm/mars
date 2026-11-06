import type { ButtonHTMLAttributes, FC, ReactNode } from 'react';
import { cn } from './cn';

export type NavButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> & {
  children: ReactNode;
  active?: boolean;
  index?: ReactNode;
};

const NavButton: FC<NavButtonProps> = ({
  children,
  active = false,
  index,
  className,
  type = 'button',
  ...props
}) => (
  <button
    data-component='NavButton'
    type={type}
    aria-current={active ? 'true' : undefined}
    className={cn(
      'my-0.5 flex w-full items-center gap-3 rounded-md px-2 py-2 text-left text-sm transition-colors',
      active
        ? 'bg-orange-950 text-white'
        : 'bg-transparent text-neutral-400 hover:bg-neutral-800 hover:text-white',
      className,
    )}
    {...props}>
    {index !== undefined ? <span className='text-neutral-600 tabular-nums'>{index}</span> : null}
    {children}
  </button>
);

NavButton.displayName = 'NavButton';

export default NavButton;
