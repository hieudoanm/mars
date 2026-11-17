'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const heading =
  'mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm';

const BuiltDifferent: FC = () => {
  return (
    <section
      className={`${sectionShell} max-md-nav:grid-cols-1 max-md-nav:gap-12 max-md-nav:py-25 grid grid-cols-2 items-center gap-25 py-38`}
      id='built-different'>
      <Reveal className='relative min-h-102 pt-9' y={22} scale={0.98}>
        <div className='border-line-7 bg-surface-4 shadow-card relative mx-auto -mb-0.5 w-4/5 translate-x-9 rounded-lg border px-6 py-4.5 transition-transform duration-500 hover:translate-x-10'>
          <span className='text-orange text-2xs tracking-caps-lg mb-1.5 block font-mono'>
            DESKTOP
          </span>
          <strong className='text-md-plus block'>Notebook UI</strong>
          <small className='text-xs-plus text-muted-4'>React · Tauri</small>
        </div>
        <div className='border-line-7 bg-surface-3 shadow-card relative mx-auto -mb-0.5 w-4/5 translate-x-4.5 rounded-lg border px-6 py-4.5 transition-transform duration-500 hover:translate-x-6'>
          <span className='text-orange text-2xs tracking-caps-lg mb-1.5 block font-mono'>
            KERNEL
          </span>
          <strong className='text-md-plus block'>Typed protocol</strong>
          <small className='text-xs-plus text-muted-4'>Requests · events · state</small>
        </div>
        <div className='border-line-7 bg-surface-2 shadow-card relative mx-auto -mb-0.5 w-4/5 rounded-lg border px-6 py-4.5 transition-transform duration-500 hover:translate-x-1'>
          <span className='text-orange text-2xs tracking-caps-lg mb-1.5 block font-mono'>
            RUNTIME
          </span>
          <strong className='text-md-plus block'>Rust execution</strong>
          <small className='text-xs-plus text-muted-4'>Isolated · persistent · resettable</small>
        </div>
        <div className='text-sm-plus text-muted-6 mt-7.5 leading-normal'>
          Clear boundaries.
          <br />
          <em className='text-orange-2 font-serif text-xl'>Better experiments.</em>
        </div>
      </Reveal>
      <Reveal delay={0.08}>
        <div className={eyebrow}>Built differently, on purpose</div>
        <h2 className={heading}>
          Not a web wrapper.
          <br />
          Not a black box.
        </h2>
        <p className='max-w-120'>
          Project Mars is designed from the document model outward. The UI, kernel, protocol,
          renderer, runtime, and CLI are separate pieces you can understand, test, and extend.
        </p>
        <ul className='mt-9 grid list-none gap-5 p-0'>
          <li className='flex items-start gap-3.5'>
            <span className='bg-mint-5 text-xs-plus text-mint-4 grid h-5 w-5 place-items-center rounded-full'>
              ✓
            </span>
            <div>
              <strong className='text-sm-plus block'>Rust-first foundation</strong>
              <small className='text-xs-plus text-muted-8 mt-1 block'>
                Core behavior is shared across desktop and CLI.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-3.5'>
            <span className='bg-mint-5 text-xs-plus text-mint-4 grid h-5 w-5 place-items-center rounded-full'>
              ✓
            </span>
            <div>
              <strong className='text-sm-plus block'>Reproducible execution</strong>
              <small className='text-xs-plus text-muted-8 mt-1 block'>
                Timeouts, lockfiles, offline, and deterministic controls.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-3.5'>
            <span className='bg-mint-5 text-xs-plus text-mint-4 grid h-5 w-5 place-items-center rounded-full'>
              ✓
            </span>
            <div>
              <strong className='text-sm-plus block'>Open interoperability</strong>
              <small className='text-xs-plus text-muted-8 mt-1 block'>
                Import and export representative Jupyter notebooks.
              </small>
            </div>
          </li>
        </ul>
      </Reveal>
    </section>
  );
};

export default BuiltDifferent;
