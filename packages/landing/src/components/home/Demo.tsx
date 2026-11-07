'use client';

import type { FC } from 'react';
import { useState, useEffect } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const Demo: FC = () => {
  const [activeCell, setActiveCell] = useState(0);
  const [cellStates, setCellStates] = useState([
    { executing: false, executed: false, output: '' },
    { executing: false, executed: false, output: '' },
    { executing: false, executed: false, output: '' },
  ]);

  useEffect(() => {
    const cycle = async () => {
      while (true) {
        // Reset
        setCellStates([
          { executing: false, executed: false, output: '' },
          { executing: false, executed: false, output: '' },
          { executing: false, executed: false, output: '' },
        ]);
        setActiveCell(0);

        // Wait before starting
        await new Promise(resolve => setTimeout(resolve, 1000));

        // Cell 1
        setActiveCell(0);
        setCellStates(prev => {
          const newStates = [...prev];
          newStates[0] = { executing: true, executed: false, output: '' };
          return newStates;
        });
        await new Promise(resolve => setTimeout(resolve, 1500));
        setCellStates(prev => {
          const newStates = [...prev];
          newStates[0] = { executing: false, executed: true, output: 'Signal loaded: 1024 samples' };
          return newStates;
        });
        await new Promise(resolve => setTimeout(resolve, 500));

        // Cell 2
        setActiveCell(1);
        setCellStates(prev => {
          const newStates = [...prev];
          newStates[1] = { executing: true, executed: false, output: '' };
          return newStates;
        });
        await new Promise(resolve => setTimeout(resolve, 1500));
        setCellStates(prev => {
          const newStates = [...prev];
          newStates[1] = { executing: false, executed: true, output: 'Mean calculated: 42.3' };
          return newStates;
        });
        await new Promise(resolve => setTimeout(resolve, 500));

        // Cell 3
        setActiveCell(2);
        setCellStates(prev => {
          const newStates = [...prev];
          newStates[2] = { executing: true, executed: false, output: '' };
          return newStates;
        });
        await new Promise(resolve => setTimeout(resolve, 1500));
        setCellStates(prev => {
          const newStates = [...prev];
          newStates[2] = { executing: false, executed: true, output: 'Chart rendered in 12ms' };
          return newStates;
        });
        await new Promise(resolve => setTimeout(resolve, 2000));
      }
    };

    cycle();
  }, []);

  return (
    <section className='mx-auto w-full max-w-shell px-6 py-30 max-sm-nav:px-4 max-sm-nav:py-22'>
      <Reveal className='text-center'>
        <div className='flex items-center justify-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2'>
          <span className='bg-orange shadow-glow-orange h-2 w-2 rounded-full' />
          See it in action
        </div>
        <h2 className='mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm'>
          A notebook environment that
          <br />
          feels <em className='text-orange-2 font-serif font-normal'>instant.</em>
        </h2>
        <p className='max-w-prose-lg mx-auto text-base text-muted-2'>
          Execute cells, inspect variables, and iterate on ideas without leaving your flow.
        </p>
      </Reveal>

      <Reveal delay={0.15} className='mt-14'>
        <div className='relative mx-auto max-w-5xl rounded-xl border border-line-5 bg-surface-6 p-4 shadow-2xl'>
          {/* Window controls */}
          <div className='mb-4 flex items-center gap-2'>
            <div className='h-3 w-3 rounded-full bg-red-500' />
            <div className='h-3 w-3 rounded-full bg-yellow-500' />
            <div className='h-3 w-3 rounded-full bg-green-500' />
            <div className='ml-4 text-xs-plus text-faint-4'>analysis.mars — Mars</div>
          </div>

          {/* Demo content */}
          <div className='grid grid-cols-12 gap-4'>
            {/* Sidebar */}
            <div className='col-span-3 hidden rounded-lg bg-panel-3 p-3 md-nav:block'>
              <div className='mb-3 text-xs font-bold text-faint-3'>FILES</div>
              <div className='space-y-1'>
                <div className='rounded bg-orange/20 px-2 py-1.5 text-xs text-orange-3'>analysis.mars</div>
                <div className='rounded px-2 py-1.5 text-xs text-faint-4'>data.csv</div>
                <div className='rounded px-2 py-1.5 text-xs text-faint-4'>notes.md</div>
              </div>
              <div className='mt-4 mb-3 text-xs font-bold text-faint-3'>VARIABLES</div>
              <div className='space-y-1 text-xs font-mono'>
                <div className={`flex justify-between px-2 py-1 ${cellStates[0].executed ? '' : 'opacity-50'}`}>
                  <span className='text-violet-1'>signal</span>
                  <span className={cellStates[0].executed ? 'text-orange-6' : 'text-faint-4'}>Vec</span>
                </div>
                <div className={`flex justify-between px-2 py-1 ${cellStates[1].executed ? '' : 'opacity-50'}`}>
                  <span className='text-violet-1'>mean</span>
                  <span className={cellStates[1].executed ? 'text-orange-6' : 'text-faint-4'}>f64</span>
                </div>
              </div>
            </div>

            {/* Main editor */}
            <div className='col-span-12 rounded-lg bg-panel-3 p-4 md-nav:col-span-9'>
              <div className='space-y-4'>
                {/* Cell 1 */}
                <div className={`rounded border ${activeCell === 0 ? 'border-orange-6' : 'border-line-5'} bg-surface-6 p-3 transition-colors`}>
                  <div className='mb-2 text-xs text-faint-4'>[1]</div>
                  <div className='font-mono text-sm text-violet-1'>
                    <span className='text-blue-1'>let</span> signal ={' '}
                    <span className='text-orange-6'>load_signal</span>();
                  </div>
                  {cellStates[0].executed && (
                    <div className='mt-2 rounded bg-surface-7 p-2 text-xs text-mint-3 animate-fade-in'>
                      ↳ {cellStates[0].output}
                    </div>
                  )}
                  {cellStates[0].executing && (
                    <div className='mt-2 text-xs text-orange-3 animate-pulse'>
                      Executing...
                    </div>
                  )}
                </div>

                {/* Cell 2 */}
                <div className={`rounded border ${activeCell === 1 ? 'border-orange-6' : 'border-line-5'} bg-surface-6 p-3 transition-colors`}>
                  <div className='mb-2 text-xs text-faint-4'>[2]</div>
                  <div className='font-mono text-sm text-violet-1'>
                    <span className='text-blue-1'>let</span> mean = signal.
                    <span className='text-blue-1'>iter</span>().
                    <span className='text-blue-1'>sum</span>()
                  </div>
                  {cellStates[1].executed && (
                    <div className='mt-2 rounded bg-surface-7 p-2 text-xs text-mint-3 animate-fade-in'>
                      ↳ {cellStates[1].output}
                    </div>
                  )}
                  {cellStates[1].executing && (
                    <div className='mt-2 text-xs text-orange-3 animate-pulse'>
                      Executing...
                    </div>
                  )}
                </div>

                {/* Cell 3 */}
                <div className={`rounded border ${activeCell === 2 ? 'border-orange-6' : 'border-line-5'} bg-surface-6 p-3 transition-colors`}>
                  <div className='mb-2 text-xs text-faint-4'>[3]</div>
                  <div className='font-mono text-sm text-violet-1'>
                    <span className='text-blue-1'>let</span> plot = signal.
                    <span className='text-blue-1'>visualize</span>();
                  </div>
                  {cellStates[2].executed && (
                    <div className='mt-2 rounded bg-surface-7 p-2 text-xs text-mint-3 animate-fade-in'>
                      ↳ {cellStates[2].output}
                    </div>
                  )}
                  {cellStates[2].executing && (
                    <div className='mt-2 text-xs text-orange-3 animate-pulse'>
                      Executing...
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      </Reveal>
    </section>
  );
};

export default Demo;
