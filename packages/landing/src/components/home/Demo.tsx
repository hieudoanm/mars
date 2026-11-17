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
        await new Promise((resolve) => setTimeout(resolve, 1000));

        // Cell 1
        setActiveCell(0);
        setCellStates((prev) => {
          const newStates = [...prev];
          newStates[0] = { executing: true, executed: false, output: '' };
          return newStates;
        });
        await new Promise((resolve) => setTimeout(resolve, 1500));
        setCellStates((prev) => {
          const newStates = [...prev];
          newStates[0] = {
            executing: false,
            executed: true,
            output: 'Signal loaded: 1024 samples',
          };
          return newStates;
        });
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Cell 2
        setActiveCell(1);
        setCellStates((prev) => {
          const newStates = [...prev];
          newStates[1] = { executing: true, executed: false, output: '' };
          return newStates;
        });
        await new Promise((resolve) => setTimeout(resolve, 1500));
        setCellStates((prev) => {
          const newStates = [...prev];
          newStates[1] = { executing: false, executed: true, output: 'Mean calculated: 42.3' };
          return newStates;
        });
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Cell 3
        setActiveCell(2);
        setCellStates((prev) => {
          const newStates = [...prev];
          newStates[2] = { executing: true, executed: false, output: '' };
          return newStates;
        });
        await new Promise((resolve) => setTimeout(resolve, 1500));
        setCellStates((prev) => {
          const newStates = [...prev];
          newStates[2] = { executing: false, executed: true, output: 'Chart rendered in 12ms' };
          return newStates;
        });
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }
    };

    cycle();
  }, []);

  return (
    <section className='max-w-shell max-sm-nav:px-4 max-sm-nav:py-22 mx-auto w-full px-6 py-30'>
      <Reveal className='text-center'>
        <div className='text-xs-plus tracking-caps-xl text-orange-2 flex items-center justify-center gap-2.5 font-bold uppercase'>
          <span className='bg-orange shadow-glow-orange h-2 w-2 rounded-full' />
          See it in action
        </div>
        <h2 className='text-display font-semibold-plus tracking-tighter-plus max-sm-nav:text-display-sm mt-5 mb-4.5 leading-tight'>
          A notebook environment that
          <br />
          feels <em className='text-orange-2 font-serif font-normal'>instant.</em>
        </h2>
        <p className='max-w-prose-lg text-muted-2 mx-auto text-base'>
          Execute cells, inspect variables, and iterate on ideas without leaving your flow.
        </p>
      </Reveal>

      <Reveal delay={0.15} className='mt-14'>
        <div className='border-line-5 bg-surface-6 relative mx-auto max-w-5xl rounded-xl border p-4 shadow-2xl'>
          {/* Window controls */}
          <div className='mb-4 flex items-center gap-2'>
            <div className='h-3 w-3 rounded-full bg-red-500' />
            <div className='h-3 w-3 rounded-full bg-yellow-500' />
            <div className='h-3 w-3 rounded-full bg-green-500' />
            <div className='text-xs-plus text-faint-4 ml-4'>analysis.mars — Project Mars</div>
          </div>

          {/* Demo content */}
          <div className='grid grid-cols-12 gap-4'>
            {/* Sidebar */}
            <div className='bg-panel-3 md-nav:block col-span-3 hidden rounded-lg p-3'>
              <div className='text-faint-3 mb-3 text-xs font-bold'>FILES</div>
              <div className='space-y-1'>
                <div className='bg-orange/20 text-orange-3 rounded px-2 py-1.5 text-xs'>
                  analysis.mars
                </div>
                <div className='text-faint-4 rounded px-2 py-1.5 text-xs'>data.csv</div>
                <div className='text-faint-4 rounded px-2 py-1.5 text-xs'>notes.md</div>
              </div>
              <div className='text-faint-3 mt-4 mb-3 text-xs font-bold'>VARIABLES</div>
              <div className='space-y-1 font-mono text-xs'>
                <div
                  className={`flex justify-between px-2 py-1 ${cellStates[0].executed ? '' : 'opacity-50'}`}>
                  <span className='text-violet-1'>signal</span>
                  <span className={cellStates[0].executed ? 'text-orange-6' : 'text-faint-4'}>
                    Vec
                  </span>
                </div>
                <div
                  className={`flex justify-between px-2 py-1 ${cellStates[1].executed ? '' : 'opacity-50'}`}>
                  <span className='text-violet-1'>mean</span>
                  <span className={cellStates[1].executed ? 'text-orange-6' : 'text-faint-4'}>
                    f64
                  </span>
                </div>
              </div>
            </div>

            {/* Main editor */}
            <div className='bg-panel-3 md-nav:col-span-9 col-span-12 rounded-lg p-4'>
              <div className='space-y-4'>
                {/* Cell 1 */}
                <div
                  className={`rounded border ${activeCell === 0 ? 'border-orange-6' : 'border-line-5'} bg-surface-6 p-3 transition-colors`}>
                  <div className='text-faint-4 mb-2 text-xs'>[1]</div>
                  <div className='text-violet-1 font-mono text-sm'>
                    <span className='text-blue-1'>let</span> signal ={' '}
                    <span className='text-orange-6'>load_signal</span>();
                  </div>
                  {cellStates[0].executed && (
                    <div className='bg-surface-7 text-mint-3 animate-fade-in mt-2 rounded p-2 text-xs'>
                      ↳ {cellStates[0].output}
                    </div>
                  )}
                  {cellStates[0].executing && (
                    <div className='text-orange-3 mt-2 animate-pulse text-xs'>Executing...</div>
                  )}
                </div>

                {/* Cell 2 */}
                <div
                  className={`rounded border ${activeCell === 1 ? 'border-orange-6' : 'border-line-5'} bg-surface-6 p-3 transition-colors`}>
                  <div className='text-faint-4 mb-2 text-xs'>[2]</div>
                  <div className='text-violet-1 font-mono text-sm'>
                    <span className='text-blue-1'>let</span> mean = signal.
                    <span className='text-blue-1'>iter</span>().
                    <span className='text-blue-1'>sum</span>()
                  </div>
                  {cellStates[1].executed && (
                    <div className='bg-surface-7 text-mint-3 animate-fade-in mt-2 rounded p-2 text-xs'>
                      ↳ {cellStates[1].output}
                    </div>
                  )}
                  {cellStates[1].executing && (
                    <div className='text-orange-3 mt-2 animate-pulse text-xs'>Executing...</div>
                  )}
                </div>

                {/* Cell 3 */}
                <div
                  className={`rounded border ${activeCell === 2 ? 'border-orange-6' : 'border-line-5'} bg-surface-6 p-3 transition-colors`}>
                  <div className='text-faint-4 mb-2 text-xs'>[3]</div>
                  <div className='text-violet-1 font-mono text-sm'>
                    <span className='text-blue-1'>let</span> plot = signal.
                    <span className='text-blue-1'>visualize</span>();
                  </div>
                  {cellStates[2].executed && (
                    <div className='bg-surface-7 text-mint-3 animate-fade-in mt-2 rounded p-2 text-xs'>
                      ↳ {cellStates[2].output}
                    </div>
                  )}
                  {cellStates[2].executing && (
                    <div className='text-orange-3 mt-2 animate-pulse text-xs'>Executing...</div>
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
