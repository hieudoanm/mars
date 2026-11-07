import { act, renderHook } from '@testing-library/react';
import type { NotebookCell } from '../../types/types';
import { useNotebookExecutionCommands } from '../useNotebookExecutionCommands';

const createCell = (id: string, cellType: 'code' | 'markdown'): NotebookCell => ({
  id,
  cell_type: cellType,
  source: '',
  execution_count: null,
  outputs: [],
  metadata: {},
});

const cells = [createCell('markdown', 'markdown'), createCell('code', 'code')];

const renderCommands = (selectedIndex = 1) => {
  const executeCells = vi.fn().mockResolvedValue(undefined);
  const hook = renderHook(() =>
    useNotebookExecutionCommands({ cells, selectedIndex, executeCells }),
  );
  return { ...hook, executeCells };
};

describe('useNotebookExecutionCommands', () => {
  it('returns handlers', () => {
    const { result } = renderCommands();
    expect(typeof result.current.executeAll).toBe('function');
    expect(typeof result.current.executeCurrent).toBe('function');
    expect(typeof result.current.executeAbove).toBe('function');
    expect(typeof result.current.executeBelow).toBe('function');
  });

  it('executes the selected code cell', async () => {
    const { result, executeCells } = renderCommands();

    await act(async () => {
      await result.current.executeCurrent();
    });

    expect(executeCells).toHaveBeenCalledWith([cells[1]]);
  });

  it('ignores the current cell when it is not code', async () => {
    const { result, executeCells } = renderCommands(0);

    await act(async () => {
      await result.current.executeCurrent();
    });

    expect(executeCells).not.toHaveBeenCalled();
  });

  it('executes only code cells', async () => {
    const { result, executeCells } = renderCommands();

    await act(async () => {
      await result.current.executeAll();
    });

    expect(executeCells).toHaveBeenCalledWith([cells[1]]);
  });

  it('executes code cells above and below the selection', async () => {
    const { result, executeCells } = renderCommands();

    await act(async () => {
      await result.current.executeAbove();
    });
    expect(executeCells).toHaveBeenCalledWith([]);

    await act(async () => {
      await result.current.executeBelow();
    });
    expect(executeCells).toHaveBeenCalledWith([]);
  });
});
