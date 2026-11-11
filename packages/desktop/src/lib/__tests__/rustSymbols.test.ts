import { describe, expect, it } from 'vitest';
import { extractRustSymbols } from '../rustSymbols';

describe('extractRustSymbols', () => {
  it('collects bindings, functions, and types', () => {
    const source = `
      let mut count = 0;
      const NAME = "mars";
      fn helper() {}
      struct Point { x: f64 }
      enum Color { Red }
      trait Drawable {}
      type Result<T> = std::result::Result<T, E>;
    `;
    const symbols = extractRustSymbols([source]);
    const byName = new Map(symbols.map((s) => [s.name, s.kind]));
    expect(byName.get('count')).toBe('binding');
    expect(byName.get('NAME')).toBe('binding');
    expect(byName.get('helper')).toBe('function');
    expect(byName.get('Point')).toBe('type');
    expect(byName.get('Color')).toBe('type');
    expect(byName.get('Drawable')).toBe('type');
    expect(byName.get('Result')).toBe('type');
  });

  it('merges symbols across sources and dedupes by name', () => {
    const symbols = extractRustSymbols(['fn run() {}', 'fn run() { let x = 1; } let y = 2;']);
    expect(symbols.filter((s) => s.name === 'run')).toHaveLength(1);
    expect(symbols.map((s) => s.name)).toContain('x');
    expect(symbols.map((s) => s.name)).toContain('y');
  });

  it('returns symbols sorted by name', () => {
    const symbols = extractRustSymbols(['fn zebra() {} fn alpha() {}']);
    expect(symbols.map((s) => s.name)).toEqual(['alpha', 'zebra']);
  });

  it('ignores non-matching text', () => {
    // regex-based: comments are not stripped, so fn inside a comment still matches
    const symbols = extractRustSymbols(['let x = 1; // fn commented() {}']);
    expect(symbols).toContainEqual({ name: 'x', kind: 'binding' });
    expect(symbols).toContainEqual({ name: 'commented', kind: 'function' });
    expect(extractRustSymbols(['plain text only'])).toEqual([]);
    expect(extractRustSymbols([])).toEqual([]);
  });
});
