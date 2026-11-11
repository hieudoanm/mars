import { EditorState } from '@codemirror/state';
import { describe, expect, it } from 'vitest';
import { inlayHintsField, setHintsEffect } from '../inlayHints';

const doc = 'fn main() {\n    let x = 1;\n}';

const stateWith = (hints: Parameters<typeof setHintsEffect.of>[0]) => {
  const state = EditorState.create({ doc, extensions: [inlayHintsField] });
  return state.update({ effects: setHintsEffect.of(hints) }).state;
};

const widgets = (state: EditorState) => {
  const field = state.field(inlayHintsField);
  const out: { from: number; to: number; label: string }[] = [];
  field.between(0, state.doc.length, (from, to, value) => {
    const spec = value.spec as { widget?: { toDOM?: () => HTMLElement } };
    const el = spec.widget?.toDOM?.();
    out.push({ from, to, label: el?.textContent ?? '' });
  });
  return out;
};

describe('inlayHints', () => {
  it('places hint widgets at the requested line/character', () => {
    const state = stateWith([{ line: 1, character: 4, label: 'x: i32' }]);
    const found = widgets(state);
    expect(found).toHaveLength(1);
    expect(found[0].label).toBe('x: i32');
    // line 1 (0-based) is "    let x = 1;" — char 4 → doc offset
    expect(found[0].from).toBe('fn main() {\n    '.length);
  });

  it('clamps out-of-range positions into the document', () => {
    const state = stateWith([
      { line: 99, character: 500, label: 'far' },
      { line: 0, character: -5, label: 'neg' },
    ]);
    const found = widgets(state);
    expect(found).toHaveLength(2);
    // sorted by position: 'neg' (pos 0) first, 'far' (doc end) last
    expect(found[0].label).toBe('neg');
    expect(found[0].from).toBe(0);
    expect(found[1].label).toBe('far');
    expect(found[1].from).toBe(state.doc.length);
  });

  it('renders widget DOM with kind class and padding', () => {
    const state = stateWith([
      {
        line: 0,
        character: 3,
        label: 'arg',
        kind: 'parameter',
        paddingLeft: true,
        paddingRight: true,
      },
      { line: 0, character: 8, label: 'plain' },
    ]);
    const field = state.field(inlayHintsField);
    const doms: HTMLElement[] = [];
    field.between(0, state.doc.length, (_f, _t, value) => {
      const spec = value.spec as { widget?: { toDOM?: () => HTMLElement } };
      const el = spec.widget?.toDOM?.();
      if (el) doms.push(el);
    });
    expect(doms).toHaveLength(2);
    expect(doms[0].className).toContain('cm-mars-inlayhint-param');
    expect(doms[0].textContent).toBe('arg');
    expect(doms[0].style.paddingLeft).toBe('0.4em');
    expect(doms[0].getAttribute('aria-hidden')).toBe('true');
    expect(doms[1].className).not.toContain('param');
    expect(doms[1].style.paddingLeft).toBe('0.4em'); // default paddingLeft ?? true
    expect(doms[1].style.paddingRight).toBe('0px'); // default paddingRight ?? false
  });

  it('clears hints when an empty set is applied', () => {
    let state = stateWith([{ line: 0, character: 0, label: 'x' }]);
    expect(widgets(state)).toHaveLength(1);
    state = state.update({ effects: setHintsEffect.of([]) }).state;
    expect(widgets(state)).toHaveLength(0);
  });

  it('keeps decorations mapped across document changes', () => {
    let state = stateWith([{ line: 2, character: 0, label: 'end' }]);
    state = state.update({ changes: { from: 0, insert: 'new line\n' } }).state;
    const found = widgets(state);
    expect(found).toHaveLength(1);
    expect(found[0].from).toBeGreaterThan('new line\n'.length);
  });
});
