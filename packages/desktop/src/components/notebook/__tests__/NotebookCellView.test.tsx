import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { NotebookCell } from '../../../types/types';

const { onSelect, onEdit, onFocus, onExecute, onClear, onDelete, onToggleCollapse, onStatus } =
  vi.hoisted(() => ({
    onSelect: vi.fn(),
    onEdit: vi.fn(),
    onFocus: vi.fn(),
    onExecute: vi.fn(),
    onClear: vi.fn(),
    onDelete: vi.fn(),
    onToggleCollapse: vi.fn(),
    onStatus: vi.fn(),
  }));

import NotebookCellView from '../NotebookCellView';

const markdownCell: NotebookCell = {
  id: 'cell-md',
  cell_type: 'markdown',
  source: '# Welcome',
  execution_count: null,
  outputs: [],
  metadata: {},
};

const codeCell: NotebookCell = {
  id: 'cell-code',
  cell_type: 'code',
  source: 'let x = 42;',
  execution_count: 1,
  outputs: [{ type: 'text', text: 'hello world' }],
  metadata: {},
};

const errorCell: NotebookCell = {
  id: 'cell-error',
  cell_type: 'code',
  source: 'panic!();',
  execution_count: 2,
  outputs: [{ type: 'error', name: 'PanicError', message: 'panic occurred', traceback: [] }],
  metadata: {},
};

const renderCell = (
  cell: NotebookCell,
  {
    selected = false,
    isExecuting = false,
    canDelete = true,
    collapsed = false,
    mode = 'command' as const,
    outputsHidden = false,
    ...extra
  }: {
    selected?: boolean;
    isExecuting?: boolean;
    canDelete?: boolean;
    collapsed?: boolean;
    mode?: 'edit' | 'command';
    outputsHidden?: boolean;
  } & Partial<Parameters<typeof NotebookCellView>[0]> = {},
) =>
  render(
    <NotebookCellView
      cell={cell}
      selected={selected}
      isExecuting={isExecuting}
      fontSize={14}
      autocompleteEnabled={true}
      foldingEnabled={true}
      canDelete={canDelete}
      collapsed={collapsed}
      mode={mode}
      outputsHidden={outputsHidden}
      onSelect={onSelect}
      onEdit={onEdit}
      onFocus={onFocus}
      onExecute={onExecute}
      onClear={onClear}
      onDelete={onDelete}
      onToggleCollapse={onToggleCollapse}
      onStatus={onStatus}
      {...extra}
    />,
  );

