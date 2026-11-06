import { act, renderHook } from '@testing-library/react';
import { useNotebookEditor } from '../useNotebookEditor';
import type { Notebook } from '../../types/types';

const notebook = (): Notebook => {
  return {
    format_version: 1,
    metadata: { language: 'rust', title: 'Example' },
    cells: [
      {
        id: 'md',
        cell_type: 'markdown',
        source: '# Intro',
        execution_count: null,
        outputs: [],
        metadata: {},
      },
      {
        id: 'code',
        cell_type: 'code',
        source: '1 + 1',
        execution_count: 4,
        outputs: [{ type: 'text', text: '2' }],
        metadata: {},
      },
    ],
  };
};

describe('useNotebookEditor', () => {
  it('selects the default code cell and supports editing, adding, inserting, and converting', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    expect(result.current.selectedId).toBe('code');
    expect(result.current.selectedIndex).toBe(1);
    act(() => result.current.updateCell('code', '2 + 2'));
    expect(result.current.cells[1].source).toBe('2 + 2');
    act(() => result.current.renameNotebook('Renamed notebook'));
    expect(result.current.notebook.metadata.title).toBe('Renamed notebook');
    act(() => result.current.insertCell(0, 'markdown'));
    expect(result.current.cells[0].cell_type).toBe('markdown');
    expect(result.current.mode).toBe('edit');
    act(() => result.current.convertSelected('markdown'));
    expect(result.current.cells[0].outputs).toEqual([]);
    act(() => result.current.addCell('code'));
    expect(result.current.cells).toHaveLength(4);
    expect(result.current.mode).toBe('edit');
  });

  it('duplicates, moves, deletes selected cells and clears outputs', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.duplicateSelected());
    expect(result.current.cells).toHaveLength(3);
    expect(result.current.cells[2].outputs).toEqual([]);
    expect(result.current.cells[2].execution_count).toBeNull();
    act(() => result.current.moveSelected(-1));
    expect(result.current.cells[1].id).toBe(result.current.selectedId);
    act(() => result.current.clearOutputs());
    expect(result.current.cells.every((cell) => cell.outputs.length === 0)).toBe(true);
    act(() => result.current.deleteSelected());
    expect(result.current.cells).toHaveLength(2);
  });

  it('removes a specific cell without changing a different selection', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    expect(result.current.selectedId).toBe('code');

    act(() => result.current.removeCell('md'));

    expect(result.current.cells).toHaveLength(1);
    expect(result.current.cells[0].id).toBe('code');
    expect(result.current.selectedId).toBe('code');
  });

  it('selects a remaining cell when the selected cell is removed', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));

    act(() => result.current.removeCell('code'));

    expect(result.current.cells).toHaveLength(1);
    expect(result.current.selectedId).toBe('md');
  });

  it('supports undo, redo, history reset, and guards invalid operations', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.updateCell('code', 'changed'));
    act(() => result.current.undo());
    expect(result.current.cells[1].source).toBe('1 + 1');
    act(() => result.current.redo());
    expect(result.current.cells[1].source).toBe('changed');
    act(() => result.current.resetHistory(notebook()));
    act(() => result.current.updateCell('code', 'intermediate'));
    act(() => result.current.updateCell('code', 'after reset'));
    act(() => result.current.undo());
    expect(result.current.cells[1].source).toBe('1 + 1');
    act(() => result.current.setSelectedId('missing'));
    act(() => result.current.duplicateSelected());
    act(() => result.current.deleteSelected());
    act(() => result.current.moveSelected(1));
    act(() => result.current.convertSelected('code'));
    expect(result.current.cells).toHaveLength(2);
  });

  it('keeps the single remaining cell and handles notebooks without cells', () => {
    const onlyCell: Notebook = { ...notebook(), cells: [notebook().cells[0]] };
    const { result, rerender } = renderHook(({ data }) => useNotebookEditor(data), {
      initialProps: { data: onlyCell },
    });
    act(() => result.current.deleteSelected());
    act(() => result.current.removeCell('md'));
    expect(result.current.cells).toHaveLength(1);
    rerender({ data: { ...onlyCell, cells: [] } });
    act(() => result.current.removeCell('missing'));
    expect(result.current.cells).toHaveLength(1);
  });
});
