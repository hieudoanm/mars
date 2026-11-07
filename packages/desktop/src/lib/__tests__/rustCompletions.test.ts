import { CompletionContext } from '@codemirror/autocomplete';
import { EditorState } from '@codemirror/state';
import { rustCompletionSource } from '../rustCompletions';

const complete = (text: string, pos: number) => {
  const state = EditorState.create({ doc: text });
  const context = new CompletionContext(state, pos, true);
  return rustCompletionSource(context);
};

describe('rustCompletionSource mars helpers', () => {
  it('offers mars_table with a snippet apply', () => {
    const result = complete('mars_t', 6);
    const option = result?.options.find((o) => o.label === 'mars_table');
    expect(option).toBeDefined();
    expect(option?.apply).toContain('mars_table(');
    expect(option?.info).toContain('table');
  });

  it('offers mars_csv', () => {
    const result = complete('mars_c', 6);
    expect(result?.options.some((o) => o.label === 'mars_csv')).toBe(true);
  });

  it('does not offer helpers for unrelated prefixes', () => {
    const result = complete('xyz', 3);
    expect(result?.options.some((o) => o.label.startsWith('mars_'))).toBeFalsy();
  });
});
