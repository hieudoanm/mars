import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const {
  onRename,
  onTimeoutChange,
  onFontSizeChange,
  onCancel,
  onRunCurrent,
  onRunAll,
  onRunAbove,
  onRunBelow,
  onClearOutputs,
  onDeleteCell,
  onAddMarkdown,
  onAddCode,
  onAddRaw,
  onCollapseAllCells,
  onExpandAllCells,
} = vi.hoisted(() => ({
  onRename: vi.fn(),
  onTimeoutChange: vi.fn(),
  onFontSizeChange: vi.fn(),
  onCancel: vi.fn(),
  onRunCurrent: vi.fn(),
  onRunAll: vi.fn(),
  onRunAbove: vi.fn(),
  onRunBelow: vi.fn(),
  onClearOutputs: vi.fn(),
  onDeleteCell: vi.fn(),
  onAddMarkdown: vi.fn(),
  onAddCode: vi.fn(),
  onAddRaw: vi.fn(),
  onCollapseAllCells: vi.fn(),
  onExpandAllCells: vi.fn(),
}));

import NotebookToolbar, { type NotebookToolbarProps } from '../NotebookToolbar';

const toolbar = (overrides: Partial<NotebookToolbarProps> = {}) => (
  <NotebookToolbar
    fileName='experiment.mars'
    title='My Notebook'
    isDirty={false}
    onRename={onRename}
    isExecuting={false}
    executionTimeout=''
    fontSize={14}
    onTimeoutChange={onTimeoutChange}
    onFontSizeChange={onFontSizeChange}
    onCancel={onCancel}
    onRunCurrent={onRunCurrent}
    onRunAll={onRunAll}
    onRunAbove={onRunAbove}
    onRunBelow={onRunBelow}
    onClearOutputs={onClearOutputs}
    canDeleteCell={true}
    onDeleteCell={onDeleteCell}
    onAddMarkdown={onAddMarkdown}
    onAddCode={onAddCode}
    onAddRaw={onAddRaw}
    onCollapseAllCells={onCollapseAllCells}
    onExpandAllCells={onExpandAllCells}
    {...overrides}
  />
);

describe('NotebookToolbar', () => {
  beforeEach(() => {
    onRename.mockReset();
    onTimeoutChange.mockReset();
    onFontSizeChange.mockReset();
    onCancel.mockReset();
    onRunCurrent.mockReset();
    onRunAll.mockReset();
    onRunAbove.mockReset();
    onRunBelow.mockReset();
    onClearOutputs.mockReset();
    onDeleteCell.mockReset();
    onAddMarkdown.mockReset();
    onAddCode.mockReset();
    onAddRaw.mockReset();
    onCollapseAllCells.mockReset();
    onExpandAllCells.mockReset();
  });

  it('renders breadcrumb, title, and dirty indicator', () => {
    render(toolbar({ isDirty: true }));
    expect(screen.getByText('Workspace')).toBeInTheDocument();
    expect(screen.getByText('experiment.mars')).toHaveAttribute('title', 'experiment.mars');
    expect(screen.getByTitle('Unsaved changes')).toHaveAttribute(
      'data-component',
      'NotebookDirtyIndicator',
    );
    expect(screen.getByRole('heading', { name: 'My Notebook' })).toHaveAttribute(
      'title',
      'My Notebook',
    );
  });

  it('renders clean state without dirty indicator', () => {
    render(toolbar());
    expect(screen.queryByTitle('Unsaved changes')).not.toBeInTheDocument();
  });

  it('renders execution timeout input', () => {
    render(toolbar());
    expect(screen.getByLabelText('Execution timeout in milliseconds')).toBeInTheDocument();
    expect(screen.getByLabelText('Execution timeout in milliseconds')).toHaveValue(null);
  });

  it('renders Stop button only when executing', () => {
    const { rerender } = render(toolbar());
    expect(screen.queryByRole('button', { name: 'Stop' })).not.toBeInTheDocument();

    rerender(toolbar({ isExecuting: true }));
    expect(screen.getByRole('button', { name: 'Stop' })).toBeInTheDocument();
  });

  it('renders tooltips instead of visible action text', () => {
    render(toolbar({ isExecuting: true }));
    const buttons = screen.getAllByRole('button');
    const tooltips = screen.getAllByRole('tooltip');

    expect(tooltips.map((tooltip) => tooltip.textContent)).toEqual([
      'Rename notebook',
      'Stop',
      'Run current cell',
      'Run all (F6)',
      'Run above',
      'Run below',
      'Clear outputs',
      'Collapse all cells',
      'Expand all cells',
      'Add Markdown',
      'Add Code',
      'Add Raw',
      'Delete selected cell (D, D)',
    ]);

    for (const [index, button] of buttons.entries()) {
      expect(button.textContent).toBe('');
      expect(button.querySelector('svg')).toBeInTheDocument();
      expect(button).toHaveAttribute('aria-label', tooltips[index].textContent);
      expect(button).toHaveAttribute('aria-describedby', tooltips[index].id);
    }
  });

  it('routes toolbar actions to their handlers', () => {
    render(toolbar());
    fireEvent.click(screen.getByRole('button', { name: 'Run current cell' }));
    fireEvent.click(screen.getByRole('button', { name: /Run all/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Run above' }));
    fireEvent.click(screen.getByRole('button', { name: 'Run below' }));
    fireEvent.click(screen.getByRole('button', { name: 'Clear outputs' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add Markdown' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add Code' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add Raw' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete selected cell (D, D)' }));

    expect(onRunCurrent).toHaveBeenCalledOnce();
    expect(onRunAll).toHaveBeenCalledOnce();
    expect(onRunAbove).toHaveBeenCalledOnce();
    expect(onRunBelow).toHaveBeenCalledOnce();
    expect(onClearOutputs).toHaveBeenCalledOnce();
    expect(onAddMarkdown).toHaveBeenCalledOnce();
    expect(onAddCode).toHaveBeenCalledOnce();
    expect(onAddRaw).toHaveBeenCalledOnce();
    expect(onDeleteCell).toHaveBeenCalledOnce();
  });

  it('disables delete when the last cell cannot be removed', () => {
    render(toolbar({ canDeleteCell: false }));
    expect(screen.getByRole('button', { name: 'Delete selected cell (D, D)' })).toBeDisabled();
  });

  it('calls onCancel when Stop is clicked during execution', () => {
    render(toolbar({ isExecuting: true }));
    fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('updates font size input', () => {
    render(toolbar());
    const input = screen.getByLabelText('Editor font size');
    expect(input).toHaveValue(14);
    fireEvent.change(input, { target: { value: '18' } });
    expect(onFontSizeChange).toHaveBeenCalledWith(18);
  });
});
