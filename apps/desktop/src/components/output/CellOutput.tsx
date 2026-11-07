import { type FC, useState } from 'react';
import {
  LuChevronDown,
  LuChevronRight,
  LuClipboard,
  LuDownload,
  LuTrash2,
} from 'react-icons/lu';
import type { Output } from '../../types/types';
import OutputView from './OutputView';
import { IconButton } from '../shared';

type CellOutputProps = {
  output: Output;
  onRemove?: () => void;
  onNavigateSource?: (cellHash: number, line: number) => void;
};

const OUTPUT_LABELS: Record<Output['type'], string> = {
  text: 'stdout',
  error: 'error',
  html: 'html',
  image: 'image',
  table: 'table',
  latex: 'latex',
  chart: 'chart',
  mime_bundle: 'output',
};

const outputText = (output: Output): string => {
  switch (output.type) {
    case 'text':
      return output.text;
    case 'error':
      return `${output.name}: ${output.message}\n${output.traceback.join('\n')}`;
    case 'html':
      return output.html;
    case 'image':
      return output.mime_type;
    case 'table':
      return [output.columns.join('\t'), ...output.rows.map((row) => row.join('\t'))].join('\n');
    case 'latex':
      return output.latex;
    case 'chart':
      return JSON.stringify(output.spec);
    case 'mime_bundle':
      return typeof output.data['text/plain'] === 'string'
        ? output.data['text/plain']
        : JSON.stringify(output.data);
  }
};

const EXTENSIONS: Record<string, string> = {
  'image/png': 'png',
  'image/jpeg': 'jpg',
  'image/svg+xml': 'svg',
  'image/gif': 'gif',
  'image/webp': 'webp',
  'text/html': 'html',
  'text/plain': 'txt',
  'application/json': 'json',
};

const TEXT_EXTENSIONS: Record<Output['type'], string> = {
  text: 'txt',
  error: 'txt',
  html: 'html',
  image: 'png',
  table: 'tsv',
  latex: 'tex',
  chart: 'json',
  mime_bundle: 'txt',
};

const downloadBlob = (blob: Blob, ext: string) => {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `mars-output.${ext}`;
  anchor.click();
  URL.revokeObjectURL(url);
};

const base64ToBlob = (data: string, mime: string): Blob | null => {
  try {
    const binary = atob(data);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
      bytes[index] = binary.charCodeAt(index);
    }
    return new Blob([bytes], { type: mime });
  } catch {
    return null;
  }
};

const downloadOutput = (output: Output) => {
  if (output.type === 'image') {
    const blob = base64ToBlob(output.data, output.mime_type);
    downloadBlob(blob ?? new Blob([outputText(output)]), EXTENSIONS[output.mime_type] ?? 'bin');
    return;
  }
  if (output.type === 'mime_bundle') {
    const entries = Object.entries(output.data);
    const imageEntry = entries.find(
      ([mime, value]) =>
        EXTENSIONS[mime]?.match(/^(png|jpg|svg|gif|webp)$/) && typeof value === 'string',
    );
    if (imageEntry) {
      const blob = base64ToBlob(imageEntry[1] as string, imageEntry[0]);
      if (blob) {
        downloadBlob(blob, EXTENSIONS[imageEntry[0]] ?? 'bin');
        return;
      }
    }
    const [mime, value] = entries.find(([key]) => key === 'text/html') ?? entries[0] ?? [];
    const text = typeof value === 'string' ? value : outputText(output);
    downloadBlob(new Blob([text], { type: 'text/plain' }), EXTENSIONS[mime ?? ''] ?? 'txt');
    return;
  }
  downloadBlob(new Blob([outputText(output)], { type: 'text/plain' }), TEXT_EXTENSIONS[output.type]);
};

const CellOutput: FC<CellOutputProps> = ({ output, onRemove, onNavigateSource }) => {
  const [collapsed, setCollapsed] = useState(false);
  const label = OUTPUT_LABELS[output.type];

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(outputText(output));
    } catch {
      // clipboard unavailable (e.g. tests / insecure context)
    }
  };

  return (
    <div className='group/output relative mb-2 rounded-md border border-neutral-800 bg-neutral-950/60'>
      <div className='flex items-center gap-1 border-b border-neutral-800/70 px-2 py-1'>
        <button
          type='button'
          className='flex items-center gap-1 text-xs text-neutral-500 hover:text-neutral-300'
          aria-label={collapsed ? `Expand ${label} output` : `Collapse ${label} output`}
          aria-expanded={!collapsed}
          onClick={() => setCollapsed((current) => !current)}>
          {collapsed ? (
            <LuChevronRight aria-hidden='true' className='h-3 w-3' />
          ) : (
            <LuChevronDown aria-hidden='true' className='h-3 w-3' />
          )}
          <span>{label}</span>
        </button>
        <span className='ml-auto flex items-center gap-0.5 opacity-0 transition-opacity group-hover/output:opacity-100'>
          <IconButton
            label='Copy output'
            icon={<LuClipboard aria-hidden='true' className='h-3 w-3' />}
            onClick={() => void copy()}
          />
          <IconButton
            label='Download output'
            icon={<LuDownload aria-hidden='true' className='h-3 w-3' />}
            onClick={() => downloadOutput(output)}
          />
          {onRemove ? (
            <IconButton
              label='Clear output'
              variant='danger'
              icon={<LuTrash2 aria-hidden='true' className='h-3 w-3' />}
              onClick={onRemove}
            />
          ) : null}
        </span>
      </div>
      <div className={collapsed ? 'hidden' : 'p-2'}>
        <OutputView output={output} onNavigateSource={onNavigateSource} />
      </div>
    </div>
  );
};

CellOutput.displayName = 'CellOutput';
export default CellOutput;
