'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';
import { repositoryUrl } from '../../lib/site';
import marsLogoFile from '@/assets/svg/mars-logo-file.svg';

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const heading =
  'mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm';
const buttonBase =
  'inline-flex items-center gap-3.5 rounded-lg border px-4.5 py-3.5 text-sm-plus font-bold-plus transition-all duration-200 hover:-translate-y-0.5 hover:border-orange-8';
const buttonPrimary = `${buttonBase} border-orange bg-orange text-ink-4 shadow-btn-orange hover:hover:bg-orange-3`;
const buttonQuiet = `${buttonBase} border-line bg-surface-1 text-[#c8d0db]`;

const GetStarted: FC = () => {
  return (
    <section
      className='border-line-3 bg-surface-1 relative overflow-hidden border-t'
      id='get-started'>
      <div className='animate-drift-reverse bg-cta-glow absolute right-1/8 -bottom-58 h-120 w-120 rounded-full' />
      <Reveal
        className={`${sectionShell} max-sm-nav:gap-5 max-sm-nav:py-19 md-nav:grid-cols-12 relative grid grid-cols-1 items-center gap-7.5 py-26`}
        y={20}>
        <img
          className='animate-float drop-shadow-float max-sm-nav:h-18 max-sm-nav:w-18 md-nav:col-span-2 h-23 w-23'
          src={marsLogoFile.src}
          alt=''
        />
        <div className='md-nav:col-span-7'>
          <div className={eyebrow}>The surface is waiting</div>
          <h2 className={`${heading} mb-3`}>
            Give your next idea
            <br />
            <em className='text-orange-2 font-serif font-normal'>room to move.</em>
          </h2>
          <p className='text-sm-plus m-0'>
            Open source. Rust-powered. Built for the moment before the answer.
          </p>
        </div>
        <div className='md-nav:col-span-3 flex flex-col items-stretch gap-2.5'>
          <a className={`${buttonPrimary} justify-center whitespace-nowrap`} href={repositoryUrl}>
            View the repository <span>↗</span>
          </a>
          <a className={`${buttonQuiet} justify-center whitespace-nowrap`} href='/docs'>
            Read the docs <span>→</span>
          </a>
        </div>
      </Reveal>
    </section>
  );
};

export default GetStarted;
