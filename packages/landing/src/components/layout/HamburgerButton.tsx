import type { FC } from 'react';
import { HiMenu, HiX } from 'react-icons/hi';

type HamburgerButtonProps = Readonly<{
  open: boolean;
  onClick: () => void;
}>;

const HamburgerButton: FC<HamburgerButtonProps> = ({ open, onClick }) => (
  <button
    type='button'
    className='md-nav:hidden text-muted hover:text-ink inline-flex items-center justify-center justify-self-start rounded-md p-2'
    aria-label='Toggle navigation menu'
    aria-expanded={open}
    aria-controls='mobile-navigation'
    onClick={onClick}>
    {open ? <HiX className='h-6 w-6' /> : <HiMenu className='h-6 w-6' />}
  </button>
);

export default HamburgerButton;
