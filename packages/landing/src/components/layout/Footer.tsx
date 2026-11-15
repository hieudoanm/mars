import type { FC } from 'react';
import Link from 'next/link';
import marsLogo from '@/assets/svg/mars-logo.svg';

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';

type FooterProps = Readonly<{
  className?: string;
}>;

const Footer: FC<FooterProps> = ({
  className = `${sectionShell} text-xs-plus text-faint-2 max-sm-nav:flex-col max-sm-nav:items-start max-sm-nav:py-6 flex items-center justify-between gap-5 py-7.5`,
}) => {
  return (
    <footer className={className}>
      <Link
        className='text-lg-plus font-bold-plus tracking-brand text-ink-3 inline-flex items-center gap-2.5'
        href='/'>
        <img className='h-6 w-6 rounded-lg' src={marsLogo.src} alt='' />
        <span>Project Mars</span>
      </Link>
      <span>Explore ideas. Execute code. Discover possibilities.</span>
      <span>MIT licensed · Built with Rust</span>
    </footer>
  );
};

export default Footer;
