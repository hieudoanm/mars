import { describe, expect, it, vi } from 'vitest';
import { addCellAttachment, getCellAttachments } from '../cellAttachments';
import type { NotebookCell } from '../../types/types';

const cell = (metadata: Record<string, unknown> = {}): NotebookCell => ({
  id: 'c1',
  cell_type: 'markdown',
  source: '',
  metadata,
  execution_count: null,
  outputs: [],
});

describe('getCellAttachments', () => {
  it('returns an empty map for missing or invalid attachments', () => {
    expect(getCellAttachments({})).toEqual({});
    expect(getCellAttachments({ attachments: 'nope' })).toEqual({});
    expect(getCellAttachments({ attachments: [1, 2] })).toEqual({});
    expect(getCellAttachments({ attachments: { a: 42, b: 'https://x' } })).toEqual({});
  });

  it('keeps only string data URLs', () => {
    expect(
      getCellAttachments({
        attachments: { 'a.png': 'data:image/png;base64,AA==', 'b.txt': 'plain' },
      }),
    ).toEqual({ 'a.png': 'data:image/png;base64,AA==' });
  });
});

describe('addCellAttachment', () => {
  it('generates a unique name and markdown reference', () => {
    const addAttachments = vi.fn();
    const ref = addCellAttachment(
      cell(),
      'data:image/png;base64,AA==',
      'image/png',
      addAttachments,
    );
    expect(ref).toMatch(/^!\[image-1\.png\]\(attachment:image-1\.png\)$/);
    expect(addAttachments).toHaveBeenCalledWith({ 'image-1.png': 'data:image/png;base64,AA==' });
  });

  it('skips names already present on the cell', () => {
    const addAttachments = vi.fn();
    const existing = cell({
      attachments: { 'image-1.png': 'data:image/png;base64,AA==' },
    });
    const ref = addCellAttachment(
      existing,
      'data:image/png;base64,BB==',
      'image/png',
      addAttachments,
    );
    expect(ref).toContain('attachment:image-2-');
    expect(ref).toContain('.png)');
  });

  it('falls back to the mime subtype for unknown types', () => {
    const addAttachments = vi.fn();
    const ref = addCellAttachment(
      cell(),
      'data:application/x-custom;base64,AA==',
      'application/x-custom',
      addAttachments,
    );
    expect(ref).toContain('.x-custom');
  });
});
