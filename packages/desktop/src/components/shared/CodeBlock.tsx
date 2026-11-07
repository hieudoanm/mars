import type { FC, HTMLAttributes, ReactNode } from 'react';
import { cn } from './cn';

export type CodeBlockTone = 'default' | 'muted' | 'danger';

export type CodeBlockProps = Omit<HTMLAttributes<HTMLPreElement>, 'children'> & {
  children: ReactNode;
  tone?: CodeBlockTone;
};

const toneClasses: Record<CodeBlockTone, string> = {
  default: 'text-neutral-300',
  muted: 'text-neutral-400',
  danger: 'border-l-4 border-red-500 text-red-200',
};

const CodeBlock: FC<CodeBlockProps> = ({ children, tone = 'default', className, ...props }) => (
  <pre
    data-component='CodeBlock'
    className={cn(
      'mt-3 overflow-auto rounded-lg bg-neutral-950 p-3 leading-relaxed whitespace-pre-wrap',
      toneClasses[tone],
      className,
    )}
    {...props}>
    {children}
  </pre>
);

CodeBlock.displayName = 'CodeBlock';

export default CodeBlock;
