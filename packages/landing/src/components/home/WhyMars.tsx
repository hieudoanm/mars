'use client';

import type { FC } from 'react';
import { Reveal } from '@/components/shared/Reveal';

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';
const eyebrow =
  'flex items-center gap-2.5 text-xs-plus font-bold uppercase tracking-caps-xl text-orange-2';
const heading =
  'mt-5 mb-4.5 text-display font-semibold-plus leading-tight tracking-tighter-plus max-sm-nav:text-display-sm';

const features = [
  {
    icon: '✦',
    iconClass: '',
    title: 'Explore without context switching',
    body: 'Mix Rust, Markdown, mathematics, tables, charts, and rich output in one document that stays readable as the idea grows.',
    footnote: 'Rich multi-media output',
  },
  {
    icon: '◈',
    iconClass: 'bg-[#36755d42] text-mint-6',
    title: 'Isolation by default',
    body: 'Notebook code runs outside the desktop process in a controlled runtime boundary. Reset, interrupt, and inspect with confidence.',
    footnote: 'Process-isolated execution',
  },
  {
    icon: '▣',
    iconClass: 'bg-[#8b543342] text-sand-1',
    title: 'Own your documents',
    body: 'The .mars format is open, JSON-based, versioned, and designed to remain useful beyond one application.',
    footnote: 'Human-readable by design',
  },
];

const FeatureCard: FC<{ feature: (typeof features)[number]; index: number }> = ({
  feature,
  index,
}) => {
  return (
    <Reveal delay={0.08 + index * 0.08}>
      <article
        className='border-line bg-feature-card relative min-h-80 overflow-hidden rounded-xl border p-7 transition-transform duration-300 hover:-translate-y-1'
        key={feature.title}>
        <div
          className={`bg-orange/30 mb-9 grid h-10 w-10 place-items-center rounded-xl text-xl text-white ${feature.iconClass}`}>
          {feature.icon}
        </div>
        <h3 className='tracking-tight-plus m-0 mb-3 text-xl'>{feature.title}</h3>
        <p className='text-sm-plus m-0'>{feature.body}</p>
        <div className='text-xs-plus text-muted-7 absolute bottom-6 flex items-center gap-2'>
          <span className='bg-mint h-1.5 w-1.5 rounded-full' /> {feature.footnote}
        </div>
      </article>
    </Reveal>
  );
};

const WhyMars: FC = () => {
  return (
    <section className={`${sectionShell} max-sm-nav:py-22 py-36`} id='why-mars'>
      <Reveal>
        <div className='max-w-prose-sm'>
          <div className={eyebrow}>A better place for unfinished thoughts</div>
          <h2 className={heading}>
            From first question
            <br />
            to working <em className='text-orange-2 font-serif font-normal'>insight.</em>
          </h2>
          <p className='max-w-prose-lg text-base'>
            Project Mars keeps the fast feedback of a notebook and brings the boundaries,
            portability, and clarity of a developer tool.
          </p>
        </div>
      </Reveal>
      <div className='md-nav:grid-cols-3 mt-14 grid grid-cols-1 gap-4'>
        {features.map((feature, index) => (
          <FeatureCard key={feature.title} feature={feature} index={index} />
        ))}
      </div>
    </section>
  );
};

export default WhyMars;
