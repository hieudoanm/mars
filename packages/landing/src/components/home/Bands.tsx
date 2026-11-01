'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const sectionShell =
  'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';

const Bands: FC = () => {
  return (
    <Reveal>
      <section className='border-line border-y bg-panel-4'>
        <div
          className={`${sectionShell} text-faint flex min-h-18 items-center justify-between text-xs-plus tracking-caps uppercase max-sm-nav:block max-sm-nav:py-4.5`}>
          <span className='max-sm-nav:max-w-32'>Made for people who think in systems</span>
          <div className='flex gap-7 font-bold text-muted-3 max-md-nav:gap-2.5 max-md-nav:text-2xs max-sm-nav:mt-3 max-sm-nav:justify-between'>
            <span>SCIENCE</span>
            <span>DATA</span>
            <span>LEARNING</span>
            <span>EXPERIMENTS</span>
          </div>
        </div>
      </section>
    </Reveal>
  );
};

export default Bands;
