import * as vscode from 'vscode';
import { CHART_MIME_TYPES, ERROR_MIME } from './constants';
import type { MarsOutput } from './types';

export const notebookOutputToMarsOutput = (output: vscode.NotebookCellOutput): MarsOutput => {
  const items = output.items;
  if (items.length === 0) return { type: 'text', text: '' };
  if (items.length === 1) return itemToMarsOutput(items[0]);

  const data: Record<string, string> = {};
  for (const item of items) {
    const mimeType = item.mime ?? 'text/plain';
    if (mimeType === ERROR_MIME) continue;
    data[mimeType] = itemToMarsValue(item);
  }
  return { type: 'mime_bundle', data, metadata: {} };
};

const itemToMarsValue = (item: vscode.NotebookCellOutputItem): string => {
  const mimeType = item.mime ?? 'text/plain';
  const bytes = Buffer.from(item.data);
  if (mimeType.startsWith('image/') && mimeType !== 'image/svg+xml') {
    return bytes.toString('base64');
  }
  return bytes.toString('utf-8');
};

const itemToMarsOutput = (item: vscode.NotebookCellOutputItem): MarsOutput => {
  const mimeType = item.mime ?? 'text/plain';
  const bytes = Buffer.from(item.data);
  const text = bytes.toString('utf-8');
  if (mimeType === ERROR_MIME) return parseErrorOutput(text);
  if (mimeType.startsWith('image/')) {
    return { type: 'image', mime_type: mimeType, data: itemToMarsValue(item) };
  }
  if (mimeType === 'text/html') return { type: 'html', html: text };
  if (mimeType === 'text/latex') return { type: 'latex', latex: text };
  if (CHART_MIME_TYPES.includes(mimeType)) return { type: 'chart', spec: parseSpec(text) };
  if (mimeType === 'text/plain' || mimeType === 'text/markdown') return { type: 'text', text };
  return { type: 'mime_bundle', data: { [mimeType]: text }, metadata: {} };
};

const parseErrorOutput = (text: string): MarsOutput => {
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed === 'object' && parsed !== null) {
      const record = parsed as Record<string, unknown>;
      const name = typeof record.name === 'string' ? record.name : 'Error';
      const message = typeof record.message === 'string' ? record.message : text;
      const stack = typeof record.stack === 'string' ? record.stack : '';
      return { type: 'error', name, message, traceback: stack ? stack.split('\n') : [] };
    }
  } catch {
    // Malformed error payloads are retained as plain error text.
  }
  return { type: 'error', name: 'Error', message: text, traceback: [] };
};

const parseSpec = (text: string): Record<string, unknown> => {
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed === 'object' && parsed !== null) return parsed as Record<string, unknown>;
  } catch {
    // Invalid chart payloads become an empty chart specification.
  }
  return {};
};
