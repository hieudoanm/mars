import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { NotebookCell } from '../../../types/types';
import NotebookSidebar from '../NotebookSidebar';

const cells: NotebookCell[] = [
  {
    id: 'cell-md',
    cell_type: 'markdown',
    source: '# Intro',
    execution_count: null,
    outputs: [],
    metadata: {},
  },
  {
    id: 'cell-code',
    cell_type: 'code',
    source: '1 + 1',
    execution_count: 1,
    outputs: [],
    metadata: {},
  },
  {
    id: 'cell-raw',
    cell_type: 'raw',
    source: 'raw content',
    execution_count: null,
    outputs: [],
    metadata: {},
  },
];

const onRenameFile = vi.fn().mockResolvedValue(true);

const renderSidebar = ({
  selectedId = 'cell-md',
  filePath = null,
  isDirty = false,
  onSelect = vi.fn(),
}: {
  selectedId?: string;
  filePath?: string | null;
  isDirty?: boolean;
  onSelect?: (id: string) => void;
} = {}) =>
  render(
    <NotebookSidebar
      cells={cells}
      selectedId={selectedId}
      filePath={filePath}
      isDirty={isDirty}
      onSelect={onSelect}
      onRenameFile={onRenameFile}
    />,
  );

describe('NotebookSidebar', () => {
  beforeEach(() => {
    onRenameFile.mockReset().mockResolvedValue(true);
  });

  it('renders notebook heading and file info', () => {
    renderSidebar({ filePath: '/notebooks/demo.mars' });
    expect(screen.getByText('Notebook')).toBeInTheDocument();
    expect(screen.getByText(/demo\.mars/)).toBeInTheDocument();
    expect(screen.getByText('Saved notebook')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Rename file' })).toBeInTheDocument();
  });

  it('renders unsaved state without a rename action', () => {
    renderSidebar({ filePath: null, isDirty: true });
    expect(screen.getByText(/Untitled\.mars/)).toBeInTheDocument();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Rename file' })).not.toBeInTheDocument();
  });

  it('renders cell navigation buttons', () => {
    renderSidebar();
    expect(screen.getByText('Cells')).toBeInTheDocument();
    expect(screen.getByText(/markdown/)).toBeInTheDocument();
    expect(screen.getByText(/code/)).toBeInTheDocument();
    expect(screen.getByText(/raw/)).toBeInTheDocument();
  });

  it('highlights selected cell', () => {
    renderSidebar({ selectedId: 'cell-code' });
    const buttons = screen.getAllByRole('button');
    expect(buttons[1].className).toContain('bg-orange-950');
    expect(buttons[0].className).not.toContain('bg-orange-950');
    expect(buttons[2].className).not.toContain('bg-orange-950');
  });

  it('calls onSelect when a cell button is clicked', () => {
    const onSelect = vi.fn();
    renderSidebar({ onSelect });
    const buttons = screen.getAllByRole('button');
    fireEvent.click(buttons[2]);
    expect(onSelect).toHaveBeenCalledWith('cell-raw');
  });

  it('shows dirty indicator next to filename', () => {
    renderSidebar({ filePath: '/notebooks/demo.mars', isDirty: true });
    expect(screen.getByText(/demo\.mars •/)).toBeInTheDocument();
  });

  it('renames the file on commit', () => {
    renderSidebar({ filePath: '/notebooks/demo.mars' });

    fireEvent.click(screen.getByRole('button', { name: 'Rename file' }));
    const input = screen.getByLabelText('File name');
    expect(input).toHaveValue('demo.mars');

    fireEvent.change(input, { target: { value: 'analysis' } });
    fireEvent.blur(input);

    expect(onRenameFile).toHaveBeenCalledWith('analysis');
    expect(screen.queryByLabelText('File name')).not.toBeInTheDocument();
  });

  it('commits the file name with Enter', () => {
    renderSidebar({ filePath: '/notebooks/demo.mars' });

    fireEvent.click(screen.getByRole('button', { name: 'Rename file' }));
    const input = screen.getByLabelText('File name');
    fireEvent.change(input, { target: { value: 'analysis.mars' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onRenameFile).toHaveBeenCalledWith('analysis.mars');
  });

  it('cancels renaming with Escape', () => {
    renderSidebar({ filePath: '/notebooks/demo.mars' });

    fireEvent.click(screen.getByRole('button', { name: 'Rename file' }));
    const input = screen.getByLabelText('File name');
    fireEvent.change(input, { target: { value: 'discard.mars' } });
    fireEvent.keyDown(input, { key: 'Escape' });

    expect(onRenameFile).not.toHaveBeenCalled();
    expect(screen.queryByLabelText('File name')).not.toBeInTheDocument();
  });

  it('displays Windows file names', () => {
    renderSidebar({ filePath: 'C:\\notebooks\\demo.mars' });
    expect(screen.getByText(/demo\.mars/)).toBeInTheDocument();
  });
});
