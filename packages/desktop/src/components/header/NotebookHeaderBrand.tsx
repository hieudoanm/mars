import { type FC } from 'react';
import marsLogo from '../../assets/svg/mars-logo.svg';

type NotebookHeaderBrandProps = {
  compact?: boolean;
  product?: 'lab' | 'notebook';
  onHome?: () => void;
};

const productName = (product?: 'lab' | 'notebook'): string => {
  if (product === 'lab') return 'Mars Lab';
  if (product === 'notebook') return 'Mars Notebook';
  return 'Project Mars';
};

const NotebookHeaderBrand: FC<NotebookHeaderBrandProps> = ({
  compact = false,
  product,
  onHome,
}) => (
  <div data-component='NotebookHeaderBrand' className='flex min-w-0 items-center gap-3'>
    {onHome ? (
      <button
        type='button'
        aria-label='Back to product selection'
        title='Back to product selection'
        onClick={onHome}
        className='shrink-0 rounded-lg transition-opacity hover:opacity-80 focus-visible:outline-2 focus-visible:outline-orange-500'>
        <img className='h-9 w-9 rounded-lg object-cover' src={marsLogo} alt='Project Mars' />
      </button>
    ) : (
      <img className='h-9 w-9 shrink-0 rounded-lg object-cover' src={marsLogo} alt='Project Mars' />
    )}
    <div className='min-w-0'>
      <h1 className='truncate text-xl leading-tight font-semibold tracking-tight text-neutral-100'>
        {productName(product)}
      </h1>
      {compact ? null : (
        <span className='block truncate text-xs text-neutral-500'>Explore ideas.</span>
      )}
    </div>
  </div>
);

NotebookHeaderBrand.displayName = 'NotebookHeaderBrand';

export default NotebookHeaderBrand;
