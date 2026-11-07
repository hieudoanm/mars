'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const sectionShell =
  'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const heading =
  'mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm';

const workflow = [
  ['01', 'Write', 'Capture the thought in code or Markdown.'],
  ['02', 'Run', 'Execute Rust with persistent bindings.'],
  ['03', 'See', 'Turn results into rich, useful output.'],
  ['04', 'Share', 'Export, reproduce, and keep moving.'],
];

const Workflow: FC = () => {
  return (
    <section className='border-y border-line-2 bg-panel-5' id='workflow'>
      <div className={`${sectionShell} py-31 pb-34 max-sm-nav:py-22`}>
        <Reveal className='mx-auto max-w-prose-sm text-center'>
          <div className={`${eyebrow} justify-center`}>One document. Many ways forward.</div>
          <h2 className={heading}>
            A workflow that follows
            <br />
            <em className='text-orange-2 font-serif font-normal'>your curiosity.</em>
          </h2>
        </Reveal>
        <div className='mt-18 flex items-stretch max-md-nav:grid max-md-nav:grid-cols-2 max-md-nav:gap-5 max-sm-nav:grid-cols-1'>
          {workflow.map(([number, title, body], index) => (
            <div className='contents' key={title}>
              <Reveal className='flex-1' delay={0.06 + index * 0.08}>
                <div
                  className={`h-full border-t px-5.5 py-5 transition-colors duration-300 ${index === 0 ? 'border-orange border-t-2' : 'border-line-4'}`}>
                  <span className='text-orange mb-8 block font-mono text-xs'>{number}</span>
                  <strong className='text-lg'>{title}</strong>
                  <p className='mb-0 text-xs'>{body}</p>
                </div>
              </Reveal>
              {index < workflow.length - 1 && (
                <div className='mt-0 w-4.5 border-t border-dashed border-line-6 max-md-nav:hidden' />
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Workflow;
