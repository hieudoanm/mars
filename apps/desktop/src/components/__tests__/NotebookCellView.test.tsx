import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { NotebookCell } from '../../lib/types';

const {
  onSelect,
  onEdit,
  onFocus,
  onExecute,
  onClear,
  onStatus,
} = vi.hoisted(() => ({
  onSelect: vi.fn(),
  onEdit: vi.fn(),
  onFocus: vi.fn(),
  onExecute: vi.fn(),
  onClear: vi.fn(),
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

describe('NotebookCellView', () => {
  beforeEach(() => {
    onSelect.mockReset();
    onEdit.mockReset();
    onFocus.mockReset();
    onExecute.mockReset();
    onClear.mockReset();
    onStatus.mockReset();
  });

  it('renders markdown cell with type label', () => {
    render(
      <NotebookCellView
        cell={markdownCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(screen.getByText('markdown')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Run cell/ })).not.toBeInTheDocument();
  });

  it('renders code cell with run button and outputs', () => {
    render(
      <NotebookCellView
        cell={codeCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(screen.getByText('code')).toBeInTheDocument();
    expect(screen.getByText('In [1]')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Run cell/ })).toBeInTheDocument();
    expect(screen.getByText('hello world')).toBeInTheDocument();
  });

  it('renders error cell output', () => {
    render(
      <NotebookCellView
        cell={errorCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(screen.getByText(/PanicError/)).toBeInTheDocument();
    expect(screen.getByText(/panic occurred/)).toBeInTheDocument();
  });

  it('shows active class when selected', () => {
    const { container } = render(
      <NotebookCellView
        cell={markdownCell}
        selected
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(container.querySelector('.active')).not.toBeNull();
  });

  it('calls onSelect when article is clicked', () => {
    render(
      <NotebookCellView
        cell={markdownCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    fireEvent.click(screen.getByRole('article'));
    expect(onSelect).toHaveBeenCalledOnce();
  });

  it('calls onExecute when Run cell is clicked without triggering onSelect', () => {
    render(
      <NotebookCellView
        cell={codeCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    const runButton = screen.getByRole('button', { name: /Run cell/ });
    fireEvent.click(runButton);
    expect(onExecute).toHaveBeenCalledOnce();
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('calls onClear and onStatus when Clear is clicked', () => {
    render(
      <NotebookCellView
        cell={codeCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Clear' }));
    expect(onClear).toHaveBeenCalledOnce();
    expect(onStatus).toHaveBeenCalledWith('Outputs cleared');
  });

  it('does not show Clear button when there are no outputs', () => {
    const noOutputCell: NotebookCell = { ...codeCell, outputs: [] };
    render(
      <NotebookCellView
        cell={noOutputCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(screen.queryByRole('button', { name: 'Clear' })).not.toBeInTheDocument();
  });

  it('renders placeholder based on cell type', () => {
    render(
      <NotebookCellView
        cell={markdownCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(screen.getByLabelText('Write Markdown...')).toBeInTheDocument();
    render(
      <NotebookCellView
        cell={codeCell}
        selected={false}
        fontSize={14}
        autocompleteEnabled={true}
        foldingEnabled={true}
        onSelect={onSelect}
        onEdit={onEdit}
        onFocus={onFocus}
        onExecute={onExecute}
        onClear={onClear}
        onStatus={onStatus}
      />,
    );
    expect(screen.getByLabelText('Write Rust code...')).toBeInTheDocument();
  });
});
