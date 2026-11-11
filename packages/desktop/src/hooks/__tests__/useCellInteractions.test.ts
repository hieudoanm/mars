import { act, renderHook } from '@testing-library/react';
import { useCellInteractions } from '../useCellInteractions';
import { useNotebookEditor } from '../useNotebookEditor';
import type { Notebook, NotebookCell } from '../../types/types';

const notebook = (): Notebook => ({
  format_version: 1,
  metadata: { language: 'rust' },
  cells: [
    {
      id: 'a',
      cell_type: 'markdown',
      source: '# A',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
    { id: 'b', cell_type: 'code', source: '1', execution_count: null, outputs: [], metadata: {} },
    { id: 'c', cell_type: 'code', source: '2', execution_count: null, outputs: [], metadata: {} },
  ],
});

const cell = (id: string): NotebookCell => ({
  id,
  cell_type: 'code',
  source: 'x',
  execution_count: null,
  outputs: [],
  metadata: {},
});

const setup = () => {
  const executeCells = vi.fn(async () => {});
  const hook = renderHook(() => {
    const editor = useNotebookEditor(notebook());
    const interactions = useCellInteractions({ editor, executeCells });
    return { editor, interactions };
  });
  return { ...hook, executeCells };
};

describe('useCellInteractions', () => {
  it('selectCell sets selection and command mode', () => {
    const { result } = setup();
    act(() => result.current.interactions.selectCell(cell('a')));
    expect(result.current.editor.selectedId).toBe('a');
    expect(result.current.editor.mode).toBe('command');
    expect(result.current.interactions.rangeIds.size).toBe(0);
  });

  it('shiftSelectCell selects the inclusive range between anchor and target', () => {
    const { result } = setup();
    act(() => result.current.interactions.selectCell(cell('a')));
    act(() => result.current.interactions.shiftSelectCell(cell('c')));
    expect([...result.current.interactions.rangeIds].sort()).toEqual(['a', 'b', 'c']);
    expect(result.current.editor.selectedId).toBe('c');
    // reverse direction
    act(() => result.current.interactions.shiftSelectCell(cell('a')));
    expect([...result.current.interactions.rangeIds].sort()).toEqual(['a', 'b', 'c']);
  });

  it('shiftSelectCell falls back to selectCell for unknown ids', () => {
    const { result } = setup();
    act(() => result.current.interactions.shiftSelectCell(cell('zzz')));
    expect(result.current.editor.selectedId).toBe('zzz');
  });

  it('deleteCellOrRange deletes the range when it has multiple cells', () => {
    const { result } = setup();
    act(() => result.current.interactions.selectCell(cell('a')));
    act(() => result.current.interactions.shiftSelectCell(cell('b')));
    act(() => result.current.interactions.deleteCellOrRange(cell('b')));
    expect(result.current.editor.cells.map((c) => c.id)).toEqual(['c']);
    expect(result.current.interactions.rangeIds.size).toBe(0);
  });

  it('deleteCellOrRange removes a single cell outside the range', () => {
    const { result } = setup();
    act(() => result.current.interactions.selectCell(cell('a')));
    act(() => result.current.interactions.deleteCellOrRange(cell('b')));
    expect(result.current.editor.cells.map((c) => c.id)).toEqual(['a', 'c']);
  });

  it('runSelectedCells executes the range or the single selection', async () => {
    const { result, executeCells } = setup();
    act(() => result.current.interactions.selectCell(cell('b')));
    await act(async () => result.current.interactions.runSelectedCells());
    expect(executeCells).toHaveBeenCalledWith([expect.objectContaining({ id: 'b' })]);

    act(() => result.current.interactions.shiftSelectCell(cell('c')));
    await act(async () => result.current.interactions.runSelectedCells());
    const lastCall = (executeCells.mock.calls.at(-1) as [NotebookCell[]] | undefined)?.[0];
    expect(lastCall?.map((c) => c.id).sort()).toEqual(['b', 'c']);
  });

  it('drag and drop moves a cell onto another', () => {
    const { result } = setup();
    act(() => result.current.interactions.handleDragCellStart('a'));
    act(() => result.current.interactions.handleCellDragOver('c'));
    expect(result.current.interactions.dropTargetId).toBe('c');
    act(() => result.current.interactions.handleCellDrop('c'));
    expect(result.current.editor.cells.map((c) => c.id)).toEqual(['b', 'c', 'a']);
    expect(result.current.interactions.dropTargetId).toBeNull();
  });

  it('ignores drops onto itself and clears drag state on end', () => {
    const { result } = setup();
    act(() => result.current.interactions.handleDragCellStart('a'));
    act(() => result.current.interactions.handleCellDragOver('a'));
    expect(result.current.interactions.dropTargetId).toBeNull();
    act(() => result.current.interactions.handleCellDrop('a'));
    expect(result.current.editor.cells.map((c) => c.id)).toEqual(['a', 'b', 'c']);
    act(() => result.current.interactions.handleDragCellEnd());
    act(() => result.current.interactions.handleCellDragOver('b'));
    expect(result.current.interactions.dropTargetId).toBeNull();
  });
});
