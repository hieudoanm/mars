import { CompletionContext } from '@codemirror/autocomplete';
import { EditorState } from '@codemirror/state';
import { rustCompletionSource, updateCompletionSources } from '../rustCompletions';

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

  it('offers Rust keywords with keyword info', () => {
    const result = complete('let', 3);
    const option = result?.options.find((o) => o.label === 'let');
    expect(option?.type).toBe('keyword');
    expect(option?.info).toBe('Rust keyword');
  });

  it('offers primitives and macros', () => {
    const primitives = complete('u32', 3);
    expect(primitives?.options.some((o) => o.label === 'u32')).toBe(true);
    const macros = complete('println', 7);
    const println = macros?.options.find((o) => o.label === 'println!');
    expect(println?.apply).toBe('println!()');
  });

  it('returns null when nothing matches', () => {
    const result = complete('zzz_no_match', 12);
    expect(result).toBeNull();
  });

  it('offers notebook symbols by kind', () => {
    updateCompletionSources(['fn compute_total() {} let helper_value = 1; struct Config {}']);
    const result = complete('comp', 4);
    const symbol = result?.options.find((o) => o.label === 'compute_total');
    expect(symbol?.type).toBe('function');
    const binding = complete('helper', 6)?.options.find((o) => o.label === 'helper_value');
    expect(binding?.type).toBe('variable');
    const ty = complete('Conf', 4)?.options.find((o) => o.label === 'Config');
    expect(ty?.type).toBe('class');
    updateCompletionSources([]);
  });
});
