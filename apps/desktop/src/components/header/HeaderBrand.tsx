import { type FC } from 'react';
import marsLogo from '../../assets/svg/mars-logo.svg';

const HeaderBrand: FC = () => (
  <div data-component='HeaderBrand' className='flex min-w-0 items-center gap-3'>
    <span className='grid h-11 w-11 shrink-0 place-items-center rounded-xl border border-orange-900/70 bg-orange-950/40 shadow-sm'>
      <img
        className='h-7 w-7 rounded-md object-cover'
        src={marsLogo}
        alt='Mars'
      />
    </span>
    <div className='min-w-0'>
      <span className='block truncate text-[11px] font-semibold tracking-[0.2em] text-orange-300 uppercase'>
        MARS
      </span>
      <h1 className='truncate text-lg font-medium tracking-tight text-neutral-100'>
        Explore ideas.
      </h1>
    </div>
  </div>
);

HeaderBrand.displayName = 'HeaderBrand';

export default HeaderBrand;
