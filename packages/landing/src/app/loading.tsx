import type { FC } from 'react';
import type { NextPage } from 'next';
import marsLogo from '@/assets/svg/mars-logo.svg';

const Loading: FC & NextPage = () => {
  return (
    <div className='bg-bg text-ink flex min-h-screen flex-col items-center justify-center'>
      <div className='flex flex-col items-center gap-4'>
        <div className='relative'>
          <img className='h-12 w-12 animate-pulse' src={marsLogo.src} alt='' />
          <span className='border-orange absolute inset-0 animate-spin rounded-full border-2 border-t-transparent' />
        </div>
        <p className='text-muted text-sm-plus'>Loading Mars…</p>
      </div>
    </div>
  );
};

export default Loading;
