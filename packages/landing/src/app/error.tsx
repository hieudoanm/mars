'use client';

import type { FC } from 'react';
import Link from 'next/link';
import { PiWarningCircle, PiArrowClockwise } from 'react-icons/pi';

type ErrorProps = Readonly<{
  error: Error & { digest?: string };
  reset: () => void;
}>;

const ErrorBoundary: FC<ErrorProps> = ({ error, reset }) => {
  return (
    <div className='bg-bg text-ink flex min-h-screen flex-col items-center justify-center px-6 py-20'>
      <div className='max-w-120 text-center'>
        <div className='bg-orange/10 text-orange-2 mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full'>
          <PiWarningCircle className='h-8 w-8' />
        </div>
        <h1 className='text-page font-bold-plus tracking-tightest mb-2 leading-tight'>
          Something went wrong
        </h1>
        <p className='text-muted mx-auto mb-8 max-w-80'>{error.message}</p>
        <div className='flex flex-col items-center gap-3 sm:flex-row'>
          <button
            className='border-orange bg-orange font-bold-plus text-ink-4 inline-flex items-center gap-2 rounded-lg border px-4.5 py-3.5 text-xs transition-transform duration-200 hover:-translate-y-0.5'
            onClick={() => reset()}>
            <PiArrowClockwise className='h-4 w-4' /> Try again
          </button>
          <Link
            className='border-line-2 text-ink font-bold-plus inline-flex items-center gap-2 rounded-lg border px-4.5 py-3.5 text-xs transition-transform duration-200 hover:-translate-y-0.5'
            href='/'>
            Back home
          </Link>
        </div>
      </div>
    </div>
  );
};

export default ErrorBoundary;
