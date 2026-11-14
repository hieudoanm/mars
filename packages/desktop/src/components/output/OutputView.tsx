import { type FC, type ReactNode, useEffect, useRef, useState } from 'react';
import katex from 'katex';
import vegaEmbed from 'vega-embed';
import type { Output } from '../../types/types';
import { imageDataUri, selectMime, type MimeBundleData } from '../../lib/mimeBundle';
import { sanitizeHtml } from '../../lib/sanitizeHtml';
import { evalNotebookScript } from '../../lib/kernelService';
import { cn, CodeBlock } from '../shared';
import 'katex/dist/katex.min.css';

import { OUTPUT_SURFACE } from './outputStyles';
import TableOutput from './TableOutput';
import WidgetOutput from './WidgetOutput';
import MarkdownPreview from './MarkdownPreview';

const SOURCE_ANNOTATION = /\[mars:cell=(\d+),line=(\d+)\]/g;

const ErrorMessage: FC<{
  message: string;
  onNavigateSource?: (cellHash: number, line: number) => void;
}> = ({ message, onNavigateSource }) => {
  const parts: ReactNode[] = [];
  let cursor = 0;
  for (const match of message.matchAll(SOURCE_ANNOTATION)) {
    const index = match.index ?? 0;
    if (index > cursor) parts.push(message.slice(cursor, index));
    const cellHash = Number(match[1]);
    const line = Number(match[2]);
    parts.push(
      onNavigateSource ? (
        <button
          key={index}
          type='button'
          className='cursor-pointer text-sky-400 underline decoration-dotted underline-offset-2 hover:text-sky-300'
          onClick={() => onNavigateSource(cellHash, line)}>
          {match[0]}
        </button>
      ) : (
        match[0]
      ),
    );
    cursor = index + match[0].length;
  }
  if (cursor < message.length) parts.push(message.slice(cursor));
  return <>{parts}</>;
};

const OutputView: FC<{
  output: Output;
  onNavigateSource?: (cellHash: number, line: number) => void;
}> = ({ output, onNavigateSource }) => {
  if (output.type === 'text') return <CodeBlock>{output.text}</CodeBlock>;
  if (output.type === 'error') {
    return (
      <CodeBlock tone='danger'>
        <strong className='text-red-400'>{output.name}</strong>
        {'\n'}
        <ErrorMessage message={output.message} onNavigateSource={onNavigateSource} />
      </CodeBlock>
    );
  }
  if (output.type === 'html') {
    return (
      <div
        data-component='OutputView'
        className={OUTPUT_SURFACE}
        dangerouslySetInnerHTML={{ __html: sanitizeHtml(output.html) }}
      />
    );
  }
  if (output.type === 'latex') {
    const html = katex.renderToString(output.latex, { displayMode: true, throwOnError: false });
    return (
      <div
        data-component='OutputView'
        className={cn(OUTPUT_SURFACE, 'font-serif text-lg text-amber-400')}
        data-latex={output.latex}
        dangerouslySetInnerHTML={{ __html: html }}
      />
    );
  }
  if (output.type === 'image') {
    return <ImageOutput mimeType={output.mime_type} value={output.data} />;
  }
  if (output.type === 'chart') return <ChartOutput spec={output.spec} />;
  if (output.type === 'mime_bundle') return <MimeBundleOutput data={output.data} />;
  if (output.type === 'table') {
    return <TableOutput columns={output.columns} rows={output.rows} />;
  }
  return <CodeBlock>{JSON.stringify(output, null, 2)}</CodeBlock>;
};
OutputView.displayName = 'OutputView';
export default OutputView;

