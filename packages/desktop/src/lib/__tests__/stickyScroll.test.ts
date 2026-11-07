import { EditorState } from '@codemirror/state';
import { ensureSyntaxTree } from '@codemirror/language';
import { rust } from '@codemirror/lang-rust';
import { stickyHeaderAt } from '../stickyScroll';

const SOURCE = `fn helper() -> i32 {
    40
}

fn main() {
    let x = helper();
    let y = x + 2;
    println!("{y}");
}`;

const state = () => {
  const s = EditorState.create({ doc: SOURCE, extensions: [rust()] });
  ensureSyntaxTree(s, s.doc.length, 100);
  return s;
};

describe('stickyHeaderAt', () => {
  it('returns the enclosing fn signature', () => {
    const s = state();
    const pos = s.doc.line(6).from; // inside main
    expect(stickyHeaderAt(s, pos)).toBe('fn main() {');
  });

  it('returns impl/trait-level headers for nested items', () => {
    const s = EditorState.create({
      doc: 'struct S;\nimpl S {\n    fn method(&self) {\n        let a = 1;\n    }\n}',
      extensions: [rust()],
    });
    ensureSyntaxTree(s, s.doc.length, 100);
    const pos = s.doc.line(4).from;
    expect(stickyHeaderAt(s, pos)).toBe('    fn method(&self) {');
  });

  it('returns null before any declaration', () => {
    const s = state();
    expect(stickyHeaderAt(s, 0)).toBeNull();
  });
});
