import type { NextPage } from 'next';
import Link from 'next/link';
import { PiArrowLeft, PiHouse } from 'react-icons/pi';

import marsLogo from '@/assets/svg/mars-logo.svg';

const NotFound: NextPage = () => {
  return (
    <div className='bg-bg text-ink flex min-h-screen flex-col items-center justify-center px-6 py-20'>
      <div className='max-w-120 text-center'>
        <img className='mx-auto mb-6 h-12 w-12' src={marsLogo.src} alt='' />
        <p className='text-orange-2 text-xs-plus font-bold-plus tracking-caps-xl mb-2 uppercase'>
          404
        </p>
        <h1 className='text-page font-bold-plus tracking-tightest mb-2 leading-tight'>
          Lost in space
        </h1>
        <p className='text-muted mx-auto mb-8 max-w-80'>
          The page you are looking for does not exist or has been moved.
        </p>
        <div className='flex flex-wrap items-center justify-center gap-3'>
          <Link
            className='border-orange bg-orange font-bold-plus text-ink-4 inline-flex items-center gap-2 rounded-lg border px-4.5 py-3.5 text-xs transition-transform duration-200 hover:-translate-y-0.5'
            href='/'>
            <PiHouse className='h-4 w-4' /> Go home
          </Link>
          <Link
            className='border-line-2 text-ink font-bold-plus inline-flex items-center gap-2 rounded-lg border px-4.5 py-3.5 text-xs transition-transform duration-200 hover:-translate-y-0.5'
            href='/docs'>
            <PiArrowLeft className='h-4 w-4' /> Back to docs
          </Link>
        </div>
      </div>
    </div>
  );
};

export default NotFound;
