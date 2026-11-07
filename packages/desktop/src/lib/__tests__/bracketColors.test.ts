import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { rust } from '@codemirror/lang-rust';
import { bracketColorizer } from '../bracketColors';

const buildView = (doc: string) => {
  const state = EditorState.create({
    doc,
    extensions: [rust(), bracketColorizer()],
  });
  return new EditorView({ state });
};

describe('bracketColorizer', () => {
  it('marks brackets from the syntax tree', () => {
    const view = buildView('fn main() { let x = (1 + 2); }');
    document.body.appendChild(view.dom);
    const marks = view.dom.querySelectorAll('.cm-bracket');
    expect(marks.length).toBeGreaterThanOrEqual(6);
    view.destroy();
  });

  it('does not color brackets inside strings', () => {
    const view = buildView('fn main() { let s = "("; }');
    document.body.appendChild(view.dom);
    const marks = view.dom.querySelectorAll('.cm-bracket');
    // fn( ) { } = 4 brackets; the "(" inside the string must not be marked
    expect(marks.length).toBe(4);
    view.destroy();
  });
});
