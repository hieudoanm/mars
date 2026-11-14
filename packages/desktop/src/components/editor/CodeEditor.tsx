import { markdown } from '@codemirror/lang-markdown';
import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
  type CompletionSource,
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
import { indentationMarkers } from '@replit/codemirror-indentation-markers';
import { oneDark } from '@codemirror/theme-one-dark';
import {
  EditorView,
  dropCursor,
  drawSelection,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  hoverTooltip,
  keymap,
  lineNumbers,
  rectangularSelection,
  ViewUpdate,
} from '@codemirror/view';
import { highlightSelectionMatches, search, searchKeymap } from '@codemirror/search';
import { setDiagnostics } from '@codemirror/lint';
import { type FC, useEffect, useRef } from 'react';
import type { CellType } from '../../types/types';
import type { Diagnostic } from '@codemirror/lint';
import { csvSnippet } from '../../lib/cellCsv';
import { inlayHintsField, setHintsEffect } from '../../lib/inlayHints';
import { semanticTokensField, setTokensEffect } from '../../lib/semanticTokens';
import { stickyScroll } from '../../lib/stickyScroll';
import { languageDescriptor } from '../../lib/languages';
import { bracketColorizer } from '../../lib/bracketColors';

type CodeEditorProps = {
  cellType: CellType;
  codeLanguage?: string;
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
  onFocus: () => void;
  diagnostics?: Diagnostic[];
  fontSize?: number;
  autocompleteEnabled?: boolean;
  foldingEnabled?: boolean;
  lineNumbersEnabled?: boolean;
  colorScheme?: 'dark' | 'light';
  onViewReady?: (view: EditorView | null) => void;
  onHoverRequest?: (line: number, character: number) => Promise<string | null>;
  extraCompletionSource?: CompletionSource;
  onSignatureRequest?: (line: number, character: number) => Promise<string | null>;
  onDefinitionRequest?: (
    line: number,
    character: number,
  ) => Promise<{ cellId: string; line: number; character: number } | null>;
  onDefinitionResult?: (target: { cellId: string; line: number; character: number }) => void;
  inlayHints?: import('../../lib/inlayHints').InlayHint[];
  semanticTokens?: import('../../lib/kernelService').LspSemanticToken[];
  onImageAttachment?: (dataUrl: string, mimeType: string) => string | null;
};

const readFileAsDataUrl = (file: File): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });

