'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const heading =
  'mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm';

const tools = [
  [
    'M',
    '',
    'CANONICAL',
    'Desktop app',
    'A polished notebook workspace with CodeMirror, rich output, autosave, and native dialogs.',
    'Explore desktop',
  ],
  [
    '>_',
    'bg-tool-1 font-mono text-base text-tool-3',
    'HEADLESS',
    'Command line',
    'Create, run, inspect, import, export, and automate notebooks anywhere Cargo runs.',
    'Read the CLI guide',
  ],
  [
    '◇',
    'bg-tool-2 text-2xl text-tool-4',
    'COMPANION',
    'Code Editor',
    'Extensions for VS Code, Zed, and JetBrains with syntax support, execution, and rich output.',
    'Download extensions',
  ],
];

const ToolCard: FC<{
  mark: string;
  markClass: string;
  tag: string;
  title: string;
  body: string;
  link: string;
  index: number;
}> = ({ mark, markClass, tag, title, body, link, index }) => {
  return (
    <Reveal delay={0.06 + index * 0.08}>
      <article
        className='border-line bg-panel-3 hover:border-orange/30 h-full rounded-xl border p-6 transition-all duration-300 hover:-translate-y-1'
        key={title}>
        <div className='mb-10 flex items-center justify-between'>
          <span
            className={`bg-tool-5 font-heavy text-ink-5 grid h-9 w-9 place-items-center rounded-lg ${markClass}`}>
            {mark}
          </span>
          <span className='text-2xs tracking-caps-md text-faint-3 font-bold'>{tag}</span>
        </div>
        <h3 className='m-0 text-xl'>{title}</h3>
        <p className='text-sm-plus min-h-19'>{body}</p>
        <a className='text-orange-2 font-bold-plus text-xs' href='/downloads'>
          {link} <span className='ml-2'>→</span>
        </a>
      </article>
    </Reveal>
  );
};

const Tools: FC = () => {
  return (
    <section className={`${sectionShell} max-sm-nav:py-22 py-30 pb-38`}>
      <Reveal className='max-w-prose-sm mx-auto text-center'>
        <div className={`${eyebrow} justify-center`}>Meet Project Mars where you work</div>
        <h2 className={heading}>
          One core.{' '}
          <em className='text-orange-2 font-serif font-normal'>Your choice of surface.</em>
        </h2>
      </Reveal>
      <div className='max-md-nav:grid-cols-1 mt-14 grid grid-cols-3 gap-4'>
        {tools.map(([mark, markClass, tag, title, body, link], index) => (
          <ToolCard
            key={title}
            mark={mark}
            markClass={markClass}
            tag={tag}
            title={title}
            body={body}
            link={link}
            index={index}
          />
        ))}
      </div>
    </section>
  );
};

export default Tools;
