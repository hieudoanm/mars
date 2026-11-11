import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { NotebookCell } from '../../../types/types';
import type { useNotebookEditor } from '../../../hooks/useNotebookEditor';
import type { NotebookCellListProps } from '../NotebookCellList';
import NotebookCellList from '../NotebookCellList';

type CapturedProps = Record<string, unknown> & { cell: NotebookCell };

const captured: CapturedProps[] = [];

vi.mock('../NotebookCellView', () => ({
  default: (props: CapturedProps) => {
    captured.push(props);
    return (
      <div data-testid={`cellview-${props.cell.id}`}>
        <button type='button' onClick={() => (props.onSelect as () => void)()}>
          select-{props.cell.id}
        </button>
        <button type='button' onClick={() => (props.onEdit as (s: string) => void)('new src')}>
          edit-{props.cell.id}
        </button>
        <button type='button' onClick={() => (props.onExecute as () => void)()}>
          exec-{props.cell.id}
        </button>
        <button type='button' onClick={() => (props.onClear as () => void)()}>
          clear-{props.cell.id}
        </button>
        <button type='button' onClick={() => (props.onDelete as () => void)()}>
          del-{props.cell.id}
        </button>
        <button
          type='button'
          onClick={() =>
            (props.onMoveUp as (() => void) | undefined)?.() ??
            (props.onMoveDown as (() => void) | undefined)?.()
          }>
          move-{props.cell.id}
        </button>
        <button
          type='button'
          onClick={() => (props.onCellClipboard as ((a: string) => void) | undefined)?.('copy')}>
          clip-{props.cell.id}
        </button>
        <button
          type='button'
          onClick={() =>
            (props.lsp as { applyEdits: (e: unknown[]) => void }).applyEdits([
              {
                cellId: props.cell.id,
                startLine: 0,
                startCharacter: 0,
                endLine: 0,
                endCharacter: 0,
                newText: 'patched',
              },
            ])
          }>
          lspedit-{props.cell.id}
        </button>
      </div>
    );
  },
}));

const cellA: NotebookCell = {
  id: 'a',
  cell_type: 'code',
  source: 'let a = 1;',
  execution_count: null,
  outputs: [],
  metadata: {},
};
const cellB: NotebookCell = { ...cellA, id: 'b', source: 'let b = 2;' };
const cells = [cellA, cellB];

const editor = {
  updateCell: vi.fn(),
  setSelectedId: vi.fn(),
  setMode: vi.fn(),
  clearOutputs: vi.fn(),
  splitCellById: vi.fn(),
  updateTags: vi.fn(),
  setCellLanguage: vi.fn(),
  moveCellById: vi.fn(),
  mergeCellById: vi.fn(),
  convertCellType: vi.fn(),
  removeOutput: vi.fn(),
  updateAttachments: vi.fn(),
  duplicateCellById: vi.fn(),
  insertCell: vi.fn(),
} as unknown as ReturnType<typeof useNotebookEditor>;

const makeProps = (overrides: Partial<NotebookCellListProps> = {}): NotebookCellListProps => ({
  cells,
  notebookLanguage: 'rust',
  editor,
  selection: {
    selectedId: 'a',
    rangeIds: new Set(['b']),
    activeCellId: 'a',
    queuedCellIds: new Set(['b']),
    canDelete: true,
    collapsedCellIds: new Set(['b']),
    toggleCellCollapse: vi.fn(),
    mode: 'command',
    hiddenOutputIds: new Set(['a']),
    toggleCellOutputs: vi.fn(),
  },
  runState: { stream: null, runInfo: { a: { status: 'success', durationMs: 5 } } },
  editorPreferences: {
    fontSize: 14,
    autocompleteEnabled: true,
    foldingEnabled: true,
    colorScheme: 'dark',
    lineNumbersEnabled: true,
  },
  lspState: {
    inlayHintsByCell: new Map([['a', [{ cellId: 'a', line: 0, character: 0, label: 'hint' }]]]),
    semanticTokensByCell: new Map(),
    preDiagnostics: { a: [{ from: 0, to: 1, severity: 'error', message: 'e' }] },
    lspDiagnosticsByCell: { a: [{ from: 2, to: 3, severity: 'warning', message: 'w' }] },
    lsp: {
      completion: vi.fn(),
      hover: vi.fn(),
      signatureHelp: vi.fn(),
      definition: vi.fn(),
      references: vi.fn(),
      rename: vi.fn(),
      applyEdits: vi.fn(),
    } as never,
    pendingReveal: { cellId: 'b', line: 1, character: 2 },
    onLspHover: vi.fn(async () => null),
    onNavigateSource: vi.fn(),
  },
  actions: {
    executeCell: vi.fn(),
    setStatus: vi.fn(),
    selectCell: vi.fn(),
    shiftSelectCell: vi.fn(),
    deleteCellOrRange: vi.fn(),
    onRunAllCells: vi.fn(),
    onEditMetadata: vi.fn(),
    onCellClipboard: vi.fn(),
  },
  drag: {
    dropTargetId: 'b',
    onDragCellStart: vi.fn(),
    onDragCellEnd: vi.fn(),
    onCellDragOver: vi.fn(),
    onCellDrop: vi.fn(),
  },
  ...overrides,
});