describe('NotebookCellView', () => {
  beforeEach(() => {
    onSelect.mockReset();
    onEdit.mockReset();
    onFocus.mockReset();
    onExecute.mockReset();
    onClear.mockReset();
    onDelete.mockReset();
    onToggleCollapse.mockReset();
    onStatus.mockReset();
  });

  it('renders markdown cell with type label', () => {
    renderCell(markdownCell);
    expect(screen.getByLabelText('Cell type')).toHaveValue('markdown');
    expect(screen.queryByRole('button', { name: /Run cell/ })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete cell (D, D)' })).toBeInTheDocument();
  });

  it('does not render outputs on non-code cells', () => {
    renderCell({
      ...markdownCell,
      outputs: [{ type: 'text', text: 'should not appear' }],
    });
    expect(screen.queryByText('should not appear')).not.toBeInTheDocument();
  });

  it('renders code cell with run button and outputs', async () => {
    renderCell(codeCell);
    expect(screen.getByLabelText('Cell type')).toHaveValue('code');
    expect(screen.getByText('In [1]')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Run cell/ })).toBeInTheDocument();
    expect(await screen.findByText('hello world', {}, { timeout: 5000 })).toBeInTheDocument();
  });

  it('renders error cell output', async () => {
    renderCell(errorCell);
    expect(await screen.findByText(/PanicError/, {}, { timeout: 5000 })).toBeInTheDocument();
    expect(screen.getByText(/panic occurred/)).toBeInTheDocument();
  });

  it('shows active state when selected', () => {
    const { container } = renderCell(markdownCell, { selected: true });
    const article = container.querySelector('article');
    expect(article?.className).toContain('border-orange-800');
  });

  it('shows a command-mode accent on the selected cell', () => {
    const { container } = renderCell(codeCell, { selected: true, mode: 'command' });
    expect(container.querySelector('article')?.className).toContain('#f97316');
  });

  it('shows an edit-mode accent on the selected cell', () => {
    const { container } = renderCell(codeCell, { selected: true, mode: 'edit' });
    expect(container.querySelector('article')?.className).toContain('#34d399');
  });

  it('hides outputs when outputsHidden is set', async () => {
    renderCell(codeCell, { outputsHidden: true });
    expect(screen.queryByText('hello world')).not.toBeInTheDocument();
  });

  it('marks the running cell with a distinct border', () => {
    const { container } = renderCell(codeCell, { isExecuting: true });
    const article = container.querySelector('article');
    expect(article).toHaveAttribute('aria-busy', 'true');
    expect(article?.className).toContain('border-amber-400');
  });

  it('renders icon-only cell actions with accessible labels', () => {
    renderCell(codeCell);
    const runButton = screen.getByRole('button', { name: /Run cell/ });
    const clearButton = screen.getByRole('button', { name: 'Clear outputs' });
    const deleteButton = screen.getByRole('button', { name: 'Delete cell (D, D)' });

    for (const button of [runButton, clearButton, deleteButton]) {
      expect(button.textContent).toBe('');
      expect(button.querySelector('svg')).toBeInTheDocument();
    }
  });

  it('calls onSelect when article is clicked', () => {
    renderCell(markdownCell);
    fireEvent.click(screen.getByRole('article'));
    expect(onSelect).toHaveBeenCalledOnce();
  });

  it('calls onExecute when Run cell is clicked without triggering onSelect', () => {
    renderCell(codeCell);
    fireEvent.click(screen.getByRole('button', { name: /Run cell/ }));
    expect(onExecute).toHaveBeenCalledOnce();
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('calls onClear and onStatus when Clear outputs is clicked', () => {
    renderCell(codeCell);
    fireEvent.click(screen.getByRole('button', { name: 'Clear outputs' }));
    expect(onClear).toHaveBeenCalledOnce();
    expect(onStatus).toHaveBeenCalledWith('Outputs cleared');
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('calls onDelete when Delete cell is clicked without triggering onSelect', () => {
    renderCell(markdownCell);
    fireEvent.click(screen.getByRole('button', { name: 'Delete cell (D, D)' }));
    expect(onDelete).toHaveBeenCalledOnce();
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('disables delete when the cell cannot be removed', () => {
    renderCell(markdownCell, { canDelete: false });
    expect(screen.getByRole('button', { name: 'Delete cell (D, D)' })).toBeDisabled();
  });

  it('does not show Clear button when there are no outputs', () => {
    renderCell({ ...codeCell, outputs: [] });
    expect(screen.queryByRole('button', { name: 'Clear outputs' })).not.toBeInTheDocument();
  });

  it('requests collapse toggle from the header button', () => {
    renderCell(codeCell);
    fireEvent.click(screen.getByRole('button', { name: 'Collapse cell' }));
    expect(onToggleCollapse).toHaveBeenCalledOnce();
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('shows a source summary and hides content while collapsed', () => {
    const { container } = renderCell(codeCell, { collapsed: true });

    const summary = container.querySelector('[data-component="CollapsedCellSummary"]');
    expect(summary).toBeInTheDocument();
    expect(summary).toHaveTextContent('let x = 42;');
    expect(summary?.nextElementSibling?.className).toContain('hidden');
    expect(screen.getByRole('button', { name: 'Expand cell' })).toBeInTheDocument();

    fireEvent.click(summary as HTMLElement);
    expect(onToggleCollapse).toHaveBeenCalledOnce();
  });

  it('renders placeholder based on cell type', async () => {
    renderCell(markdownCell);
    expect(await screen.findByLabelText('Write Markdown...')).toBeInTheDocument();
    renderCell(codeCell);
    expect(await screen.findByLabelText('Write Rust code...')).toBeInTheDocument();
  });

  it('opens the context menu on right-click and runs menu actions', async () => {
    const onShiftSelect = vi.fn();
    const onInsertAbove = vi.fn();
    const onInsertBelow = vi.fn();
    const onDuplicate = vi.fn();
    const onRunAll = vi.fn();
    const onChangeType = vi.fn();
    const onEditMetadata = vi.fn();
    const onCellClipboard = vi.fn();
    const onToggleOutputs = vi.fn();
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText: vi.fn(async () => {}) },
      configurable: true,
    });
    renderCell(codeCell, {
      selected: true,
      onShiftSelect,
      onInsertAbove,
      onInsertBelow,
      onDuplicate,
      onRunAll,
      onChangeType,
      onEditMetadata,
      onCellClipboard,
      onToggleOutputs,
    });

    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    expect(onSelect).toHaveBeenCalled();

    fireEvent.click(await screen.findByText('Copy cell source'));
    expect(onStatus).toHaveBeenCalledWith('Cell source copied');

    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Insert cell above'));
    expect(onInsertAbove).toHaveBeenCalled();
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Insert cell below'));
    expect(onInsertBelow).toHaveBeenCalled();
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Duplicate cell'));
    expect(onDuplicate).toHaveBeenCalled();
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Run all cells'));
    expect(onRunAll).toHaveBeenCalled();
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Paste cell below'));
    expect(onCellClipboard).toHaveBeenCalledWith('paste-below');
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Hide outputs'));
    expect(onToggleOutputs).toHaveBeenCalled();
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Convert to markdown'));
    expect(onChangeType).toHaveBeenCalledWith('markdown');
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Edit metadata'));
    expect(onEditMetadata).toHaveBeenCalled();
    fireEvent.contextMenu(screen.getByRole('article'), { clientX: 10, clientY: 20 });
    fireEvent.click(await screen.findByText('Delete cell'));
    expect(onDelete).toHaveBeenCalled();
    void onShiftSelect;
  });

  it('calls onShiftSelect on shift-click', () => {
    const onShiftSelect = vi.fn();
    renderCell(markdownCell, { onShiftSelect });
    fireEvent.click(screen.getByRole('article'), { shiftKey: true });
    expect(onShiftSelect).toHaveBeenCalledOnce();
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('handles drag over and drop', () => {
    const onCellDragOver = vi.fn();
    const onCellDrop = vi.fn();
    renderCell(codeCell, { onCellDragOver, onCellDrop });
    const article = screen.getByRole('article');
    fireEvent.dragOver(article);
    expect(onCellDragOver).toHaveBeenCalledWith('cell-code');
    fireEvent.drop(article);
    expect(onCellDrop).toHaveBeenCalledWith('cell-code');
  });

  it('renders streaming stdout and stderr', () => {
    renderCell(codeCell, {
      stream: { cellId: 'cell-code', stdout: 'out text', stderr: 'err text' } as never,
    });
    expect(screen.getByText('out text')).toBeInTheDocument();
    expect(screen.getByText('err text')).toBeInTheDocument();
  });

  it('toggles the markdown preview from the header', async () => {
    renderCell(markdownCell);
    const preview = await screen.findByRole('heading', { name: 'Welcome' });
    expect(preview).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Hide Markdown preview' }));
    await vi.waitFor(() =>
      expect(screen.queryByRole('button', { name: 'Hide Markdown preview' })).toBeNull(),
    );
    expect(screen.getByRole('button', { name: 'Show Markdown preview' })).toBeInTheDocument();
  });

  it('loads and applies quick fixes when diagnostics exist', async () => {
    const applyEdits = vi.fn();
    const lsp = {
      completion: vi.fn(async () => null),
      signature: vi.fn(async () => null),
      definition: vi.fn(async () => null),
      onDefinitionResult: vi.fn(),
      codeActions: vi.fn(async () => [
        {
          title: 'Add semicolon',
          edits: [
            {
              cellId: 'cell-code',
              startLine: 0,
              startCharacter: 0,
              endLine: 0,
              endCharacter: 0,
              newText: ';',
            },
          ],
        },
      ]),
      applyEdits,
    };
    renderCell(
      { ...codeCell, outputs: [] },
      {
        liveDiagnostics: [{ from: 0, to: 3, severity: 'error', message: 'oops' }],
        lsp,
      },
    );
    fireEvent.click(await screen.findByRole('button', { name: 'Quick fixes' }));
    fireEvent.click(await screen.findByText('Add semicolon'));
    expect(applyEdits).toHaveBeenCalled();
    expect(onStatus).toHaveBeenCalledWith('Applied: Add semicolon');
  });
});
