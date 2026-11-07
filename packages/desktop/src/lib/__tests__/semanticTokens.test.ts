import { EditorState } from '@codemirror/state';
import { semanticTokensField, setTokensEffect } from '../semanticTokens';

const stateWith = (source: string) =>
  EditorState.create({ doc: source, extensions: [semanticTokensField] });

describe('semanticTokensField', () => {
  it('marks token ranges with type classes', () => {
    const state = stateWith('fn main() {}');
    const tr = state.update({
      effects: setTokensEffect.of([
        { cellId: 'c1', line: 0, character: 0, length: 2, tokenType: 'keyword' },
        { cellId: 'c1', line: 0, character: 3, length: 4, tokenType: 'function' },
      ]),
    });
    const field = tr.state.field(semanticTokensField);
    const classes: string[] = [];
    field.between(0, tr.state.doc.length, (_f, _t, v) => {
      classes.push(v.spec.class as string);
    });
    expect(classes).toEqual(['cm-semantic-keyword', 'cm-semantic-function']);
  });

  it('clamps tokens to line bounds', () => {
    const state = stateWith('ab');
    const tr = state.update({
      effects: setTokensEffect.of([
        { cellId: 'c1', line: 0, character: 1, length: 99, tokenType: 'variable' },
      ]),
    });
    const field = tr.state.field(semanticTokensField);
    let size = 0;
    field.between(0, 10, (_f, _t, v) => {
      size += 1;
      expect(v.spec.class).toBe('cm-semantic-variable');
      return undefined;
    });
    expect(size).toBe(1);
  });
});
