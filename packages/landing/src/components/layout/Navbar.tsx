'use client';

import type { FC } from 'react';
import { useEffect, useState } from 'react';
import Link from 'next/link';
import { repositoryUrl } from '../../lib/site';
import marsLogo from '@/assets/svg/mars-logo.svg';
import HamburgerButton from './HamburgerButton';
import NavLinks from './NavLinks';
import MobileDrawer from './MobileDrawer';

const headerShell =
  'relative z-20 mx-auto flex w-full flex-wrap items-center justify-between gap-y-3 px-6 py-7 max-sm-nav:px-4 max-sm-nav:py-5';

type NavbarProps = Readonly<{
  className?: string;
  showHome?: boolean;
}>;

const Navbar: FC<NavbarProps> = ({ className = headerShell, showHome = false }) => {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open]);

  const close = () => setOpen(false);

  return (
    <header
      className={`${className} max-md-nav:grid max-md-nav:grid-cols-3 max-md-nav:items-center`}>
      <HamburgerButton open={open} onClick={() => setOpen((v) => !v)} />
      <Link
        className='text-lg-plus font-bold-plus tracking-brand md-nav:justify-self-start inline-flex items-center gap-2.5 justify-self-center'
        href='/'
        aria-label='Project Mars home'>
        <img className='h-7.5 w-7.5 rounded-lg' src={marsLogo.src} alt='' />
        <span>Project Mars</span>
      </Link>
      <a
        className='text-orange-2 hover:text-ink text-sm-plus max-sm-nav:text-xs-plus md-nav:order-3 md-nav:justify-self-auto justify-self-end font-semibold'
        href={repositoryUrl}>
        GitHub <span aria-hidden='true'>↗</span>
      </a>
      <nav
        className='text-sm-plus md-nav:order-2 md-nav:w-auto md-nav:gap-7.5 md-nav:flex hidden w-full flex-wrap items-center justify-center gap-x-5 gap-y-2'
        aria-label='Primary navigation'>
        <NavLinks showHome={showHome} onNavigate={close} />
      </nav>
      <MobileDrawer open={open} onClose={close}>
        <NavLinks showHome={showHome} onNavigate={close} />
      </MobileDrawer>
    </header>
  );
};

export default Navbar;
