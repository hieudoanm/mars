import { describe, expect, it } from 'vitest';
import { addCellComment, getCellComments, removeCellComment } from '../cellComments';

describe('getCellComments', () => {
  it('returns an empty array without metadata', () => {
    expect(getCellComments(undefined)).toEqual([]);
    expect(getCellComments({})).toEqual([]);
  });

  it('filters malformed entries', () => {
    const metadata = {
      comments: [
        { text: 'hi', created_at: '2026-01-01T00:00:00Z' },
        { text: 42, created_at: 'x' },
        'nope',
        null,
      ],
    };
    expect(getCellComments(metadata)).toEqual([{ text: 'hi', created_at: '2026-01-01T00:00:00Z' }]);
  });
});

describe('addCellComment', () => {
  it('appends a trimmed comment with a timestamp', () => {
    const next = addCellComment([], '  hello  ');
    expect(next).toHaveLength(1);
    expect(next[0].text).toBe('hello');
    expect(next[0].created_at).toBeTruthy();
  });

  it('returns the same array for blank text', () => {
    const comments = [{ text: 'a', created_at: 'x' }];
    expect(addCellComment(comments, '   ')).toBe(comments);
  });
});

describe('removeCellComment', () => {
  it('removes by index', () => {
    const comments = [
      { text: 'a', created_at: '1' },
      { text: 'b', created_at: '2' },
    ];
    expect(removeCellComment(comments, 0)).toEqual([comments[1]]);
    expect(removeCellComment(comments, 5)).toEqual(comments);
  });
});