const MimeBundleOutput: FC<{ data: MimeBundleData }> = ({ data }) => {
  const selected = selectMime(data);
  if (!selected) return <CodeBlock>{JSON.stringify(data, null, 2)}</CodeBlock>;

  switch (selected.kind) {
    case 'image':
      return <ImageOutput mimeType={selected.mimeType} value={selected.value} />;
    case 'html':
      return (
        <div
          data-component='MimeBundleOutput'
          className={OUTPUT_SURFACE}
          dangerouslySetInnerHTML={{ __html: sanitizeHtml(selected.value) }}
        />
      );
    case 'latex': {
      const html = katex.renderToString(selected.value, { displayMode: true, throwOnError: false });
      return (
        <div
          data-component='MimeBundleOutput'
          className={cn(OUTPUT_SURFACE, 'font-serif text-lg text-amber-400')}
          data-latex={selected.value}
          dangerouslySetInnerHTML={{ __html: html }}
        />
      );
    }
    case 'chart':
      return <ChartSpec value={selected.value} />;
    case 'javascript':
      return <JavascriptOutput source={selected.value} />;
    case 'widget':
      return <WidgetOutput modelId={selected.value} />;
    case 'markdown':
      return <MarkdownPreview source={selected.value} />;
    case 'text':
      return <CodeBlock>{selected.value}</CodeBlock>;
  }
};
MimeBundleOutput.displayName = 'MimeBundleOutput';

const ImageOutput: FC<{ mimeType: string; value: string }> = ({ mimeType, value }) => {
  const [expanded, setExpanded] = useState(false);
  const [zoomed, setZoomed] = useState(false);
  const src = imageDataUri(mimeType, value);
  return (
    <>
      <div
        data-component='ImageOutput'
        className='mt-3 flex justify-center rounded-lg bg-white p-3'>
        <button
          type='button'
          title='View full size'
          aria-label='View image full size'
          className='cursor-zoom-in'
          onClick={() => setExpanded(true)}>
          <img className='block h-auto max-w-full' src={src} alt='Notebook output' />
        </button>
      </div>
      {expanded ? (
        <div
          data-component='ImageLightbox'
          role='dialog'
          aria-label='Image output'
          className={`fixed inset-0 z-50 flex bg-black/80 p-8 ${zoomed ? 'items-start justify-start overflow-auto' : 'items-center justify-center'}`}
          onClick={() => {
            setExpanded(false);
            setZoomed(false);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Escape') {
              setExpanded(false);
              setZoomed(false);
            }
          }}>
          <img
            className={
              zoomed
                ? 'block max-w-none cursor-zoom-out'
                : 'block max-h-full max-w-full cursor-zoom-in object-contain'
            }
            src={src}
            alt='Notebook output'
            onClick={(event) => {
              event.stopPropagation();
              setZoomed((current) => !current);
            }}
          />
        </div>
      ) : null}
    </>
  );
};
ImageOutput.displayName = 'ImageOutput';

const ChartSpec: FC<{ value: string }> = ({ value }) => {
  try {
    const spec: unknown = JSON.parse(value);
    if (typeof spec === 'object' && spec !== null && !Array.isArray(spec)) {
      return <ChartOutput spec={spec as Record<string, unknown>} />;
    }
  } catch {
    // Fall through to the raw chart specification below.
  }
  return <CodeBlock tone='muted'>{value}</CodeBlock>;
};
ChartSpec.displayName = 'ChartSpec';

const ChartOutput: FC<{ spec: Record<string, unknown> }> = ({ spec }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let finalized = false;
    let view: { finalize: () => void } | undefined;
    setError(null);
    if (containerRef.current) {
      void vegaEmbed(containerRef.current, spec, { actions: false, renderer: 'canvas' })
        .then((result) => {
          if (finalized) result.view.finalize();
          else view = result.view;
        })
        .catch((reason: unknown) => {
          if (!finalized)
            setError(reason instanceof Error ? reason.message : 'Unable to render chart');
        });
    }
    return () => {
      finalized = true;
      view?.finalize();
    };
  }, [spec]);
  return (
    <div className='mt-3'>
      {error ? <CodeBlock tone='danger'>Chart rendering failed: {error}</CodeBlock> : null}
      <div
        data-component='ChartOutput'
        className='overflow-auto rounded-lg border border-neutral-700 bg-neutral-950 p-3'
        ref={containerRef}
      />
    </div>
  );
};
ChartOutput.displayName = 'ChartOutput';

const JavascriptOutput: FC<{ source: string }> = ({ source }) => {
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    setError(null);
    evalNotebookScript(source).catch((reason: unknown) => {
      if (!cancelled)
        setError(reason instanceof Error ? reason.message : 'JavaScript execution failed');
    });
    return () => {
      cancelled = true;
    };
  }, [source]);
  if (error) return <CodeBlock tone='danger'>JavaScript failed: {error}</CodeBlock>;
  return null;
};
JavascriptOutput.displayName = 'JavascriptOutput';
