'use client';

import type { FC } from 'react';
import { HiExclamationCircle, HiRefresh } from 'react-icons/hi';

type GlobalErrorProps = Readonly<{
  error: Error & { digest?: string };
  reset: () => void;
}>;

const GlobalError: FC<GlobalErrorProps> = ({ error, reset }) => {
  return (
    <html>
      <body className='bg-bg text-ink'>
        <div className='flex min-h-screen flex-col items-center justify-center px-6 py-20'>
          <div className='max-w-120 text-center'>
            <div className='bg-orange/10 text-orange-2 mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full'>
              <HiExclamationCircle className='h-8 w-8' />
            </div>
            <h1 className='mb-2 text-page leading-tight font-bold-plus tracking-tightest'>
              Something went wrong
            </h1>
            <p className='text-muted mx-auto mb-8 max-w-80'>{error.message}</p>
            <button
              className='border-orange bg-orange inline-flex items-center gap-2 rounded-lg border px-4.5 py-3.5 text-xs font-bold-plus text-ink-4 transition-transform duration-200 hover:-translate-y-0.5'
              onClick={() => reset()}>
              <HiRefresh className='h-4 w-4' /> Try again
            </button>
          </div>
        </div>
      </body>
    </html>
  );
};

export default GlobalError;
