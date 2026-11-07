import { describe, expect, it } from 'vitest';
import { cellSourceHash } from '../cellHash';

describe('cellSourceHash', () => {
  it('is deterministic and matches the Rust djb2 implementation', () => {
    // Values computed by mars_runtime::expression::cell_source_hash.
    expect(cellSourceHash('1 + 1')).toBe(193460274);
    expect(cellSourceHash('let x = 1;')).toBe(2796018187);
    expect(cellSourceHash('')).toBe(5381);
  });

  it('hashes UTF-8 bytes, not UTF-16 code units', () => {
    expect(cellSourceHash('é')).toBe(cellSourceHash('é'));
    expect(cellSourceHash('é')).not.toBe(cellSourceHash('é'));
  });
});
