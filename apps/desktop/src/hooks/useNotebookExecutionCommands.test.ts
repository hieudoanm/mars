import { renderHook } from '@testing-library/react';
import { useNotebookExecutionCommands } from './useNotebookExecutionCommands';

describe('useNotebookExecutionCommands', () => {
  it('returns handlers', () => {
    const { result } = renderHook(() =>
      useNotebookExecutionCommands({
        cells: [],
        selectedIndex: 0,
        executeCells: async () => undefined,
      } as unknown as Parameters<typeof useNotebookExecutionCommands>[0]),
    );
    expect(typeof result.current.executeAll).toBe('function');
    expect(typeof result.current.executeAbove).toBe('function');
  });
});
