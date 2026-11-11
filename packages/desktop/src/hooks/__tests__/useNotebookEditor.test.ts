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

  it('pastes cells at an index and selects the new cell in command mode', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() =>
      result.current.pasteCell(1, { source: 'pasted()', cellType: 'code', language: 'rust' }),
    );
    expect(result.current.cells).toHaveLength(3);
    expect(result.current.cells[1].source).toBe('pasted()');
    expect(result.current.selectedId).toBe(result.current.cells[1].id);
    expect(result.current.mode).toBe('command');
  });

  it('duplicates a cell by id', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.duplicateCellById('md'));
    expect(result.current.cells).toHaveLength(3);
    expect(result.current.cells[1].source).toBe('# Intro');
    act(() => result.current.duplicateCellById('missing'));
    expect(result.current.cells).toHaveLength(3);
  });

  it('deletes multiple cells by id and reselects', () => {
    const three: Notebook = {
      ...notebook(),
      cells: [...notebook().cells, { ...notebook().cells[1], id: 'tail', source: 'tail' }],
    };
    const { result } = renderHook(() => useNotebookEditor(three));
    act(() => result.current.deleteCellsByIds(new Set(['md', 'tail'])));
    expect(result.current.cells.map((cell) => cell.id)).toEqual(['code']);
    act(() => result.current.setSelectedId('code'));
    act(() => result.current.deleteCellsByIds(new Set(['code'])));
    // deleting the only remaining cell is a no-op
    expect(result.current.cells).toHaveLength(1);
  });

  it('splits, merges, and moves cells by id', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.splitCellById('code', 2));
    expect(result.current.cells).toHaveLength(3);
    act(() => result.current.mergeCellById('code'));
    expect(result.current.cells).toHaveLength(2);
    expect(result.current.cells[1].source).toBe('1 \n+ 1');
    act(() => result.current.moveCellById('code', -1));
    expect(result.current.cells[0].id).toBe('code');
    act(() => result.current.moveCellById('missing', 1));
    expect(result.current.cells).toHaveLength(2);
  });

  it('moves a cell to another cell position', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.moveCellTo('md', 'code'));
    expect(result.current.cells[0].id).toBe('code');
    act(() => result.current.moveCellTo('md', 'md'));
    act(() => result.current.moveCellTo('missing', 'code'));
    expect(result.current.cells).toHaveLength(2);
  });

  it('converts cells by id and type', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.convertCellType('md', 'code'));
    expect(result.current.cells[0].cell_type).toBe('code');
    expect(result.current.cells[0].outputs).toEqual([]);
  });

  it('updates metadata, tags, language, attachments, and dependencies', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.updateCellMetadata('code', { custom: true }));
    expect(result.current.cells[1].metadata).toEqual({ custom: true });
    act(() => result.current.updateTags('code', ['ml', 'rust']));
    expect(result.current.cells[1].metadata.tags).toEqual(['ml', 'rust']);
    act(() => result.current.setCellLanguage('code', 'python'));
    expect(result.current.cells[1].metadata.language).toBe('python');
    act(() => result.current.updateAttachments('code', { 'a.png': 'data:image/png;base64,AA==' }));
    expect(result.current.cells[1].metadata.attachments).toEqual({
      'a.png': 'data:image/png;base64,AA==',
    });
    act(() => result.current.setDependencies({ serde: '1.0' }));
    expect(result.current.notebook.metadata.dependencies).toEqual({ serde: '1.0' });
  });

  it('merges selected cell with next and splits at an offset', () => {
    const three: Notebook = {
      ...notebook(),
      cells: [...notebook().cells, { ...notebook().cells[1], id: 'tail', source: 'tail' }],
    };
    const { result } = renderHook(() => useNotebookEditor(three));
    act(() => result.current.setSelectedId('md'));
    act(() => result.current.mergeSelectedWithNext());
    expect(result.current.cells).toHaveLength(2);
    act(() => result.current.splitSelectedAt(2));
    expect(result.current.cells.length).toBeGreaterThanOrEqual(2);
  });

  it('removes a specific output from a cell', () => {
    const { result } = renderHook(() => useNotebookEditor(notebook()));
    act(() => result.current.removeOutput('code', 0));
    expect(result.current.cells[1].outputs).toEqual([]);
  });
});
