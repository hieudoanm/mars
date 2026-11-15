'use client';

import type { FC } from 'react';
import { motion, useReducedMotion } from 'motion/react';
import { repositoryUrl } from '../../lib/site';
import marsLogo from '@/assets/svg/mars-logo.svg';

const sectionShell =
  'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const buttonBase =
  'inline-flex items-center gap-3.5 rounded-lg border px-4.5 py-3.5 text-sm-plus font-bold-plus transition-all duration-200 hover:-translate-y-0.5 hover:border-orange-8';
const buttonPrimary = `${buttonBase} border-orange bg-orange text-ink-4 shadow-btn-orange hover:hover:bg-orange-3`;
const buttonQuiet = `${buttonBase} border-line bg-surface-1 text-[#c8d0db]`;
const ease = [0.22, 1, 0.36, 1] as const;

const Hero: FC = () => {
  const reduce = useReducedMotion();
  const reveal = (delay: number) =>
    reduce
      ? {}
      : {
          initial: { opacity: 0, y: 24 },
          animate: { opacity: 1, y: 0 },
          transition: { duration: 0.65, delay, ease },
        };

  return (
    <section
      className={`${sectionShell} relative grid min-h-160 grid-cols-1 items-center gap-12 pt-20 pb-27 max-md-nav:pt-14 max-sm-nav:pt-12 max-sm-nav:pb-19 lg:grid-cols-10`}>
      <div className='relative z-10 lg:col-span-5'>
        <motion.div className={eyebrow} {...reveal(0)}>
          <span className='bg-orange h-2 w-2 rounded-full shadow-glow-orange' />{' '}
          Rust-powered interactive notebooks
        </motion.div>
        <motion.h1
          className='mt-6 mb-5.5 max-w-prose text-hero leading-none font-bold-plus tracking-tightest max-md-nav:max-md-nav:text-hero-md max-sm-nav:max-sm-nav:text-hero-sm'
          {...reveal(0.08)}>
          Think in cells.
          <br />
          <em className='text-orange-2 font-serif font-normal'>Build in Rust.</em>
        </motion.h1>
        <motion.p className='m-0 max-w-prose-md text-lg max-sm-nav:text-base' {...reveal(0.16)}>
          Project Mars is a focused environment for exploring ideas, executing code, and discovering
          possibilities - without giving up the power of a real developer tool.
        </motion.p>
        <motion.div className='my-8 flex flex-wrap gap-3' {...reveal(0.24)}>
          <a className={buttonPrimary} href='#get-started'>
            Start exploring <span aria-hidden='true'>→</span>
          </a>
          <a className={buttonQuiet} href={repositoryUrl}>
            View on GitHub <span aria-hidden='true'>↗</span>
          </a>
        </motion.div>
        <motion.div
          className='text-faint flex flex-wrap gap-4.5 text-xs-plus max-sm-nav:grid max-sm-nav:gap-2'
          {...reveal(0.32)}>
          <span className='flex items-center gap-1.5'>
            <span className='text-mint font-extrabold'>✓</span> Native <code>.mars</code> format
          </span>
          <span className='flex items-center gap-1.5'>
            <span className='text-mint font-extrabold'>✓</span> Isolated Rust execution
          </span>
          <span className='flex items-center gap-1.5'>
            <span className='text-mint font-extrabold'>✓</span> Jupyter import/export
          </span>
        </motion.div>
      </div>
      <motion.div
        className='relative grid h-125 place-items-center max-md-nav:h-98 max-sm-nav:h-82 lg:col-span-5'
        aria-label='Project Mars notebook interface preview'
        {...(reduce
          ? {}
          : {
              initial: { opacity: 0, scale: 0.96, y: 18 },
              animate: { opacity: 1, scale: 1, y: 0 },
              transition: { duration: 0.8, delay: 0.12, ease },
            })}>
        <img
          className='pointer-events-none absolute h-90 w-90 opacity-80 max-sm-nav:h-62 max-sm-nav:w-62'
          src='/icons/mars-orbit.svg'
          alt=''
          aria-hidden='true'
        />
        <div className='absolute h-48 w-132 -rotate-25 rounded-full border border-orange/25 max-sm-nav:w-90' />
        <div className='absolute h-35 w-112 rotate-36 rounded-full border border-slate-400/20 max-sm-nav:w-75' />
        <div className='animate-float relative z-10 h-75 w-75 drop-shadow-hero max-sm-nav:h-55 max-sm-nav:w-55'>
          <img className='h-full w-full' src={marsLogo.src} alt='Project Mars logo' />
        </div>
        <motion.div
          className='absolute top-18 right-0 z-20 rounded-lg border border-line-8 bg-surface-5 px-4 py-3.5 font-mono text-xs leading-relaxed shadow-code max-sm-nav:top-6 max-sm-nav:-right-2 max-sm-nav:text-2xs'
          {...(reduce
            ? {}
            : {
                initial: { opacity: 0, x: 18, y: -8 },
                animate: { opacity: 1, x: 0, y: 0 },
                transition: { duration: 0.55, delay: 0.45, ease },
              })}>
          <span className='mb-2.5 flex gap-1'>
            <i className='h-1.5 w-1.5 rounded-full bg-orange-8' />
            <i className='h-1.5 w-1.5 rounded-full bg-gold-1' />
            <i className='h-1.5 w-1.5 rounded-full bg-mint' />
          </span>
          <code>
            <b>let</b> ideas = <span className='text-orange-5'>explore</span>();
          </code>
        </motion.div>
        <motion.div
          className='absolute bottom-18 left-0.5 z-20 rounded-lg border border-line-8 bg-surface-5 px-4 py-3.5 font-mono text-xs leading-relaxed shadow-code max-sm-nav:bottom-5 max-sm-nav:-left-2 max-sm-nav:text-2xs'
          {...(reduce
            ? {}
            : {
                initial: { opacity: 0, x: -18, y: 8 },
                animate: { opacity: 1, x: 0, y: 0 },
                transition: { duration: 0.55, delay: 0.55, ease },
              })}>
          <span className='mb-1.5 block text-2xs tracking-caps-sm text-muted-5'>
            OUT [03]
          </span>
          <code>
            <span className='text-mint-2'>discoveries</span> = 42
          </code>
        </motion.div>
        <span className='absolute top-1/4 left-1/10 text-sm-plus text-orange-7'>✦</span>
        <span className='absolute top-1/3 right-1/12 text-2xs text-orange-7'>✦</span>
        <span className='absolute right-1/5 bottom-1/5 text-2xl text-orange-7'>·</span>
      </motion.div>
    </section>
  );
};

export default Hero;
