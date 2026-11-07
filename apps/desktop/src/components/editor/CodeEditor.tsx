import { markdown } from '@codemirror/lang-markdown';
import { rust } from '@codemirror/lang-rust';
import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from '@codemirror/autocomplete';
import { defaultKeymap, indentWithTab } from '@codemirror/commands';
import {
  bracketMatching,
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
  indentOnInput,
  syntaxHighlighting,
} from '@codemirror/language';
import { EditorState } from '@codemirror/state';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView, keymap, ViewUpdate } from '@codemirror/view';
import { search, searchKeymap } from '@codemirror/search';
import { setDiagnostics } from '@codemirror/lint';
import { type FC, useEffect, useRef } from 'react';
import type { CellType } from '../../types/types';
import type { Diagnostic } from '@codemirror/lint';

type CodeEditorProps = {
  cellType: CellType;
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
  onFocus: () => void;
  diagnostics?: Diagnostic[];
  fontSize?: number;
  autocompleteEnabled?: boolean;
  foldingEnabled?: boolean;
  onViewReady?: (view: EditorView | null) => void;
};

const CodeEditor: FC<CodeEditorProps> = ({
  cellType,
  value,
  placeholder,
  onChange,
  onFocus,
  diagnostics = [],
  fontSize = 13,
  autocompleteEnabled = true,
  foldingEnabled = true,
  onViewReady,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  const onFocusRef = useRef(onFocus);
  const onViewReadyRef = useRef(onViewReady);
  const valueRef = useRef(value);

  onChangeRef.current = onChange;
  onFocusRef.current = onFocus;
  onViewReadyRef.current = onViewReady;
  valueRef.current = value;

  useEffect(() => {
    if (!containerRef.current) return;
    const updateListener = EditorView.updateListener.of((update: ViewUpdate) => {
      if (update.docChanged) {
        onChangeRef.current(update.state.doc.toString());
      }
      if (update.focusChanged && update.view.hasFocus) {
        onFocusRef.current();
      }
    });
    const view = new EditorView({
      state: EditorState.create({
        doc: valueRef.current,
        extensions: [
          oneDark,
          keymap.of([
            ...defaultKeymap,
            ...foldKeymap,
            ...closeBracketsKeymap,
            ...completionKeymap,
            indentWithTab,
          ]),
          closeBrackets(),
          bracketMatching(),
          foldingEnabled ? foldGutter() : [],
          indentOnInput(),
          autocompleteEnabled ? autocompletion({ activateOnTyping: true }) : [],
          syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
          EditorView.lineWrapping,
          updateListener,
          EditorView.theme({
            '&': {
              backgroundColor: '#242424',
              color: '#e5e5e5',
              minHeight: '44px',
              border: '1px solid #333',
              borderRadius: '7px',
              fontSize: `${fontSize}px`,
            },
            '&.cm-focused': {
              outline: 'none',
              borderColor: '#c2410c',
            },
            '.cm-scroller': {
              minHeight: '44px',
              padding: '10px 14px',
            },
            '.cm-content': {
              minHeight: '24px',
              padding: '0',
            },
            '.cm-gutters': {
              display: 'none',
            },
            '.cm-placeholder': {
              color: '#77736e',
            },
          }),
          EditorView.contentAttributes.of({
            'aria-label': placeholder,
            'data-placeholder': placeholder,
          }),
          search(),
          keymap.of(searchKeymap),
          cellType === 'markdown' ? markdown() : cellType === 'code' ? rust() : [],
        ],
      }),
      parent: containerRef.current,
    });
    viewRef.current = view;
    onViewReadyRef.current?.(view);
    return () => {
      view.destroy();
      viewRef.current = null;
      onViewReadyRef.current?.(null);
    };
  }, [cellType, placeholder, fontSize, autocompleteEnabled, foldingEnabled]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch(setDiagnostics(view.state, diagnostics));
  }, [diagnostics]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const currentValue = view.state.doc.toString();
    if (currentValue === value) return;
    view.dispatch({
      changes: { from: 0, to: currentValue.length, insert: value },
    });
  }, [value]);

  return <div data-component='CodeEditor' className='block w-full' ref={containerRef} />;
};
CodeEditor.displayName = 'CodeEditor';
export default CodeEditor;
