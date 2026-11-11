import { describe, expect, it } from 'vitest';
import { applyTextEdits, offsetToLineChar } from '../cellTextEdit';
import type { LspCellTextEdit } from '../kernelService';

const edit = (partial: Partial<LspCellTextEdit>): LspCellTextEdit => ({
  cellId: 'c1',
  startLine: 0,
  startCharacter: 0,
  endLine: 0,
  endCharacter: 0,
  newText: '',
  ...partial,
});

describe('offsetToLineChar', () => {
  it('converts offsets to line/character', () => {
    expect(offsetToLineChar('ab\ncd\nef', 0)).toEqual({ line: 0, character: 0 });
    expect(offsetToLineChar('ab\ncd\nef', 3)).toEqual({ line: 1, character: 0 });
    expect(offsetToLineChar('ab\ncd\nef', 5)).toEqual({ line: 1, character: 2 });
    expect(offsetToLineChar('ab\ncd\nef', 8)).toEqual({ line: 2, character: 2 });
  });

  it('clamps out-of-range offsets', () => {
    expect(offsetToLineChar('ab', -5)).toEqual({ line: 0, character: 0 });
    expect(offsetToLineChar('ab', 99)).toEqual({ line: 0, character: 2 });
  });
});

describe('applyTextEdits', () => {
  it('replaces a single-line range', () => {
    expect(
      applyTextEdits('hello world', [
        edit({ startCharacter: 6, endCharacter: 11, newText: 'mars' }),
      ]),
    ).toBe('hello mars');
  });

  it('handles multi-line ranges', () => {
    expect(
      applyTextEdits('fn main() {\n  old();\n}\n', [
        edit({ startLine: 1, startCharacter: 2, endLine: 1, endCharacter: 8, newText: 'new();' }),
      ]),
    ).toBe('fn main() {\n  new();\n}\n');
  });

  it('applies multiple edits right-to-left regardless of input order', () => {
    expect(
      applyTextEdits('abcdef', [
        edit({ startCharacter: 0, endCharacter: 1, newText: 'X' }),
        edit({ startCharacter: 5, endCharacter: 6, newText: 'Y' }),
      ]),
    ).toBe('XbcdeY');
  });

  it('inserts text at a point', () => {
    expect(applyTextEdits('ac', [edit({ startCharacter: 1, endCharacter: 1, newText: 'b' })])).toBe(
      'abc',
    );
  });

  it('returns the source unchanged for empty edits', () => {
    expect(applyTextEdits('unchanged', [])).toBe('unchanged');
  });
});
