import { type FC } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeSanitize from 'rehype-sanitize';

type MarkdownPreviewProps = {
  source: string;
};

const MarkdownPreview: FC<MarkdownPreviewProps> = ({ source }) => (
  <div
    data-component='MarkdownPreview'
    className='prose prose-invert prose-a:text-orange-400 prose-code:text-amber-300 prose-headings:text-neutral-100 mt-3 max-w-none rounded-lg border border-neutral-800 bg-neutral-950/60 p-4 text-sm'>
    <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRaw, rehypeSanitize]}>
      {source}
    </ReactMarkdown>
  </div>
);

MarkdownPreview.displayName = 'MarkdownPreview';
export default MarkdownPreview;
