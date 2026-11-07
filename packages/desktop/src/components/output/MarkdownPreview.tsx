import { cloneElement, isValidElement, type FC, type HTMLAttributes, type ReactNode } from 'react';
import {
  LuCircleAlert,
  LuInfo,
  LuLightbulb,
  LuOctagonAlert,
  LuTriangleAlert,
} from 'react-icons/lu';
import ReactMarkdown, { defaultUrlTransform } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize';

type MarkdownPreviewProps = {
  source: string;
  attachments?: Record<string, string>;
};

const ATTACHMENT_REFERENCE = /\]\(attachment:([^)\s]+)\)/g;

const resolveAttachments = (source: string, attachments?: Record<string, string>) => {
  if (!attachments || Object.keys(attachments).length === 0) return source;
  return source.replace(ATTACHMENT_REFERENCE, (match, name: string) => {
    const dataUrl = attachments[name];
    return dataUrl ? `](${dataUrl})` : match;
  });
};

// Allow data: URIs for embedded attachment images while keeping the default
// protocol restrictions for everything else.
const sanitizeSchema = {
  ...defaultSchema,
  protocols: {
    ...defaultSchema.protocols,
    src: [...(defaultSchema.protocols?.src ?? []), 'data'],
  },
};

const textOf = (node: ReactNode): string => {
  if (typeof node === 'string' || typeof node === 'number') return String(node);
  if (Array.isArray(node)) return node.map(textOf).join('');
  if (isValidElement<{ children?: ReactNode }>(node)) return textOf(node.props.children);
  return '';
};

const slugify = (text: string): string =>
  text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, '')
    .trim()
    .replace(/\s+/g, '-');

const headingId =
  (level: number): FC<HTMLAttributes<HTMLHeadingElement> & { children?: ReactNode }> =>
  ({ children, ...props }) => {
    const Tag = `h${level}` as 'h1';
    const id = slugify(textOf(children));
    return (
      <Tag id={id || undefined} {...props}>
        {children}
      </Tag>
    );
  };

const ALERT_PATTERN = /^\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*/i;

const ALERT_STYLES: Record<string, { className: string; Icon: typeof LuInfo }> = {
  NOTE: { className: 'border-sky-500/60 bg-sky-950/30', Icon: LuInfo },
  TIP: { className: 'border-emerald-500/60 bg-emerald-950/30', Icon: LuLightbulb },
  IMPORTANT: {
    className: 'border-violet-500/60 bg-violet-950/30',
    Icon: LuCircleAlert,
  },
  WARNING: { className: 'border-amber-500/60 bg-amber-950/30', Icon: LuTriangleAlert },
  CAUTION: { className: 'border-red-500/60 bg-red-950/30', Icon: LuOctagonAlert },
};

// Remove a leading `[!TYPE]` marker from the first text node of a rendered
// blockquote, returning the alert type when present.
const stripAlertMarker = (node: ReactNode): { marker: string | null; node: ReactNode } => {
  if (typeof node === 'string') {
    const match = node.match(ALERT_PATTERN);
    if (!match) return { marker: null, node };
    return { marker: match[1].toUpperCase(), node: node.slice(match[0].length) };
  }
  if (Array.isArray(node)) {
    for (let index = 0; index < node.length; index += 1) {
      const result = stripAlertMarker(node[index]);
      if (result.marker) {
        const copy = [...node];
        copy[index] = result.node;
        return { marker: result.marker, node: copy };
      }
    }
    return { marker: null, node };
  }
  if (isValidElement(node)) {
    const props = node.props as { children?: ReactNode };
    const result = stripAlertMarker(props.children);
    if (!result.marker) return { marker: null, node };
    return { marker: result.marker, node: cloneElement(node, undefined, result.node) };
  }
  return { marker: null, node };
};

const AlertBlockquote: FC<HTMLAttributes<HTMLElement>> = ({ children, ...rest }) => {
  const { marker, node } = stripAlertMarker(children);
  if (!marker) {
    return (
      <blockquote {...rest} className='border-s-4 border-neutral-600 ps-4 text-neutral-300 italic'>
        {children}
      </blockquote>
    );
  }
  const style = ALERT_STYLES[marker] ?? ALERT_STYLES['NOTE'];
  return (
    <div className={`my-3 rounded-r-lg border-s-4 px-4 py-2 not-italic ${style.className}`}>
      <div className='mb-1 flex items-center gap-1.5 text-xs font-semibold tracking-wide uppercase'>
        <style.Icon className='h-3.5 w-3.5' />
        {marker}
      </div>
      <div className='text-neutral-200 [&_p]:my-1'>{node}</div>
    </div>
  );
};

const urlTransform = (url: string) =>
  url.startsWith('data:image/') ? url : defaultUrlTransform(url);

const MarkdownPreview: FC<MarkdownPreviewProps> = ({ source, attachments }) => (
  <div
    data-component='MarkdownPreview'
    className='prose prose-invert prose-a:text-orange-400 prose-code:text-amber-300 prose-headings:text-neutral-100 mt-3 max-w-none rounded-lg border border-neutral-800 bg-neutral-950/60 p-4 text-sm'>
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[[rehypeSanitize, sanitizeSchema], rehypeRaw]}
      urlTransform={urlTransform}
      components={{
        blockquote: AlertBlockquote,
        h1: headingId(1),
        h2: headingId(2),
        h3: headingId(3),
        h4: headingId(4),
        h5: headingId(5),
        h6: headingId(6),
      }}>
      {resolveAttachments(source, attachments)}
    </ReactMarkdown>
  </div>
);

MarkdownPreview.displayName = 'MarkdownPreview';
export default MarkdownPreview;
