import { syntaxTree } from '@codemirror/language';
import { RangeSetBuilder } from '@codemirror/state';
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from '@codemirror/view';

const BRACKET_COLORS = [
  '#f87171',
  '#fb923c',
  '#facc15',
  '#4ade80',
  '#60a5fa',
  '#a78bfa',
  '#f472b6',
];

const OPENERS = new Set(['(', '[', '{']);
const CLOSERS = new Set([')', ']', '}']);

const bracketDecorations = (view: EditorView): DecorationSet => {
  const builder = new RangeSetBuilder<Decoration>();
  const stack: number[] = [];
  syntaxTree(view.state).iterate({
    enter(node) {
      if (OPENERS.has(node.name)) {
        const depth = stack.length % BRACKET_COLORS.length;
        stack.push(depth);
        builder.add(
          node.from,
          node.to,
          Decoration.mark({
            class: `cm-bracket cm-bracket-${depth}`,
            attributes: { 'data-depth': String(depth) },
          }),
        );
      } else if (CLOSERS.has(node.name)) {
        const depth = stack.pop() ?? 0;
        builder.add(
          node.from,
          node.to,
          Decoration.mark({
            class: `cm-bracket cm-bracket-${depth}`,
            attributes: { 'data-depth': String(depth) },
          }),
        );
      }
    },
  });
  return builder.finish();
};

export const bracketColorizer = () =>
  ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = bracketDecorations(view);
      }
      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = bracketDecorations(update.view);
        }
      }
    },
    { decorations: (plugin) => plugin.decorations },
  );
