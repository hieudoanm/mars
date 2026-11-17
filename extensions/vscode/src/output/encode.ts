import * as vscode from 'vscode';
import type { MarsOutput } from './types';
import { ERROR_MIME } from './constants';

const MIME_PRIORITY = [
  'image/svg+xml',
  'image/png',
  'image/jpeg',
  'image/gif',
  'image/webp',
  'text/html',
  'text/latex',
  'text/markdown',
  'text/plain',
];

const textItem = (value: string, mime: string): vscode.NotebookCellOutputItem => {
  return new vscode.NotebookCellOutputItem(Buffer.from(value, 'utf-8'), mime);
};

const imageItem = (mimeType: string, value: string): vscode.NotebookCellOutputItem => {
  const data = value.trim();
  if (data.startsWith('data:')) {
    const uri = parseDataUri(data);
    if (uri) {
      if (mimeType === 'image/svg+xml') {
        return textItem(
          uri.base64
            ? Buffer.from(uri.payload, 'base64').toString('utf-8')
            : decodeDataUriText(uri.payload),
          mimeType,
        );
      }
      return new vscode.NotebookCellOutputItem(
        Buffer.from(uri.payload, uri.base64 ? 'base64' : 'utf-8'),
        mimeType,
      );
    }
  }
  if (mimeType === 'image/svg+xml') {
    return textItem(
      data.startsWith('<') ? data : Buffer.from(data, 'base64').toString('utf-8'),
      mimeType,
    );
  }
  return new vscode.NotebookCellOutputItem(Buffer.from(data, 'base64'), mimeType);
};

const parseDataUri = (value: string): { base64: boolean; payload: string } | null => {
  const comma = value.indexOf(',');
  if (comma === -1) return null;
  const header = value.slice(5, comma);
  return { base64: header.endsWith(';base64'), payload: value.slice(comma + 1) };
};

const decodeDataUriText = (payload: string): string => {
  try {
    return decodeURIComponent(payload);
  } catch {
    return payload;
  }
};

const mimeValueToString = (value: unknown): string | null => {
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) {
    const parts: string[] = [];
    for (const part of value) {
      if (typeof part !== 'string') return null;
      parts.push(part);
    }
    return parts.join('');
  }
  return null;
};

const mimeItem = (mimeType: string, value: unknown): vscode.NotebookCellOutputItem | null => {
  if (mimeType.startsWith('image/')) {
    const text = mimeValueToString(value);
    return text === null ? null : imageItem(mimeType, text);
  }
  const text = mimeValueToString(value);
  if (text !== null) return textItem(text, mimeType);
  return textItem(JSON.stringify(value), mimeType);
};

const tableToHtml = (columns: string[], rows: string[][]): string => {
  const headers = columns.map((column) => `<th>${escapeHtml(column)}</th>`).join('');
  const body = rows
    .map((row) => `<tr>${row.map((value) => `<td>${escapeHtml(value)}</td>`).join('')}</tr>`)
    .join('');
  return `<table><thead><tr>${headers}</tr></thead><tbody>${body}</tbody></table>`;
};

const escapeHtml = (value: string): string => {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
};

export const marsOutputToNotebookOutput = (output: MarsOutput): vscode.NotebookCellOutput => {
  const items: vscode.NotebookCellOutputItem[] = [];
  switch (output.type) {
    case 'text':
      items.push(textItem(output.text, 'text/plain'));
      break;
    case 'error':
      items.push(
        textItem(
          JSON.stringify({
            name: output.name,
            message: output.message,
            stack: output.traceback.join('\n'),
          }),
          ERROR_MIME,
        ),
      );
      break;
    case 'html':
      items.push(textItem(output.html, 'text/html'));
      break;
    case 'latex':
      items.push(textItem(output.latex, 'text/latex'));
      break;
    case 'image':
      items.push(imageItem(output.mime_type, output.data));
      break;
    case 'table':
      items.push(textItem(tableToHtml(output.columns, output.rows), 'text/html'));
      break;
    case 'chart':
      items.push(textItem(JSON.stringify(output.spec), 'application/vnd.vega-lite+json'));
      break;
    case 'mime_bundle': {
      const entries = Object.entries(output.data);
      entries.sort((left, right) => mimeRank(left[0]) - mimeRank(right[0]));
      for (const [mimeType, value] of entries) {
        const item = mimeItem(mimeType, value);
        if (item) items.push(item);
      }
      if (items.length === 0) items.push(textItem(JSON.stringify(output.data), 'text/plain'));
      break;
    }
  }
  return new vscode.NotebookCellOutput(items);
};

const mimeRank = (mimeType: string): number => {
  const index = MIME_PRIORITY.indexOf(mimeType);
  return index === -1 ? MIME_PRIORITY.length : index;
};
