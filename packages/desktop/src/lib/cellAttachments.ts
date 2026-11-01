import type { NotebookCell } from '../types/types';

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

/** Extract the `metadata.attachments` map (name → data URL) from a cell. */
export const getCellAttachments = (metadata: Record<string, unknown>): Record<string, string> => {
  const raw = metadata['attachments'];
  if (!isRecord(raw)) return {};
  const result: Record<string, string> = {};
  for (const [name, value] of Object.entries(raw)) {
    if (typeof value === 'string' && value.startsWith('data:')) {
      result[name] = value;
    }
  }
  return result;
};

const EXTENSIONS: Record<string, string> = {
  'image/png': 'png',
  'image/jpeg': 'jpg',
  'image/gif': 'gif',
  'image/webp': 'webp',
  'image/svg+xml': 'svg',
};

/**
 * Register an image data URL on a cell and return the generated attachment
 * name plus the Markdown reference to insert into the cell source.
 */
export const addCellAttachment = (
  cell: NotebookCell,
  dataUrl: string,
  mimeType: string,
  addAttachments: (patch: Record<string, string>) => void,
): string => {
  const existing = getCellAttachments(cell.metadata);
  const ext = EXTENSIONS[mimeType] ?? mimeType.split('/')[1] ?? 'png';
  let index = 1;
  let name = `image-${index}.${ext}`;
  while (name in existing) {
    index += 1;
    name = `image-${index}-${Date.now().toString(36)}.${ext}`;
  }
  addAttachments({ [name]: dataUrl });
  return `![${name}](attachment:${name})`;
};
