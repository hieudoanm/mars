import { syntaxTree } from '@codemirror/language';

type SyntaxNode = ReturnType<ReturnType<typeof syntaxTree>['resolveInner']>;
import { EditorState } from '@codemirror/state';
import { EditorView, ViewPlugin, ViewUpdate } from '@codemirror/view';

const DECLARATION_TYPES = new Set([
  'FunctionItem',
  'ImplItem',
  'TraitItem',
  'StructItem',
  'EnumItem',
  'UnionItem',
  'ModItem',
  'ConstItem',
  'StaticItem',
  'TypeItem',
  'MacroRules',
]);

const enclosingDeclaration = (state: EditorState, pos: number): SyntaxNode | null => {
  const posLine = state.doc.lineAt(pos).number;
  let node: SyntaxNode | null = syntaxTree(state).resolveInner(pos, 1);
  while (node) {
    if (DECLARATION_TYPES.has(node.name)) {
      // Only pin declarations whose header starts above the viewport; the
      // first match walking up is the innermost enclosing declaration.
      if (state.doc.lineAt(node.from).number < posLine) return node;
    }
    node = node.parent;
  }
  return null;
};

/** First line of the innermost enclosing Rust declaration above `pos`. */
export const stickyHeaderAt = (state: EditorState, pos: number): string | null => {
  const decl = enclosingDeclaration(state, pos);
  if (!decl) return null;
  return state.doc.lineAt(decl.from).text.trimEnd();
};

/** Pins the innermost enclosing Rust declaration header while scrolling. */
export const stickyScroll = () =>
  ViewPlugin.fromClass(
    class {
      private readonly dom: HTMLElement;
      private frame: number | null = null;

      constructor(view: EditorView) {
        this.dom = document.createElement('div');
        this.dom.className = 'cm-mars-sticky';
        this.dom.setAttribute('aria-hidden', 'true');
        view.dom.appendChild(this.dom);
        this.scheduleRefresh(view);
      }

      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged || update.geometryChanged) {
          this.scheduleRefresh(update.view);
        }
      }

      // Layout reads are forbidden during view updates, so refresh on the
      // next animation frame instead.
      private scheduleRefresh(view: EditorView) {
        if (this.frame !== null) return;
        this.frame = requestAnimationFrame(() => {
          this.frame = null;
          if (view.dom.isConnected) this.refresh(view);
        });
      }

      private refresh(view: EditorView) {
        const block = view.lineBlockAtHeight(view.scrollDOM.scrollTop);
        if (!block) {
          this.dom.style.display = 'none';
          return;
        }
        const pos = block.from;
        const header = stickyHeaderAt(view.state, pos);
        if (!header) {
          this.dom.style.display = 'none';
          return;
        }
        this.dom.textContent = header;
        // Align with the text column, past the line-number gutter.
        const gutterWidth = Math.max(
          0,
          view.contentDOM.getBoundingClientRect().left - view.dom.getBoundingClientRect().left,
        );
        this.dom.style.left = `${gutterWidth}px`;
        this.dom.style.display = '';
      }

      destroy() {
        if (this.frame !== null) cancelAnimationFrame(this.frame);
        this.dom.remove();
      }
    },
  );
