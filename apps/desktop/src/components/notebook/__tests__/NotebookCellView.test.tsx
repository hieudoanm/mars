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
  }: {
    selected?: boolean;
    isExecuting?: boolean;
    canDelete?: boolean;
    collapsed?: boolean;
  } = {},
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
      onSelect={onSelect}
      onEdit={onEdit}
      onFocus={onFocus}
      onExecute={onExecute}
      onClear={onClear}
      onDelete={onDelete}
      onToggleCollapse={onToggleCollapse}
      onStatus={onStatus}
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
});
