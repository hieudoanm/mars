import { describe, expect, it } from 'vitest';
import { createSearchPattern, DEFAULT_SEARCH_OPTIONS, matchesPattern } from '../textSearch';

describe('createSearchPattern', () => {
  it('returns null for empty or whitespace queries', () => {
    expect(createSearchPattern('', DEFAULT_SEARCH_OPTIONS)).toBeNull();
    expect(createSearchPattern('   ', DEFAULT_SEARCH_OPTIONS)).toBeNull();
  });

  it('matches case-insensitively by default', () => {
    const pattern = createSearchPattern('hello', DEFAULT_SEARCH_OPTIONS);
    expect(matchesPattern(pattern!, 'say HELLO world')).toBe(true);
    expect(matchesPattern(pattern!, 'say goodbye')).toBe(false);
  });

  it('respects caseSensitive option', () => {
    const pattern = createSearchPattern('hello', {
      ...DEFAULT_SEARCH_OPTIONS,
      caseSensitive: true,
    });
    expect(matchesPattern(pattern!, 'HELLO')).toBe(false);
    expect(matchesPattern(pattern!, 'hello')).toBe(true);
  });

  it('respects wholeWord option', () => {
    const pattern = createSearchPattern('cat', {
      ...DEFAULT_SEARCH_OPTIONS,
      wholeWord: true,
    });
    expect(matchesPattern(pattern!, 'the cat sat')).toBe(true);
    expect(matchesPattern(pattern!, 'concatenate')).toBe(false);
    expect(matchesPattern(pattern!, 'cat.')).toBe(true);
  });

  it('escapes regex metacharacters in literal mode', () => {
    const pattern = createSearchPattern('a.b', DEFAULT_SEARCH_OPTIONS);
    expect(matchesPattern(pattern!, 'a.b')).toBe(true);
    expect(matchesPattern(pattern!, 'axb')).toBe(false);
  });

  it('treats query as regex when regex option is set', () => {
    const pattern = createSearchPattern('fn\\s+\\w+', {
      ...DEFAULT_SEARCH_OPTIONS,
      regex: true,
    });
    expect(matchesPattern(pattern!, 'fn main()')).toBe(true);
    expect(matchesPattern(pattern!, 'fnmain')).toBe(false);
  });

  it('returns null for invalid regex', () => {
    expect(createSearchPattern('([', { ...DEFAULT_SEARCH_OPTIONS, regex: true })).toBeNull();
  });
});
