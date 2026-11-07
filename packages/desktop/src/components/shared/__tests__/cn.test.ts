import { cn } from '../cn';

describe('cn', () => {
  it('joins truthy class names', () => {
    expect(cn('a', 'b', 'c')).toBe('a b c');
  });

  it('drops empty values', () => {
    expect(cn('a', false, null, undefined, '', 'b')).toBe('a b');
  });

  it('stringifies numbers', () => {
    expect(cn('gap', 0, 2)).toBe('gap 0 2');
  });

  it('returns an empty string when there are no values', () => {
    expect(cn()).toBe('');
  });
});
