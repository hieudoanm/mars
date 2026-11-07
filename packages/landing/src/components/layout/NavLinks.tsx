import type { FC } from 'react';
import Link from 'next/link';

type NavLinksProps = Readonly<{
  showHome?: boolean;
  onNavigate?: () => void;
}>;

const navLinkClassName = 'text-muted hover:text-ink';

const NavLinks: FC<NavLinksProps> = ({ showHome = false, onNavigate }) => (
  <>
    {showHome ? (
      <Link className={navLinkClassName} href='/' onClick={onNavigate}>
        Home
      </Link>
    ) : null}
    <Link className={navLinkClassName} href='/#why-mars' onClick={onNavigate}>
      Why Mars
    </Link>
    <Link className={navLinkClassName} href='/#workflow' onClick={onNavigate}>
      Workflow
    </Link>
    <Link className={navLinkClassName} href='/#built-different' onClick={onNavigate}>
      Architecture
    </Link>
    <Link className={navLinkClassName} href='/#get-started' onClick={onNavigate}>
      Get started
    </Link>
    <span className='text-line-5 max-md-nav:hidden select-none' aria-hidden='true'>
      |
    </span>
    <span className='flex items-center gap-5'>
      <Link className={navLinkClassName} href='/blog' onClick={onNavigate}>
        Blog
      </Link>
      <Link className={navLinkClassName} href='/docs' onClick={onNavigate}>
        Docs
      </Link>
      <Link className={navLinkClassName} href='/downloads' onClick={onNavigate}>
        Downloads
      </Link>
    </span>
  </>
);

export default NavLinks;
