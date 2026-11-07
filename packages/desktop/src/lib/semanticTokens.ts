import {
  RangeSetBuilder,
  StateEffect,
  StateField,
  type EditorState,
  type RangeSet,
} from '@codemirror/state';
import { Decoration, EditorView } from '@codemirror/view';
import type { LspSemanticToken } from './kernelService';

const setTokensEffect = StateEffect.define<LspSemanticToken[]>();

const tokenClass = (tokenType: string) =>
  `cm-semantic-${tokenType.replace(/[^a-zA-Z]/g, '').toLowerCase()}`;

const buildDecorations = (tokens: LspSemanticToken[], state: EditorState) => {
  const builder = new RangeSetBuilder<Decoration>();
  for (const token of tokens) {
    const line = state.doc.line(Math.min(token.line + 1, state.doc.lines));
    const start = line.from + Math.min(Math.max(token.character, 0), line.length);
    const end = Math.min(start + Math.max(token.length, 0), line.to);
    if (end <= start) continue;
    builder.add(start, end, Decoration.mark({ class: tokenClass(token.tokenType) }));
  }
  return builder.finish();
};

export const semanticTokensField = StateField.define<RangeSet<Decoration>>({
  create: (state) => buildDecorations([], state),
  update: (decorations, transaction) => {
    let next = decorations.map(transaction.changes);
    for (const effect of transaction.effects) {
      if (effect.is(setTokensEffect)) {
        next = buildDecorations(effect.value, transaction.state);
      }
    }
    return next;
  },
  provide: (field) => EditorView.decorations.from(field),
});

export { setTokensEffect };
