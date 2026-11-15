'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const sectionShell =
  'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const heading =
  'mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm';

const BuiltDifferent: FC = () => {
  return (
    <section
      className={`${sectionShell} grid grid-cols-2 items-center gap-25 py-38 max-md-nav:grid-cols-1 max-md-nav:gap-12 max-md-nav:py-25`}
      id='built-different'>
      <Reveal className='relative min-h-102 pt-9' y={22} scale={0.98}>
        <div className='relative mx-auto -mb-0.5 w-4/5 translate-x-9 rounded-lg border border-line-7 bg-surface-4 px-6 py-4.5 shadow-card transition-transform duration-500 hover:translate-x-10'>
          <span className='text-orange mb-1.5 block font-mono text-2xs tracking-caps-lg'>
            DESKTOP
          </span>
          <strong className='block text-md-plus'>Notebook UI</strong>
          <small className='text-xs-plus text-muted-4'>React · Tauri</small>
        </div>
        <div className='relative mx-auto -mb-0.5 w-4/5 translate-x-4.5 rounded-lg border border-line-7 bg-surface-3 px-6 py-4.5 shadow-card transition-transform duration-500 hover:translate-x-6'>
          <span className='text-orange mb-1.5 block font-mono text-2xs tracking-caps-lg'>
            KERNEL
          </span>
          <strong className='block text-md-plus'>Typed protocol</strong>
          <small className='text-xs-plus text-muted-4'>Requests · events · state</small>
        </div>
        <div className='relative mx-auto -mb-0.5 w-4/5 rounded-lg border border-line-7 bg-surface-2 px-6 py-4.5 shadow-card transition-transform duration-500 hover:translate-x-1'>
          <span className='text-orange mb-1.5 block font-mono text-2xs tracking-caps-lg'>
            RUNTIME
          </span>
          <strong className='block text-md-plus'>Rust execution</strong>
          <small className='text-xs-plus text-muted-4'>
            Isolated · persistent · resettable
          </small>
        </div>
        <div className='mt-7.5 text-sm-plus leading-normal text-muted-6'>
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
          Project Mars is designed from the document model outward. The UI, kernel, protocol, renderer,
          runtime, and CLI are separate pieces you can understand, test, and extend.
        </p>
        <ul className='mt-9 grid list-none gap-5 p-0'>
          <li className='flex items-start gap-3.5'>
            <span className='grid h-5 w-5 place-items-center rounded-full bg-mint-5 text-xs-plus text-mint-4'>
              ✓
            </span>
            <div>
              <strong className='block text-sm-plus'>Rust-first foundation</strong>
              <small className='mt-1 block text-xs-plus text-muted-8'>
                Core behavior is shared across desktop and CLI.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-3.5'>
            <span className='grid h-5 w-5 place-items-center rounded-full bg-mint-5 text-xs-plus text-mint-4'>
              ✓
            </span>
            <div>
              <strong className='block text-sm-plus'>Reproducible execution</strong>
              <small className='mt-1 block text-xs-plus text-muted-8'>
                Timeouts, lockfiles, offline, and deterministic controls.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-3.5'>
            <span className='grid h-5 w-5 place-items-center rounded-full bg-mint-5 text-xs-plus text-mint-4'>
              ✓
            </span>
            <div>
              <strong className='block text-sm-plus'>Open interoperability</strong>
              <small className='mt-1 block text-xs-plus text-muted-8'>
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
