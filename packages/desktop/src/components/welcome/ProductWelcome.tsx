import type { FC } from 'react';
import type { ProductView } from '../../lib/product';

type ProductWelcomeProps = {
  onSelect: (view: Exclude<ProductView, 'welcome'>) => void;
};

const ProductCard: FC<{
  title: string;
  tagline: string;
  description: string;
  points: string[];
  onClick: () => void;
}> = ({ title, tagline, description, points, onClick }) => (
  <button
    type='button'
    onClick={onClick}
    className='group w-72 rounded-2xl border border-neutral-800 bg-neutral-900/60 p-6 text-left transition-all hover:border-orange-700 hover:bg-neutral-900'>
    <h2 className='text-lg font-semibold text-neutral-100'>{title}</h2>
    <p className='mt-1 text-sm text-orange-500'>{tagline}</p>
    <p className='mt-3 text-sm leading-6 text-neutral-400'>{description}</p>
    <ul className='mt-4 space-y-1.5'>
      {points.map((point) => (
        <li key={point} className='flex items-center gap-2 text-xs text-neutral-500'>
          <span className='text-orange-500'>•</span>
          {point}
        </li>
      ))}
    </ul>
    <span className='mt-5 inline-block text-sm font-medium text-neutral-300 transition-colors group-hover:text-orange-400'>
      Open →
    </span>
  </button>
);

const ProductWelcome: FC<ProductWelcomeProps> = ({ onSelect }) => {
  return (
    <div
      data-component='ProductWelcome'
      className='flex h-full flex-col items-center justify-center gap-10 bg-neutral-950 p-8'>
      <div className='text-center'>
        <h1 className='text-3xl font-bold tracking-tight text-neutral-100'>Welcome to Project Mars</h1>
        <p className='mt-2 text-sm text-neutral-500'>Explore ideas. Execute code. Discover possibilities.</p>
      </div>
      <div className='flex flex-wrap items-stretch justify-center gap-6'>
        <ProductCard
          title='Mars Lab'
          tagline='Machine Learning in Rust'
          description='Turn CSV datasets into trained models and export them as ONNX — all in Rust, no Python required.'
          points={['83 algorithms · 6 task types', 'SQLite job history', 'CSV → ONNX in one click']}
          onClick={() => onSelect('lab')}
        />
        <ProductCard
          title='Mars Notebook'
          tagline='Interactive computing'
          description='A Jupyter-style notebook with a Rust-first runtime, multi-language cells, and rich outputs.'
          points={['Rust, Python, R, Julia, C++', '.mars format + .ipynb interop', 'LSP, widgets, MIME outputs']}
          onClick={() => onSelect('notebook')}
        />
      </div>
    </div>
  );
};

ProductWelcome.displayName = 'ProductWelcome';
export default ProductWelcome;
