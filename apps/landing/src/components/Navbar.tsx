import { repositoryUrl } from '../lib/site';

const headerShell =
  'relative z-[2] mx-auto flex w-[min(calc(100%_-_48px),1240px)] items-center justify-between py-7 max-[520px]:w-[min(calc(100%_-_32px),1240px)] max-[520px]:py-5';

export default function Navbar() {
  return (
    <header className={headerShell}>
      <a
        className='inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em]'
        href='#'
        aria-label='Mars home'>
        <img className='h-[30px] w-[30px] rounded-lg' src='/images/svg/mars-logo.svg' alt='' />
        <span>Mars</span>
      </a>
      <nav
        className='text-muted flex gap-[30px] text-[13px] max-[850px]:hidden'
        aria-label='Primary navigation'>
        <a className='hover:text-ink' href='#why-mars'>
          Why Mars
        </a>
        <a className='hover:text-ink' href='#workflow'>
          Workflow
        </a>
        <a className='hover:text-ink' href='#built-different'>
          Architecture
        </a>
        <a className='hover:text-ink' href='/docs'>
          Docs
        </a>
        <a className='hover:text-ink' href='#get-started'>
          Get started
        </a>
      </nav>
      <a
        className='text-orange-2 hover:text-ink text-[13px] font-semibold max-[520px]:text-[11px]'
        href={repositoryUrl}>
        GitHub <span aria-hidden='true'>↗</span>
      </a>
    </header>
  );
}
