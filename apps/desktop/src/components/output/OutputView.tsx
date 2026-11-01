import { type FC, useEffect, useRef, useState } from 'react';
import katex from 'katex';
import vegaEmbed from 'vega-embed';
import type { Output } from '../../types/types';
import { imageDataUri, selectMime, type MimeBundleData } from '../../lib/mimeBundle';
import { sanitizeHtml } from '../../lib/sanitizeHtml';
import { cn, CodeBlock } from '../shared';
import 'katex/dist/katex.min.css';

const OUTPUT_SURFACE =
  'mt-3 overflow-auto rounded-lg bg-neutral-950 p-3 leading-relaxed whitespace-pre-wrap';

const OutputView: FC<{ output: Output }> = ({ output }) => {
  if (output.type === 'text') return <CodeBlock>{output.text}</CodeBlock>;
  if (output.type === 'error') {
    return (
      <CodeBlock tone='danger'>
        <strong className='text-red-400'>{output.name}</strong>
        {'\n'}
        {output.message}
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
    return (
      <div data-component='OutputView' className={OUTPUT_SURFACE}>
        <table className='w-full border-collapse text-sm'>
          <thead>
            <tr>
              {output.columns.map((column, index) => (
                <th
                  key={index}
                  className='border border-neutral-700 bg-neutral-800 p-2 text-left text-neutral-100'>
                  {column}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {output.rows.map((row, rowIndex) => (
              <tr key={rowIndex}>
                {row.map((value, columnIndex) => (
                  <td
                    key={columnIndex}
                    className='border border-neutral-700 bg-neutral-900 p-2 text-left text-neutral-300'>
                    {value}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
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
    case 'text':
      return <CodeBlock>{selected.value}</CodeBlock>;
  }
};
MimeBundleOutput.displayName = 'MimeBundleOutput';

const ImageOutput: FC<{ mimeType: string; value: string }> = ({ mimeType, value }) => {
  return (
    <div data-component='ImageOutput' className='mt-3 flex justify-center rounded-lg bg-white p-3'>
      <img
        className='block h-auto max-w-full'
        src={imageDataUri(mimeType, value)}
        alt='Notebook output'
      />
    </div>
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
