import { lazy, Suspense, useEffect, useMemo, useRef, useState, type FC } from 'react';
import { EditorView } from '@codemirror/view';
import type { CellType, NotebookCell } from '../../types/types';
import type { KernelStream } from '../../hooks/useKernelExecution';
import type { Diagnostic } from '@codemirror/lint';
import { diagnosticsForCell } from '../../lib/diagnostics';
import { cellLanguage as languageOf } from '../../lib/cellLanguage';
import { languageDescriptor } from '../../lib/languages';
import { createLspCompletionSource } from '../../lib/lspCompletions';
import { offsetToLineChar } from '../../lib/cellTextEdit';
import { getCellAttachments } from '../../lib/cellAttachments';
import type {
  LspCellTextEdit,
  LspCodeAction,
  LspCompletionItem,
  LspDefinitionTarget,
} from '../../lib/kernelService';
import NotebookCellHeader from './NotebookCellHeader';
import { CellComments } from './CellComments';
import { getCellComments } from '../../lib/cellComments';
import { DataPreview } from './DataPreview';
import CellContextMenu, { type ContextMenuItem } from './CellContextMenu';
import LazyMount from '../shared/LazyMount';

const CodeEditor = lazy(() => import('../editor/CodeEditor'));
const CellOutput = lazy(() => import('../output/CellOutput'));
const MarkdownPreview = lazy(() => import('../output/MarkdownPreview'));

export type NotebookCellViewProps = {
  cell: NotebookCell;
  notebookLanguage?: string;
  selected: boolean;
  isExecuting: boolean;
  runInfo?: { status: 'success' | 'error' | 'interrupted'; durationMs: number } | null;
  isQueued?: boolean;
  inlayHints?: import('../../lib/inlayHints').InlayHint[];
  semanticTokens?: import('../../lib/kernelService').LspSemanticToken[];
  fontSize: number;
  autocompleteEnabled: boolean;
  foldingEnabled: boolean;
  colorScheme?: 'dark' | 'light';
  canDelete: boolean;
  collapsed: boolean;
  mode?: 'edit' | 'command';
  outputsHidden?: boolean;
  lineNumbersEnabled?: boolean;
  onToggleOutputs?: () => void;
  onCellClipboard?: (action: 'cut' | 'copy' | 'paste-below' | 'paste-above') => void;
  stream?: KernelStream | null;
  isDropTarget?: boolean;
  onSelect: () => void;
  onEdit: (source: string) => void;
  onFocus: () => void;
  onExecute: () => void;
  onClear: () => void;
  onDelete: () => void;
  onToggleCollapse: () => void;
  onMoveUp?: () => void;
  onMoveDown?: () => void;
  onMergeNext?: () => void;
  onSplitCell?: (offset: number) => void;
  onUpdateTags?: (tags: string[]) => void;
  onUpdateComments?: (comments: import('../../lib/cellComments').CellComment[]) => void;
  onEditMetadata?: () => void;
  onChangeLanguage?: (language: string) => void;
  onShiftSelect?: () => void;
  onChangeType?: (cellType: CellType) => void;
  onRemoveOutput?: (outputIndex: number) => void;
  onNavigateSource?: (cellHash: number, line: number) => void;
  onAddAttachment?: (dataUrl: string, mimeType: string) => string | null;
  onDuplicate?: () => void;
  onInsertAbove?: () => void;
  onInsertBelow?: () => void;
  onRunAll?: () => void;
  onRunFromHere?: () => void;
  onStatus: (status: string) => void;
  onDragCellStart?: (id: string) => void;
  onDragCellEnd?: () => void;
  onCellDragOver?: (id: string) => void;
  onCellDrop?: (id: string) => void;
  liveDiagnostics?: Diagnostic[];
  onLspHover?: (line: number, character: number) => Promise<string | null>;
  lsp?: {
    completion: (
      cellId: string,
      line: number,
      character: number,
    ) => Promise<{ items?: LspCompletionItem[] } | LspCompletionItem[] | null>;
    signature: (cellId: string, line: number, character: number) => Promise<string | null>;
    definition: (
      cellId: string,
      line: number,
      character: number,
    ) => Promise<LspDefinitionTarget | null>;
    onDefinitionResult: (target: LspDefinitionTarget) => void;
    codeActions?: (cellId: string, line: number, character: number) => Promise<LspCodeAction[]>;
    applyEdits?: (edits: LspCellTextEdit[]) => void;
  };
  revealRequest?: { line: number; character: number } | null;
};

