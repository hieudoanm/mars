import type { FC } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

const Markdown: FC<{ content: string }> = ({ content }) => {
  return (
    <div data-testid='markdown' className='markdown'>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
};

export default Markdown;
