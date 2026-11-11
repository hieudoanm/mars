import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import NotebookCellHeader, { type NotebookCellHeaderProps } from '../NotebookCellHeader';

const setup = (overrides: Partial<NotebookCellHeaderProps> = {}) => {
  const props: NotebookCellHeaderProps = {
    cellId: 'c1',
    cellType: 'code',
    cellLanguage: 'rust',
    isExecuting: false,
    runInfo: null,
    collapsed: false,
    canDelete: true,
    hasOutputs: true,
    markdownPreviewVisible: false,
    onToggleCollapse: vi.fn(),
    onSearch: vi.fn(),
    onToggleMarkdownPreview: vi.fn(),
    onExecute: vi.fn(),
    onClear: vi.fn(),
    onDelete: vi.fn(),
    onStatus: vi.fn(),
    ...overrides,
  };
  render(<NotebookCellHeader {...props} />);
  return props;
};

describe('NotebookCellHeader', () => {
  it('renders badges for executing, queued, and finished states', () => {
    const { unmount } = render(
      <NotebookCellHeader
        cellId='c1'
        cellType='code'
        isExecuting
        isQueued
        collapsed={false}
        canDelete
        hasOutputs={false}
        markdownPreviewVisible={false}
        onToggleCollapse={vi.fn()}
        onSearch={vi.fn()}
        onToggleMarkdownPreview={vi.fn()}
        onExecute={vi.fn()}
        onClear={vi.fn()}
        onDelete={vi.fn()}
        onStatus={vi.fn()}
      />,
    );
    expect(screen.getByText('Running…')).toBeInTheDocument();
    unmount();

    setup({ runInfo: { status: 'success', durationMs: 12 } });
    expect(screen.getByText(/12ms/)).toBeInTheDocument();
    setup({ isQueued: true });
    expect(screen.getByText('Queued')).toBeInTheDocument();
  });

  it('changes cell type and language via selects', () => {
    const onChangeType = vi.fn();
    const onChangeLanguage = vi.fn();
    setup({ onChangeType, onChangeLanguage });
    fireEvent.change(screen.getByLabelText('Cell type'), { target: { value: 'markdown' } });
    expect(onChangeType).toHaveBeenCalledWith('markdown');
    fireEvent.change(screen.getByLabelText('Cell language'), { target: { value: 'python' } });
    expect(onChangeLanguage).toHaveBeenCalledWith('python');
  });

  it('hides the language select for non-code cells', () => {
    setup({ cellType: 'markdown', markdownPreviewVisible: true });
    expect(screen.queryByLabelText('Cell language')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Run cell (Shift+Enter)' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Hide Markdown preview' })).toBeInTheDocument();
  });

  it('fires action callbacks', () => {
    const props = setup({
      onMoveUp: vi.fn(),
      onMoveDown: vi.fn(),
      onMergeNext: vi.fn(),
      onSplitCell: vi.fn(),
      onUpdateTags: vi.fn(),
    });
    fireEvent.click(screen.getByRole('button', { name: 'Move cell up' }));
    expect(props.onMoveUp).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Move cell down' }));
    expect(props.onMoveDown).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Merge with next cell' }));
    expect(props.onMergeNext).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Split cell at cursor' }));
    expect(props.onSplitCell).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Run cell (Shift+Enter)' }));
    expect(props.onExecute).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Clear outputs' }));
    expect(props.onClear).toHaveBeenCalled();
    expect(props.onStatus).toHaveBeenCalledWith('Outputs cleared');
    fireEvent.click(screen.getByRole('button', { name: 'Delete cell (D, D)' }));
    expect(props.onDelete).toHaveBeenCalled();
  });

  it('toggles collapse and search', () => {
    const props = setup({ collapsed: true });
    fireEvent.click(screen.getByRole('button', { name: 'Expand cell' }));
    expect(props.onToggleCollapse).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Search (Cmd+F)' }));
    expect(props.onSearch).toHaveBeenCalled();
  });

  it('toggles markdown preview on markdown cells', () => {
    const props = setup({ cellType: 'markdown' });
    fireEvent.click(screen.getByRole('button', { name: 'Show Markdown preview' }));
    expect(props.onToggleMarkdownPreview).toHaveBeenCalled();
  });

  it('disables delete for the last cell and edits tags', () => {
    const onUpdateTags = vi.fn();
    setup({ canDelete: false, tags: ['rust'], onUpdateTags });
    expect(screen.getByRole('button', { name: 'Delete cell (D, D)' })).toBeDisabled();
    expect(screen.getByText('rust')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Edit cell tags' }));
    fireEvent.change(screen.getByLabelText('Cell tags'), { target: { value: 'rust, ml' } });
    fireEvent.blur(screen.getByLabelText('Cell tags'));
    expect(onUpdateTags).toHaveBeenCalledWith(['rust', 'ml']);
  });

  it('handles drag start/end', () => {
    const onDragCellStart = vi.fn();
    const onDragCellEnd = vi.fn();
    setup({ onDragCellStart, onDragCellEnd });
    const grip = screen.getByLabelText('Drag to reorder cell');
    fireEvent.dragStart(grip, {
      dataTransfer: { effectAllowed: '', setData: vi.fn() },
    });
    expect(onDragCellStart).toHaveBeenCalledWith('c1');
    fireEvent.dragEnd(grip);
    expect(onDragCellEnd).toHaveBeenCalled();
  });
});
