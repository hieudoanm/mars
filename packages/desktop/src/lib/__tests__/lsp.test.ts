import { describe, expect, it } from 'vitest';
import { lspHoverText, signatureHelpText } from '../lsp';

describe('lspHoverText', () => {
  it('returns null for missing or empty results', () => {
    expect(lspHoverText(null)).toBeNull();
    expect(lspHoverText({ contents: [] })).toBeNull();
    expect(lspHoverText({ contents: { kind: 'markdown', value: '  ' } })).toBeNull();
  });

  it('extracts markdown and plaintext hover contents', () => {
    expect(lspHoverText({ contents: { kind: 'markdown', value: '```rust\nfn main()\n```' } })).toBe(
      '```rust\nfn main()\n```',
    );
    expect(lspHoverText({ contents: [{ kind: 'plaintext', value: 'a' }, 'b' as never] })).toBe('a');
  });

  it('joins multiple content parts', () => {
    expect(
      lspHoverText({
        contents: [
          { kind: 'markdown', value: 'first' },
          { kind: 'plaintext', value: 'second' },
        ],
      }),
    ).toBe('first\n\nsecond');
  });
});

describe('signatureHelpText', () => {
  it('returns null without signatures', () => {
    expect(signatureHelpText(null)).toBeNull();
    expect(signatureHelpText({})).toBeNull();
    expect(signatureHelpText({ signatures: [] })).toBeNull();
    expect(signatureHelpText({ signatures: [{ label: 1 }] })).toBeNull();
  });

  it('formats the active signature label', () => {
    expect(
      signatureHelpText({
        signatures: [{ label: 'fn foo(x: i32)' }],
        activeSignature: 0,
      }),
    ).toBe('fn foo(x: i32)');
  });

  it('appends the active parameter label', () => {
    expect(
      signatureHelpText({
        signatures: [
          {
            label: 'fn foo(x: i32, y: i32)',
            parameters: [{ label: 'x: i32' }, { label: 'y: i32' }],
          },
        ],
        activeSignature: 0,
        activeParameter: 1,
      }),
    ).toBe('fn foo(x: i32, y: i32)\n↳ y: i32');
  });

  it('falls back to the first signature for out-of-range indices', () => {
    expect(
      signatureHelpText({
        signatures: [{ label: 'a' }, { label: 'b' }],
        activeSignature: 99,
      }),
    ).toBe('a');
  });
});
