import { describe, expect, it } from 'vitest';
import { diffLines } from '../textDiff';

describe('diffLines', () => {
  it('returns all same rows for identical input', () => {
    expect(diffLines('a\nb\nc', 'a\nb\nc')).toEqual([
      { type: 'same', text: 'a' },
      { type: 'same', text: 'b' },
      { type: 'same', text: 'c' },
    ]);
  });

  it('marks added and deleted lines', () => {
    const rows = diffLines('a\nb\nc', 'a\nx\nc');
    expect(rows).toContainEqual({ type: 'del', text: 'b' });
    expect(rows).toContainEqual({ type: 'add', text: 'x' });
    expect(rows.filter((r) => r.type === 'same')).toHaveLength(2);
  });

  it('handles pure additions at the end', () => {
    const rows = diffLines('a', 'a\nb\nc');
    expect(rows).toEqual([
      { type: 'same', text: 'a' },
      { type: 'add', text: 'b' },
      { type: 'add', text: 'c' },
    ]);
  });

  it('handles pure deletions', () => {
    const rows = diffLines('a\nb\nc', 'a');
    expect(rows).toEqual([
      { type: 'same', text: 'a' },
      { type: 'del', text: 'b' },
      { type: 'del', text: 'c' },
    ]);
  });

  it('handles empty inputs', () => {
    expect(diffLines('', '')).toEqual([{ type: 'same', text: '' }]);
    expect(diffLines('', 'a\nb')).toEqual([
      { type: 'del', text: '' },
      { type: 'add', text: 'a' },
      { type: 'add', text: 'b' },
    ]);
    expect(diffLines('a\nb', '')).toEqual([
      { type: 'del', text: 'a' },
      { type: 'del', text: 'b' },
      { type: 'add', text: '' },
    ]);
  });
});