const editorFallback = (
  <div className='min-h-[44px] rounded-md border border-neutral-800 bg-neutral-950' />
);

const NotebookCellView: FC<NotebookCellViewProps> = ({
  cell,
  notebookLanguage = 'rust',
  selected,
  isExecuting,
  runInfo = null,
  isQueued = false,
  inlayHints = [],
  semanticTokens = [],
  fontSize,
  autocompleteEnabled,
  foldingEnabled,
  colorScheme = 'dark',
  canDelete,
  collapsed,
  mode = 'command',
  outputsHidden = false,
  lineNumbersEnabled = true,
  onToggleOutputs,
  onCellClipboard,
  stream = null,
  isDropTarget = false,
  onSelect,
  onEdit,
  onFocus,
  onExecute,
  onClear,
  onDelete,
  onToggleCollapse,
  onMoveUp,
  onMoveDown,
  onMergeNext,
  onSplitCell,
  onUpdateTags,
  onUpdateComments,
  onEditMetadata,
  onChangeLanguage,
  onShiftSelect,
  onChangeType,
  onRemoveOutput,
  onNavigateSource,
  onAddAttachment,
  onDuplicate,
  onInsertAbove,
  onInsertBelow,
  onRunAll,
  onRunFromHere,
  onStatus,
  onDragCellStart,
  onDragCellEnd,
  onCellDragOver,
  onCellDrop,
  liveDiagnostics,
  onLspHover,
  lsp,
  revealRequest = null,
}) => {
  const cellLanguage = languageOf(cell, notebookLanguage);
  const language = languageDescriptor(cellLanguage);
  const editorViewRef = useRef<EditorView | null>(null);
  const [showMarkdownPreview, setShowMarkdownPreview] = useState(true);
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);
  const [quickFixes, setQuickFixes] = useState<LspCodeAction[] | null>(null);
  const [quickFixesLoading, setQuickFixesLoading] = useState(false);

  const cellDiagnostics = [...diagnosticsForCell(cell), ...(liveDiagnostics ?? [])];
  const canQuickFix =
    cell.cell_type === 'code' &&
    language?.lsp === true &&
    lsp?.codeActions !== undefined &&
    cellDiagnostics.length > 0;

  const loadQuickFixes = async () => {
    if (!lsp?.codeActions) return;
    const first = cellDiagnostics[0];
    const { line, character } = offsetToLineChar(cell.source, first?.from ?? 0);
    setQuickFixesLoading(true);
    try {
      setQuickFixes(await lsp.codeActions(cell.id, line, character));
    } finally {
      setQuickFixesLoading(false);
    }
  };
  const completionSource = useMemo(
    () =>
      lsp && cell.cell_type === 'code' && language?.lsp === true
        ? createLspCompletionSource((line, character) => lsp.completion(cell.id, line, character))
        : undefined,
    [lsp, cell.id, cell.cell_type, cellLanguage],
  );

  useEffect(() => {
    if (!revealRequest) return;
    const view = editorViewRef.current;
    if (!view) return;
    const line = view.state.doc.line(Math.min(revealRequest.line + 1, view.state.doc.lines));
    const pos = Math.min(line.from + revealRequest.character, line.to);
    view.dispatch({
      selection: { anchor: pos },
      effects: EditorView.scrollIntoView(pos, { y: 'center' }),
    });
    view.focus();
  }, [revealRequest]);
  const firstSourceLine =
    cell.source
      .split('\n')
      .find((line) => line.trim().length > 0)
      ?.trim() ?? '';
  const openSearch = () => {
    const view = editorViewRef.current;
    if (!view) return;
    view.focus();
    void import('@codemirror/search').then((module) => module.openSearchPanel(view));
  };
  const buildMenuItems = (): ContextMenuItem[] => {
    const items: ContextMenuItem[] = [];
    if (cell.cell_type === 'code') {
      items.push({ label: 'Run cell', onSelect: onExecute });
      if (onRunFromHere) {
        items.push({ label: 'Run from this cell', onSelect: onRunFromHere });
      }
    }
    if (onRunAll) {
      items.push({ label: 'Run all cells', onSelect: onRunAll });
    }
    items.push({
      label: 'Insert cell above',
      onSelect: () => onInsertAbove?.(),
      disabled: !onInsertAbove,
      separatorBefore: true,
    });
    items.push({
      label: 'Insert cell below',
      onSelect: () => onInsertBelow?.(),
      disabled: !onInsertBelow,
    });
    if (onDuplicate) {
      items.push({ label: 'Duplicate cell', onSelect: onDuplicate });
    }
    items.push({
      label: 'Copy cell source',
      onSelect: () => {
        void navigator.clipboard.writeText(cell.source).catch(() => {});
        onStatus('Cell source copied');
      },
      separatorBefore: true,
    });
    if (onCellClipboard) {
      items.push({
        label: 'Cut cell',
        onSelect: () => onCellClipboard('cut'),
        separatorBefore: true,
      });
      items.push({ label: 'Copy cell', onSelect: () => onCellClipboard('copy') });
      items.push({ label: 'Paste cell below', onSelect: () => onCellClipboard('paste-below') });
      items.push({ label: 'Paste cell above', onSelect: () => onCellClipboard('paste-above') });
    }
    if (onToggleOutputs && cell.outputs.length > 0) {
      items.push({
        label: outputsHidden ? 'Show outputs' : 'Hide outputs',
        onSelect: onToggleOutputs,
      });
    }
    if (onMoveUp) items.push({ label: 'Move cell up', onSelect: onMoveUp });
    if (onMoveDown) items.push({ label: 'Move cell down', onSelect: onMoveDown });
    if (onMergeNext) {
      items.push({ label: 'Merge with next cell', onSelect: onMergeNext });
    }
    if (cell.cell_type === 'code' && onClear) {
      items.push({ label: 'Clear outputs', onSelect: onClear });
    }
    if (onEditMetadata) {
      items.push({ label: 'Edit metadata', onSelect: onEditMetadata });
    }
    const otherTypes = (['markdown', 'code', 'raw'] as CellType[]).filter(
      (type) => type !== cell.cell_type,
    );
    for (const type of otherTypes) {
      items.push({
        label: `Convert to ${type === 'code' ? 'code' : type}`,
        onSelect: () => onChangeType?.(type),
        disabled: !onChangeType,
      });
    }
    items.push({
      label: 'Delete cell',
      onSelect: onDelete,
      disabled: !canDelete,
      danger: true,
      separatorBefore: true,
    });
    return items;
  };

  return (
    <article
      data-component='NotebookCellView'
      id={`cell-${cell.id}`}
      aria-busy={isExecuting || undefined}
      className={`flex scroll-mt-6 rounded-xl border [contain-intrinsic-size:auto_14rem] [content-visibility:auto] ${isExecuting ? 'border-amber-400 ring-2 ring-amber-500/40' : isDropTarget ? 'border-orange-500 ring-1 ring-orange-500/60' : selected ? 'border-orange-800 ring-1 ring-orange-950' : 'border-transparent'} ${selected && !isExecuting && !isDropTarget ? (mode === 'edit' ? 'shadow-[inset_3px_0_0_0_#34d399]' : 'shadow-[inset_3px_0_0_0_#f97316]') : ''} bg-neutral-900`}
      onClick={(event) => {
        if (event.shiftKey && onShiftSelect) onShiftSelect();
        else onSelect();
      }}
      onDragOver={(event) => {
        if (!onCellDragOver) return;
        event.preventDefault();
        onCellDragOver(cell.id);
      }}
      onDrop={(event) => {
        event.preventDefault();
        onCellDrop?.(cell.id);
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        onSelect();
        setMenuPos({ x: event.clientX, y: event.clientY });
      }}>
      <div className='w-12 p-1.5 pt-4 text-right text-xs text-neutral-500'>
        <span>{cell.cell_type === 'code' ? `In [${cell.execution_count ?? ' '}]` : ' '}</span>
      </div>
      <div className='min-w-0 flex-1 py-3 pr-4'>
        <NotebookCellHeader
          cellId={cell.id}
          cellType={cell.cell_type}
          cellLanguage={cellLanguage}
          isExecuting={isExecuting}
          runInfo={runInfo}
          isQueued={isQueued}
          collapsed={collapsed}
          canDelete={canDelete}
          hasOutputs={cell.outputs.length > 0}
          markdownPreviewVisible={showMarkdownPreview}
          onChangeType={onChangeType}
          onMoveUp={onMoveUp}
          onMoveDown={onMoveDown}
          onMergeNext={onMergeNext}
          onSplitCell={
            onSplitCell
              ? () => onSplitCell(editorViewRef.current?.state.selection.main.head ?? 0)
              : undefined
          }
          tags={Array.isArray(cell.metadata?.tags) ? (cell.metadata.tags as string[]) : []}
          onUpdateTags={onUpdateTags}
          onChangeLanguage={onChangeLanguage}
          onToggleCollapse={onToggleCollapse}
          onSearch={openSearch}
          onToggleMarkdownPreview={() => setShowMarkdownPreview((current) => !current)}
          onExecute={onExecute}
          onClear={onClear}
          onDelete={onDelete}
          onStatus={onStatus}
          onDragCellStart={onDragCellStart}
          onDragCellEnd={onDragCellEnd}
        />
        {collapsed ? (
          <button
            type='button'
            data-component='CollapsedCellSummary'
            className='w-full truncate rounded-md px-2 py-1.5 text-left font-mono text-xs text-neutral-500 hover:bg-neutral-800/60 hover:text-neutral-300'
            onClick={(event) => {
              event.stopPropagation();
              onToggleCollapse();
            }}>
            {firstSourceLine || 'Empty cell'}
          </button>
        ) : null}
        <div className={collapsed ? 'hidden' : undefined}>
          <LazyMount>
            <Suspense fallback={editorFallback}>
              <CodeEditor
                value={cell.source}
                onChange={onEdit}
                onFocus={onFocus}
                placeholder={
                  cell.cell_type === 'markdown'
                    ? 'Write Markdown...'
                    : cell.cell_type === 'raw'
                      ? 'Write raw content...'
                      : `Write ${language?.label ?? cellLanguage} code...`
                }
                cellType={cell.cell_type}
                codeLanguage={cellLanguage}
                fontSize={fontSize}
                autocompleteEnabled={autocompleteEnabled}
                foldingEnabled={foldingEnabled}
                lineNumbersEnabled={lineNumbersEnabled}
                colorScheme={colorScheme}
                inlayHints={inlayHints}
                semanticTokens={semanticTokens}
                diagnostics={[...diagnosticsForCell(cell), ...(liveDiagnostics ?? [])]}
                onHoverRequest={
                  cell.cell_type === 'code' && language?.lsp === true && onLspHover
                    ? onLspHover
                    : undefined
                }
                extraCompletionSource={completionSource}
                onSignatureRequest={
                  cell.cell_type === 'code' && language?.lsp === true && lsp
                    ? (line, character) => lsp.signature(cell.id, line, character)
                    : undefined
                }
                onDefinitionRequest={
                  cell.cell_type === 'code' && language?.lsp === true && lsp
                    ? (line, character) => lsp.definition(cell.id, line, character)
                    : undefined
                }
                onDefinitionResult={language?.lsp === true ? lsp?.onDefinitionResult : undefined}
                onImageAttachment={
                  cell.cell_type === 'markdown' && onAddAttachment ? onAddAttachment : undefined
                }
                onViewReady={(view) => {
                  editorViewRef.current = view;
                }}
              />
            </Suspense>
          </LazyMount>
          {cell.cell_type === 'raw' ? (
            <p className='mt-2 text-xs text-neutral-500'>
              Raw cells are preserved and never executed.
            </p>
          ) : null}
          {cell.cell_type === 'markdown' && showMarkdownPreview ? (
            <Suspense fallback={null}>
              <MarkdownPreview
                source={cell.source}
                attachments={getCellAttachments(cell.metadata)}
              />
            </Suspense>
          ) : null}
          {stream && (stream.stdout || stream.stderr) ? (
            <div
              data-component='CellStreamOutput'
              className='mt-2 rounded-md border border-neutral-800 bg-neutral-950 p-3 font-mono text-xs'>
              {stream.stdout ? (
                <pre className='whitespace-pre-wrap text-neutral-300'>{stream.stdout}</pre>
              ) : null}
              {stream.stderr ? (
                <pre className='whitespace-pre-wrap text-red-400'>{stream.stderr}</pre>
              ) : null}
            </div>
          ) : null}
          {canQuickFix ? (
            <div className='relative mt-1 flex justify-end'>
              <button
                type='button'
                disabled={quickFixesLoading}
                className='flex items-center gap-1 rounded-md border border-neutral-700 px-2 py-0.5 text-[10px] text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200 disabled:opacity-50'
                onClick={() => void loadQuickFixes()}>
                {quickFixesLoading ? 'Checking…' : 'Quick fixes'}
              </button>
              {quickFixes !== null ? (
                <div className='absolute top-full right-0 z-20 mt-1 w-64 rounded-md border border-neutral-700 bg-neutral-900 py-1 shadow-xl'>
                  {quickFixes.length === 0 ? (
                    <p className='px-3 py-1.5 text-xs text-neutral-500'>No quick fixes available</p>
                  ) : (
                    quickFixes.map((action, index) => (
                      <button
                        key={`${action.title}-${index}`}
                        type='button'
                        className='block w-full px-3 py-1.5 text-left text-xs text-neutral-300 hover:bg-neutral-800'
                        onClick={() => {
                          if (action.edits.length > 0) lsp?.applyEdits?.(action.edits);
                          onStatus(`Applied: ${action.title}`);
                          setQuickFixes(null);
                        }}>
                        {action.title}
                      </button>
                    ))
                  )}
                </div>
              ) : null}
            </div>
          ) : null}
          {cell.cell_type === 'code' && !outputsHidden
            ? cell.outputs.map((output, outputIndex) => (
                <Suspense key={`${cell.id}-${outputIndex}`} fallback={null}>
                  <CellOutput
                    output={output}
                    onRemove={onRemoveOutput ? () => onRemoveOutput(outputIndex) : undefined}
                    onNavigateSource={onNavigateSource}
                  />
                </Suspense>
              ))
            : null}
          {onUpdateComments ? (
            <CellComments comments={getCellComments(cell.metadata)} onChange={onUpdateComments} />
          ) : null}
          {typeof cell.metadata?.preview === 'string' && cell.metadata.preview ? (
            <DataPreview path={cell.metadata.preview} />
          ) : null}
        </div>
      </div>

      {menuPos ? (
        <CellContextMenu
          x={menuPos.x}
          y={menuPos.y}
          onClose={() => setMenuPos(null)}
          items={buildMenuItems()}
        />
      ) : null}
    </article>
  );
};
NotebookCellView.displayName = 'NotebookCellView';
export default NotebookCellView;
