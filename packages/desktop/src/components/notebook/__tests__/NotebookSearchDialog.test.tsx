import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import NotebookSearchDialog, { searchNotebookCells } from '../NotebookSearchDialog';
import type { NotebookCell } from '../../../types/types';

const cell = (
  id: string,
  source: string,
  cell_type: NotebookCell['cell_type'] = 'code',
): NotebookCell => ({
  id,
  cell_type,
  source,
  execution_count: null,
  outputs: [],
  metadata: {},
});

const cells = [
  cell('a', 'fn main() {\n  println!("hi");\n}'),
  cell('b', '# Title\nSome println docs', 'markdown'),
];

describe('searchNotebookCells', () => {
  it('finds matches with cell, line, and character info', () => {
    const matches = searchNotebookCells(cells, 'println');
    expect(matches).toHaveLength(2);
    expect(matches[0]).toMatchObject({ cellId: 'a', cellIndex: 0, lineNumber: 2, character: 2 });
    expect(matches[1]).toMatchObject({ cellId: 'b', cellIndex: 1, cellType: 'markdown' });
  });

  it('returns no matches for empty queries', () => {
    expect(searchNotebookCells(cells, '')).toEqual([]);
    expect(searchNotebookCells(cells, '   ')).toEqual([]);
  });

  it('respects search options', () => {
    const caseCells = [cell('c', 'Foo\nfoo')];
    expect(searchNotebookCells(caseCells, 'foo')).toHaveLength(2);
    expect(
      searchNotebookCells(caseCells, 'foo', {
        caseSensitive: true,
        wholeWord: false,
        regex: false,
      }),
    ).toHaveLength(1);
    expect(
      searchNotebookCells(caseCells, 'f.o', {
        caseSensitive: false,
        wholeWord: false,
        regex: true,
      }),
    ).toHaveLength(2);
  });
});

describe('NotebookSearchDialog', () => {
  const setup = (overrides: Partial<Parameters<typeof NotebookSearchDialog>[0]> = {}) => {
    const props = {
      cells,
      open: true,
      onClose: vi.fn(),
      onSelectCell: vi.fn(),
      onReplaceAll: vi.fn(),
      ...overrides,
    };
    render(<NotebookSearchDialog {...props} />);
    return props;
  };

  it('renders nothing when closed', () => {
    const { container } = render(
      <NotebookSearchDialog cells={cells} open={false} onClose={vi.fn()} onSelectCell={vi.fn()} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('filters matches as the query changes and selects on click', () => {
    const props = setup();
    const input = screen.getByLabelText('Search query');
    fireEvent.change(input, { target: { value: 'println' } });
    expect(screen.getAllByRole('button', { name: /Cell \d/ })).toHaveLength(2);
    fireEvent.click(screen.getAllByRole('button', { name: /Cell \d/ })[0]);
    expect(props.onSelectCell).toHaveBeenCalledWith('a', 2, 2);
    expect(props.onClose).toHaveBeenCalled();
  });

  it('selects the first match on Enter and closes on Escape', () => {
    const props = setup();
    const input = screen.getByLabelText('Search query');
    fireEvent.change(input, { target: { value: 'println' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(props.onSelectCell).toHaveBeenCalledWith('a', 2, 2);
    fireEvent.change(input, { target: { value: 'zzz' } });
    fireEvent.keyDown(input, { key: 'Escape' });
    expect(props.onClose).toHaveBeenCalledTimes(2);
  });

  it('shows a no-matches message', () => {
    setup();
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'nomatch' } });
    expect(screen.getByText('No matches.')).toBeInTheDocument();
  });

  it('toggles search option flags', () => {
    setup();
    const matchCase = screen.getByRole('button', { name: 'Match case' });
    expect(matchCase).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(matchCase);
    expect(matchCase).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: 'Match whole word' }));
    fireEvent.click(screen.getByRole('button', { name: 'Use regular expression' }));
  });

  it('replaces all after confirmation', () => {
    const props = setup();
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'println' } });
    fireEvent.change(screen.getByLabelText('Replace with'), { target: { value: 'print' } });
    fireEvent.click(screen.getByRole('button', { name: 'Replace all occurrences' }));
    expect(screen.getByText(/Replace 2 occurrences/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Replace all' }));
    expect(props.onReplaceAll).toHaveBeenCalledWith(
      'println',
      'print',
      expect.objectContaining({ caseSensitive: false }),
    );
  });

  it('cancels the replace confirmation', () => {
    setup();
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'println' } });
    fireEvent.click(screen.getByRole('button', { name: 'Replace all occurrences' }));
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByText(/Replace \d+ occurrence/)).toBeNull();
  });

  it('hides replace controls without onReplaceAll', () => {
    setup({ onReplaceAll: undefined });
    expect(screen.queryByLabelText('Replace with')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Replace all occurrences' })).toBeNull();
  });

  it('closes via the close button', () => {
    const props = setup();
    fireEvent.click(screen.getByRole('button', { name: 'Close search' }));
    expect(props.onClose).toHaveBeenCalled();
  });
});
