import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { NotebookCell } from '../../lib/types';
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

describe('NotebookSidebar', () => {
  it('renders notebook heading and file info', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='cell-md'
        filePath='/notebooks/demo.mars'
        isDirty={false}
        onSelect={onSelect}
      />,
    );
    expect(screen.getByText('Notebook')).toBeInTheDocument();
    expect(screen.getByText(/demo\.mars/)).toBeInTheDocument();
    expect(screen.getByText('Saved notebook')).toBeInTheDocument();
  });

  it('renders unsaved state', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='cell-md'
        filePath={null}
        isDirty
        onSelect={onSelect}
      />,
    );
    expect(screen.getByText(/Untitled\.mars/)).toBeInTheDocument();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });

  it('renders cell navigation buttons', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='cell-md'
        filePath={null}
        isDirty={false}
        onSelect={onSelect}
      />,
    );
    expect(screen.getByText('Cells')).toBeInTheDocument();
    expect(screen.getByText(/markdown/)).toBeInTheDocument();
    expect(screen.getByText(/code/)).toBeInTheDocument();
    expect(screen.getByText(/raw/)).toBeInTheDocument();
  });

  it('highlights selected cell', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='cell-code'
        filePath={null}
        isDirty={false}
        onSelect={onSelect}
      />,
    );
    const buttons = screen.getAllByRole('button');
    expect(buttons[1]).toHaveClass('selected');
    expect(buttons[0]).not.toHaveClass('selected');
    expect(buttons[2]).not.toHaveClass('selected');
  });

  it('calls onSelect when a cell button is clicked', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='cell-md'
        filePath={null}
        isDirty={false}
        onSelect={onSelect}
      />,
    );
    const buttons = screen.getAllByRole('button');
    fireEvent.click(buttons[2]);
    expect(onSelect).toHaveBeenCalledWith('cell-raw');
  });

  it('shows dirty indicator next to filename', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='cell-md'
        filePath='/notebooks/demo.mars'
        isDirty
        onSelect={onSelect}
      />,
    );
    expect(screen.getByText(/demo\.mars •/)).toBeInTheDocument();
  });
});
