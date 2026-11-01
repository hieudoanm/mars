import type { FC, HTMLAttributes, ReactNode } from 'react';
import { cn } from './cn';

export type SectionHeadingLevel = 'h2' | 'h3' | 'div';

export type SectionHeadingProps = Omit<HTMLAttributes<HTMLHeadingElement>, 'children'> & {
  children: ReactNode;
  as?: SectionHeadingLevel;
};

const SectionHeading: FC<SectionHeadingProps> = ({
  children,
  as: Component = 'h2',
  className,
  ...props
}) => (
  <Component
    data-component='SectionHeading'
    className={cn('mx-2 mt-1 mb-3 text-xs tracking-widest text-neutral-400 uppercase', className)}
    {...props}>
    {children}
  </Component>
);

SectionHeading.displayName = 'SectionHeading';

export default SectionHeading;
