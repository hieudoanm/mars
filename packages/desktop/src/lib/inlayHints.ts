import { EditorState, RangeSetBuilder, StateEffect, StateField } from '@codemirror/state';
import { Decoration, EditorView, WidgetType } from '@codemirror/view';

export type InlayHint = {
  line: number;
  character: number;
  label: string;
  kind?: string;
  paddingLeft?: boolean;
  paddingRight?: boolean;
};

const setHintsEffect = StateEffect.define<InlayHint[]>();

class InlayHintWidget extends WidgetType {
  constructor(
    private readonly label: string,
    private readonly kind: string | undefined,
    private readonly paddingLeft: boolean,
    private readonly paddingRight: boolean,
  ) {
    super();
  }

  eq(other: InlayHintWidget): boolean {
    return (
      other.label === this.label &&
      other.kind === this.kind &&
      other.paddingLeft === this.paddingLeft &&
      other.paddingRight === this.paddingRight
    );
  }

  toDOM(): HTMLElement {
    const element = document.createElement('span');
    element.className = `cm-mars-inlayhint${this.kind === 'parameter' ? ' cm-mars-inlayhint-param' : ''}`;
    element.textContent = this.label;
    element.style.paddingLeft = this.paddingLeft ? '0.4em' : '0';
    element.style.paddingRight = this.paddingRight ? '0.4em' : '0';
    element.setAttribute('aria-hidden', 'true');
    return element;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

const hintPosition = (state: EditorState, hint: InlayHint): number => {
  const line = state.doc.line(Math.min(Math.max(hint.line + 1, 1), state.doc.lines));
  const offset = Math.min(Math.max(hint.character, 0), line.length);
  return line.from + offset;
};

const buildDecorations = (hints: InlayHint[], state: EditorState) => {
  const builder = new RangeSetBuilder<Decoration>();
  const positioned = hints
    .map((hint) => ({ hint, pos: hintPosition(state, hint) }))
    .sort((a, b) => a.pos - b.pos);
  for (const { hint, pos } of positioned) {
    builder.add(
      pos,
      pos,
      Decoration.widget({
        widget: new InlayHintWidget(
          hint.label,
          hint.kind,
          hint.paddingLeft ?? true,
          hint.paddingRight ?? false,
        ),
        side: 1,
      }),
    );
  }
  return builder.finish();
};

export const inlayHintsField = StateField.define<ReturnType<typeof buildDecorations>>({
  create: (state) => buildDecorations([], state),
  update: (decorations, transaction) => {
    let next = decorations.map(transaction.changes);
    for (const effect of transaction.effects) {
      if (effect.is(setHintsEffect)) {
        next = buildDecorations(effect.value, transaction.state);
      }
    }
    return next;
  },
  provide: (field) => EditorView.decorations.from(field),
});

export { setHintsEffect };
