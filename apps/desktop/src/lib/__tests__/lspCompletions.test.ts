import { describe, expect, it, vi } from 'vitest';
import { EditorState } from '@codemirror/state';
import type { CompletionContext } from '@codemirror/autocomplete';
import { createLspCompletionSource } from '../lspCompletions';
import type { LspCompletionItem } from '../kernelService';

const contextAt = (doc: string, pos: number): CompletionContext => {
  const state = EditorState.create({ doc });
  const matchBefore = (pattern: RegExp) => {
    const before = doc.slice(0, pos);
    const match = new RegExp(`(?:^|\\W)(${pattern.source})$`).exec(before);
    if (!match) return null;
    const from = pos - match[1].length;
    return { from, text: match[1], to: pos };
  };
  return {
    state,
    pos,
    explicit: false,
    tokenBefore: undefined,
    matchBefore,
  } as unknown as CompletionContext;
};

describe('createLspCompletionSource', () => {
  it('maps LSP items to CodeMirror completions', async () => {
    const items: LspCompletionItem[] = [
      { label: 'println!', kind: 3, detail: 'macro', insertText: 'println!' },
      { label: 'Vec', kind: 7 },
    ];
    const source = createLspCompletionSource(vi.fn().mockResolvedValue({ items }));
    const result = await source(contextAt('fn main() { pri', 15));
    expect(result).not.toBeNull();
    expect(result?.options).toHaveLength(2);
    expect(result?.options[0]).toMatchObject({
      label: 'println!',
      type: 'function',
      detail: 'macro',
      apply: 'println!',
    });
    expect(result?.options[1]).toMatchObject({ label: 'Vec', type: 'class' });
  });

  it('accepts a bare array response', async () => {
    const source = createLspCompletionSource(vi.fn().mockResolvedValue([{ label: 'x', kind: 6 }]));
    const result = await source(contextAt('let x', 5));
    expect(result?.options[0]?.label).toBe('x');
  });

  it('returns null for empty or failed fetches', async () => {
    const empty = createLspCompletionSource(vi.fn().mockResolvedValue({ items: [] }));
    expect(await empty(contextAt('abc', 3))).toBeNull();
    const failing = createLspCompletionSource(vi.fn().mockRejectedValue(new Error('lsp down')));
    expect(await failing(contextAt('abc', 3))).toBeNull();
    const nil = createLspCompletionSource(vi.fn().mockResolvedValue(null));
    expect(await nil(contextAt('abc', 3))).toBeNull();
  });

  it('passes zero-based line and character to the fetcher', async () => {
    const fetcher = vi.fn().mockResolvedValue({ items: [{ label: 'a' }] });
    const source = createLspCompletionSource(fetcher);
    await source(contextAt('one\ntwo three', 11));
    expect(fetcher).toHaveBeenCalledWith(1, 7);
  });
});