const CodeEditor: FC<CodeEditorProps> = ({
  cellType,
  codeLanguage = 'rust',
  value,
  placeholder,
  onChange,
  onFocus,
  diagnostics = [],
  fontSize = 13,
  autocompleteEnabled = true,
  foldingEnabled = true,
  lineNumbersEnabled = true,
  colorScheme = 'dark',
  onViewReady,
  onHoverRequest,
  extraCompletionSource,
  onSignatureRequest,
  onDefinitionRequest,
  onDefinitionResult,
  onImageAttachment,
  inlayHints = [],
  semanticTokens = [],
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  const onImageAttachmentRef = useRef(onImageAttachment);
  onImageAttachmentRef.current = onImageAttachment;
  const onFocusRef = useRef(onFocus);
  const onViewReadyRef = useRef(onViewReady);
  const onHoverRequestRef = useRef(onHoverRequest);
  const onSignatureRequestRef = useRef(onSignatureRequest);
  const onDefinitionRequestRef = useRef(onDefinitionRequest);
  const onDefinitionResultRef = useRef(onDefinitionResult);
  const signatureTooltipRef = useRef<HTMLDivElement | null>(null);
  const valueRef = useRef(value);
  const inlayHintsRef = useRef(inlayHints);
  inlayHintsRef.current = inlayHints;
  const semanticTokensRef = useRef(semanticTokens);
  semanticTokensRef.current = semanticTokens;

  onChangeRef.current = onChange;
  onFocusRef.current = onFocus;
  onViewReadyRef.current = onViewReady;
  onHoverRequestRef.current = onHoverRequest;
  onSignatureRequestRef.current = onSignatureRequest;
  onDefinitionRequestRef.current = onDefinitionRequest;
  onDefinitionResultRef.current = onDefinitionResult;
  valueRef.current = value;

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({ effects: setHintsEffect.of(inlayHintsRef.current) });
    view.dispatch({ effects: setTokensEffect.of(semanticTokensRef.current) });
  }, [inlayHints, semanticTokens]);

  const clearSignatureTooltip = () => {
    signatureTooltipRef.current?.remove();
    signatureTooltipRef.current = null;
  };

  useEffect(() => {
    if (!containerRef.current) return;
    const showSignature = (view: EditorView, pos: number) => {
      const request = onSignatureRequestRef.current;
      if (!request) return;
      const docLine = view.state.doc.lineAt(pos);
      void request(docLine.number - 1, pos - docLine.from)
        .then((text) => {
          if (!text || signatureTooltipRef.current) return;
          const coords = view.coordsAtPos(pos);
          if (!coords) return;
          const element = document.createElement('div');
          element.className =
            'fixed z-50 max-w-lg rounded-md border border-neutral-700 bg-neutral-900 px-3 py-2 font-mono text-xs leading-5 text-neutral-200 shadow-xl';
          element.textContent = text;
          element.style.left = `${coords.left}px`;
          element.style.top = `${coords.bottom + 4}px`;
          document.body.appendChild(element);
          signatureTooltipRef.current = element;
          window.setTimeout(() => {
            element.remove();
            if (signatureTooltipRef.current === element) {
              signatureTooltipRef.current = null;
            }
          }, 8000);
        })
        .catch(() => {});
    };
    const updateListener = EditorView.updateListener.of((update: ViewUpdate) => {
      if (update.docChanged) {
        onChangeRef.current(update.state.doc.toString());
        let inserted = '';
        update.changes.iterChanges((_fromA, _toA, _fromB, _toB, text) => {
          inserted += text.toString();
        });
        if (/[,(]/.test(inserted) && onSignatureRequestRef.current) {
          const pos = update.state.selection.main.head;
          window.setTimeout(() => showSignature(update.view, pos), 200);
        } else {
          clearSignatureTooltip();
        }
      }
      if (update.focusChanged && update.view.hasFocus) {
        onFocusRef.current();
      }
    });
    const language = languageDescriptor(codeLanguage);
    const view = new EditorView({
      state: EditorState.create({
        doc: valueRef.current,
        extensions: [
          ...(colorScheme === 'dark' ? [oneDark] : []),
          EditorState.allowMultipleSelections.of(true),
          inlayHintsField,
          semanticTokensField,
          ...(cellType === 'code' && language?.editorAssist
            ? [bracketColorizer(), stickyScroll()]
            : []),
          drawSelection(),
          dropCursor(),
          highlightActiveLine(),
          highlightActiveLineGutter(),
          highlightSpecialChars(),
          highlightSelectionMatches(),
          rectangularSelection(),
          ...(cellType === 'code' && language?.editorAssist
            ? indentationMarkers({
                markerType: 'codeOnly',
                hideFirstIndent: true,
                thickness: 1,
              })
            : []),
          keymap.of([
            ...defaultKeymap,
            ...foldKeymap,
            ...closeBracketsKeymap,
            ...completionKeymap,
            indentWithTab,
          ]),
          closeBrackets(),
          bracketMatching(),
          ...(lineNumbersEnabled
            ? foldingEnabled
              ? [foldGutter(), lineNumbers()]
              : [lineNumbers()]
            : foldingEnabled
              ? [foldGutter()]
              : []),
          indentOnInput(),
          autocompleteEnabled
            ? autocompletion({
                activateOnTyping: true,
                ...(cellType === 'code' && language?.completionSource
                  ? {
                      override: [
                        language.completionSource,
                        ...(extraCompletionSource ? [extraCompletionSource] : []),
                      ],
                    }
                  : {}),
              })
            : [],
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
              backgroundColor: 'transparent',
              color: '#57534e',
              border: 'none',
            },
            '.cm-lineNumbers .cm-gutterElement': {
              padding: '0 8px 0 4px',
              minWidth: '2.5ch',
            },
            '.cm-placeholder': {
              color: '#77736e',
            },
          }),
          ...(colorScheme === 'light'
            ? [
                EditorView.theme({
                  '&': {
                    backgroundColor: '#ffffff',
                    color: '#1c1917',
                    borderColor: '#d6d3d1',
                  },
                  '&.cm-focused': {
                    borderColor: '#c2410c',
                  },
                  '.cm-scroller': {
                    caretColor: '#1c1917',
                  },
                  '.cm-placeholder': {
                    color: '#a8a29e',
                  },
                  '.cm-panels': {
                    backgroundColor: '#ffffff',
                    color: '#1c1917',
                  },
                  '.cm-searchMatch': {
                    backgroundColor: '#fde68a',
                  },
                  '.cm-searchMatch-selected': {
                    backgroundColor: '#f59e0b',
                  },
                }),
              ]
            : []),
          EditorView.domEventHandlers({
            mousedown: (event, view) => {
              if (!(event.ctrlKey || event.metaKey)) return false;
              const request = onDefinitionRequestRef.current;
              const result = onDefinitionResultRef.current;
              if (!request || !result || cellType !== 'code') return false;
              const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
              if (pos === null) return false;
              const docLine = view.state.doc.lineAt(pos);
              event.preventDefault();
              void request(docLine.number - 1, pos - docLine.from)
                .then((target) => {
                  if (target) result(target);
                })
                .catch(() => {});
              return true;
            },
            paste: (event, view) => {
              if (cellType === 'code') {
                const text = event.clipboardData?.getData('text/plain') ?? '';
                const snippet = csvSnippet(text);
                if (snippet) {
                  event.preventDefault();
                  const selection = view.state.selection.main;
                  view.dispatch({
                    changes: { from: selection.from, to: selection.to, insert: snippet },
                    selection: { anchor: selection.from + snippet.length },
                  });
                  return true;
                }
                return false;
              }
              const handler = onImageAttachmentRef.current;
              if (!handler || cellType !== 'markdown') return false;
              const items = event.clipboardData?.items;
              if (!items) return false;
              for (const item of items) {
                if (item.kind !== 'file' || !item.type.startsWith('image/')) continue;
                const file = item.getAsFile();
                if (!file) continue;
                event.preventDefault();
                void readFileAsDataUrl(file)
                  .then((dataUrl) => {
                    const snippet = handler(dataUrl, file.type);
                    if (!snippet) return;
                    const selection = view.state.selection.main;
                    view.dispatch({
                      changes: {
                        from: selection.from,
                        to: selection.to,
                        insert: snippet,
                      },
                      selection: { anchor: selection.from + snippet.length },
                    });
                  })
                  .catch(() => {});
                return true;
              }
              return false;
            },
            drop: (event, view) => {
              const handler = onImageAttachmentRef.current;
              if (!handler || cellType !== 'markdown') return false;
              const file = Array.from(event.dataTransfer?.files ?? []).find((entry) =>
                entry.type.startsWith('image/'),
              );
              if (!file) return false;
              event.preventDefault();
              const pos =
                view.posAtCoords({ x: event.clientX, y: event.clientY }) ??
                view.state.selection.main.head;
              void readFileAsDataUrl(file)
                .then((dataUrl) => {
                  const snippet = handler(dataUrl, file.type);
                  if (!snippet) return;
                  view.dispatch({
                    changes: { from: pos, insert: snippet },
                    selection: { anchor: pos + snippet.length },
                  });
                })
                .catch(() => {});
              return true;
            },
          }),
          EditorView.contentAttributes.of({
            'aria-label': placeholder,
            'data-placeholder': placeholder,
          }),
          search(),
          keymap.of(searchKeymap),
          onHoverRequest
            ? hoverTooltip(async (view, pos) => {
                const request = onHoverRequestRef.current;
                if (!request) return null;
                const docLine = view.state.doc.lineAt(pos);
                const text = await request(docLine.number - 1, pos - docLine.from).catch(
                  () => null,
                );
                if (!text) return null;
                return {
                  pos,
                  end: pos,
                  above: true,
                  create: () => {
                    const dom = document.createElement('div');
                    dom.className =
                      'max-w-md rounded-md border border-neutral-700 bg-neutral-900 px-3 py-2 font-mono text-xs leading-5 text-neutral-200 shadow-xl';
                    dom.textContent = text;
                    return { dom };
                  },
                };
              })
            : [],
          cellType === 'markdown'
            ? markdown()
            : cellType === 'code'
              ? (language?.syntax?.() ?? [])
              : [],
        ],
      }),
      parent: containerRef.current,
    });
    viewRef.current = view;
    view.dispatch({ effects: setHintsEffect.of(inlayHintsRef.current) });
    view.dispatch({ effects: setTokensEffect.of(semanticTokensRef.current) });
    onViewReadyRef.current?.(view);
    return () => {
      clearSignatureTooltip();
      view.destroy();
      viewRef.current = null;
      onViewReadyRef.current?.(null);
    };
  }, [
    cellType,
    codeLanguage,
    placeholder,
    fontSize,
    autocompleteEnabled,
    foldingEnabled,
    colorScheme,
    extraCompletionSource,
  ]);

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
