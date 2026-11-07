import { type FC } from 'react';
import marsLogo from '../../assets/svg/mars-logo.svg';

type NotebookHeaderBrandProps = {
  compact?: boolean;
};

const NotebookHeaderBrand: FC<NotebookHeaderBrandProps> = ({ compact = false }) => (
  <div data-component='NotebookHeaderBrand' className='flex min-w-0 items-center gap-3'>
    <img className='h-9 w-9 shrink-0 rounded-lg object-cover' src={marsLogo} alt='Mars' />
    <div className='min-w-0'>
      <h1 className='truncate text-xl leading-tight font-semibold tracking-tight text-neutral-100'>
        Mars
      </h1>
      {compact ? null : (
        <span className='block truncate text-xs text-neutral-500'>Explore ideas.</span>
      )}
    </div>
  </div>
);

NotebookHeaderBrand.displayName = 'NotebookHeaderBrand';

export default NotebookHeaderBrand;
