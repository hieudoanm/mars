import type { FC } from 'react';
import { HiMenu, HiX } from 'react-icons/hi';

type HamburgerButtonProps = Readonly<{
  open: boolean;
  onClick: () => void;
}>;

const HamburgerButton: FC<HamburgerButtonProps> = ({ open, onClick }) => (
  <button
    type='button'
    className='md-nav:hidden justify-self-start inline-flex items-center justify-center rounded-md p-2 text-muted hover:text-ink'
    aria-label='Toggle navigation menu'
    aria-expanded={open}
    aria-controls='mobile-navigation'
    onClick={onClick}>
    {open ? <HiX className='h-6 w-6' /> : <HiMenu className='h-6 w-6' />}
  </button>
);

export default HamburgerButton;