describe('NotebookCellList', () => {
  beforeEach(() => {
    captured.length = 0;
    vi.clearAllMocks();
  });

  it('renders one view per cell with derived props', () => {
    render(<NotebookCellList {...makeProps()} />);
    expect(captured).toHaveLength(2);
    const [a, b] = captured;
    expect(a.selected).toBe(true); // selectedId match
    expect(b.selected).toBe(true); // rangeIds match
    expect(a.isExecuting).toBe(true); // activeCellId
    expect(b.isQueued).toBe(true);
    expect(b.collapsed).toBe(true);
    expect(a.outputsHidden).toBe(true);
    expect(b.outputsHidden).toBe(false);
    expect(b.isDropTarget).toBe(true);
    expect(a.runInfo).toEqual({ status: 'success', durationMs: 5 });
    expect(b.runInfo).toBeNull();
    expect(a.inlayHints).toHaveLength(1);
    expect(a.liveDiagnostics).toHaveLength(2); // pre + lsp merged
    expect(b.revealRequest).toEqual({ cellId: 'b', line: 1, character: 2 });
    expect(a.revealRequest).toBeNull();
  });

  it('wires selection, execution, and editing callbacks', () => {
    const props = makeProps();
    render(<NotebookCellList {...props} />);
    fireEvent.click(screen.getByText('select-a'));
    expect(props.actions.selectCell).toHaveBeenCalledWith(cellA);
    fireEvent.click(screen.getByText('exec-b'));
    expect(editor.setSelectedId).toHaveBeenCalledWith('b');
    expect(props.actions.executeCell).toHaveBeenCalledWith(cellB);
    fireEvent.click(screen.getByText('edit-a'));
    expect(editor.updateCell).toHaveBeenCalledWith('a', 'new src');
    fireEvent.click(screen.getByText('clear-a'));
    expect(editor.clearOutputs).toHaveBeenCalledWith('a');
    fireEvent.click(screen.getByText('del-b'));
    expect(props.actions.deleteCellOrRange).toHaveBeenCalledWith(cellB);
    fireEvent.click(screen.getByText('clip-a'));
    expect(props.actions.onCellClipboard).toHaveBeenCalledWith(cellA, 'copy');
    fireEvent.click(screen.getByText('move-a'));
    expect(editor.moveCellById).toHaveBeenCalledWith('a', -1);
  });

  it('applies grouped LSP edits through the editor', () => {
    render(<NotebookCellList {...makeProps()} />);
    fireEvent.click(screen.getByText('lspedit-a'));
    expect(editor.updateCell).toHaveBeenCalledWith('a', expect.stringContaining('patched'));
    // unknown cell ids are skipped without throwing
    const view = captured[0];
    (view.lsp as { applyEdits: (e: unknown[]) => void }).applyEdits([
      {
        cellId: 'missing',
        startLine: 0,
        startCharacter: 0,
        endLine: 0,
        endCharacter: 0,
        newText: 'x',
      },
    ]);
    expect(editor.updateCell).toHaveBeenCalledTimes(1);
  });

  it('omits merge for the last cell and clipboard/metadata when not provided', () => {
    const props = makeProps();
    props.actions.onEditMetadata = undefined;
    props.actions.onCellClipboard = undefined;
    render(<NotebookCellList {...props} />);
    const last = captured[1];
    expect(last.onMergeNext).toBeUndefined();
    expect(last.onEditMetadata).toBeUndefined();
    expect(last.onCellClipboard).toBeUndefined();
    expect(captured[0].onMergeNext).toBeTypeOf('function');
  });
});
