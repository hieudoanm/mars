import { useEffect, useRef } from 'react';
import katex from 'katex';
import vegaEmbed from 'vega-embed';
import type { Output } from '../lib/types';
import { imageDataUri, selectMime, type MimeBundleData } from '../lib/mimeBundle';
import { sanitizeHtml } from '../lib/sanitizeHtml';

export default function OutputView({ output }: { output: Output }) {
  if (output.type === 'text') return <pre className='output text-output'>{output.text}</pre>;
  if (output.type === 'error') {
    return (
      <pre className='output error-output'>
        <strong>{output.name}</strong>
        {'\n'}
        {output.message}
      </pre>
    );
  }
  if (output.type === 'html') {
    return (
      <div
        className='output html-output'
        dangerouslySetInnerHTML={{ __html: sanitizeHtml(output.html) }}
      />
    );
  }
  if (output.type === 'latex') {
    const html = katex.renderToString(output.latex, { displayMode: true, throwOnError: false });
    return (
      <div
        className='output math-output'
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
      <div className='output table-output'>
        <table>
          <thead>
            <tr>
              {output.columns.map((column, index) => (
                <th key={index}>{column}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {output.rows.map((row, rowIndex) => (
              <tr key={rowIndex}>
                {row.map((value, columnIndex) => (
                  <td key={columnIndex}>{value}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }
  return <pre className='output'>{JSON.stringify(output, null, 2)}</pre>;
}

function MimeBundleOutput({ data }: { data: MimeBundleData }) {
  const selected = selectMime(data);
  if (!selected) return <pre className='output'>{JSON.stringify(data, null, 2)}</pre>;

  switch (selected.kind) {
    case 'image':
      return <ImageOutput mimeType={selected.mimeType} value={selected.value} />;
    case 'html':
      return (
        <div
          className='output html-output'
          dangerouslySetInnerHTML={{ __html: sanitizeHtml(selected.value) }}
        />
      );
    case 'latex': {
      const html = katex.renderToString(selected.value, { displayMode: true, throwOnError: false });
      return (
        <div
          className='output math-output'
          data-latex={selected.value}
          dangerouslySetInnerHTML={{ __html: html }}
        />
      );
    }
    case 'chart':
      return <ChartSpec value={selected.value} />;
    case 'markdown':
    case 'text':
      return <pre className='output text-output'>{selected.value}</pre>;
  }
}

function ImageOutput({ mimeType, value }: { mimeType: string; value: string }) {
  return (
    <div className='output image-output'>
      <img src={imageDataUri(mimeType, value)} alt='Notebook output' />
    </div>
  );
}

function ChartSpec({ value }: { value: string }) {
  try {
    const spec: unknown = JSON.parse(value);
    if (typeof spec === 'object' && spec !== null && !Array.isArray(spec)) {
      return <ChartOutput spec={spec as Record<string, unknown>} />;
    }
  } catch {
    // Fall through to the raw chart specification below.
  }
  return <pre className='output text-output'>{value}</pre>;
}

function ChartOutput({ spec }: { spec: Record<string, unknown> }) {
  const containerRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    let finalized = false;
    let view: { finalize: () => void } | undefined;
    if (containerRef.current) {
      void vegaEmbed(containerRef.current, spec, { actions: false, renderer: 'canvas' })
        .then((result) => {
          if (finalized) result.view.finalize();
          else view = result.view;
        })
        .catch(() => undefined);
    }
    return () => {
      finalized = true;
      view?.finalize();
    };
  }, [spec]);
  return <div className='output chart-output' ref={containerRef} />;
}
