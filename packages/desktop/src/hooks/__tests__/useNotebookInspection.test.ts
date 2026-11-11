import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { NotebookCell } from '../../types/types';
import { diagnoseCell, inspectVariables, lspDiagnostics } from '../../lib/kernelService';
import { useNotebookInspection } from '../useNotebookInspection';

vi.mock('../../lib/kernelService', () => ({
  diagnoseCell: vi.fn(async () => []),
  inspectVariables: vi.fn(async () => []),
  lspDiagnostics: vi.fn(async () => []),
}));

const cell = (id: string, source = 'fn main() {}'): NotebookCell => ({
  id,
  cell_type: 'code',
  source,
  execution_count: null,
  outputs: [],
  metadata: {},
});

const setup = (cells: NotebookCell[], selectedIndex = 0, executionCount = 0) =>
  renderHook(() =>
    useNotebookInspection({
      sessionId: 's',
      cells,
      selectedIndex,
      executionCount,
      kernelLanguage: 'rust',
    }),
  );

describe('useNotebookInspection', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
  });
  afterEach(() => vi.useRealTimers());

  it('refreshes variables after executions and clears them at zero', async () => {
    vi.mocked(inspectVariables).mockResolvedValue([{ name: 'x', typeName: 'i32', value: '1' }]);
    const { result, rerender } = setup([cell('a')], 0, 1);
    await act(async () => {
      vi.advanceTimersByTime(500);
    });
    expect(inspectVariables).toHaveBeenCalledWith('s', 'rust');
    expect(result.current.liveVariables).toHaveLength(1);

    rerender();
    // executionCount prop change requires new renderHook props; emulate via setup below
  });

  it('clears variables when execution count is zero', async () => {
    const { result } = setup([cell('a')], 0, 0);
    expect(result.current.liveVariables).toEqual([]);
    expect(inspectVariables).not.toHaveBeenCalled();
  });

  it('refreshVariables can be called manually and survives failures', async () => {
    vi.mocked(inspectVariables).mockRejectedValueOnce(new Error('down'));
    const { result } = setup([cell('a')]);
    await act(async () => {
      result.current.refreshVariables();
      await Promise.resolve();
    });
    expect(result.current.liveVariables).toEqual([]);
    vi.mocked(inspectVariables).mockResolvedValue([{ name: 'y', typeName: null, value: null }]);
    await act(async () => {
      result.current.refreshVariables();
      await Promise.resolve();
    });
    expect(result.current.liveVariables).toHaveLength(1);
  });

  it('runs pre-diagnostics for the selected runnable cell', async () => {
    vi.mocked(diagnoseCell).mockResolvedValue([
      {
        cellIndex: 1,
        line: 1,
        column: 1,
        endLine: 1,
        endColumn: 2,
        severity: 'error',
        message: 'bad',
      },
    ] as never);
    const { result } = setup([cell('a', 'let x = ;')], 0, 1);
    await act(async () => {
      vi.advanceTimersByTime(800);
    });
    expect(diagnoseCell).toHaveBeenCalled();
    expect(result.current.preDiagnostics.a?.length).toBeGreaterThan(0);
  });

  it('skips diagnostics for non-code or unselected cells', async () => {
    const md: NotebookCell = { ...cell('m', '# hi'), cell_type: 'markdown' };
    const { result } = setup([md], 0, 1);
    await act(async () => {
      vi.advanceTimersByTime(1000);
    });
    expect(diagnoseCell).not.toHaveBeenCalled();
    expect(result.current.preDiagnostics).toEqual({});
  });

  it('polls LSP diagnostics and maps them to cells', async () => {
    vi.mocked(lspDiagnostics).mockResolvedValue([
      {
        cellId: 'a',
        line: 0,
        character: 0,
        endLine: 0,
        endCharacter: 1,
        severity: 'warning',
        message: 'lint',
      },
    ] as never);
    const { result } = setup([cell('a')]);
    await act(async () => {
      await Promise.resolve();
    });
    expect(result.current.lspDiagnosticsByCell.a?.length).toBeGreaterThan(0);
    expect(result.current.problems.length).toBeGreaterThan(0);
    expect(result.current.problems[0].cellIndex).toBe(0);
  });

  it('problems reflect severity and line numbers across cells', async () => {
    vi.mocked(lspDiagnostics).mockResolvedValue([
      {
        cellId: 'b',
        line: 2,
        character: 0,
        endLine: 2,
        endCharacter: 1,
        severity: 'error',
        message: 'err',
      },
    ] as never);
    const cells = [cell('a', 'one\ntwo'), cell('b', 'x\ny\nz')];
    const { result } = setup(cells, 1, 0);
    await act(async () => {
      await Promise.resolve();
    });
    const problem = result.current.problems.find((p) => p.cellId === 'b');
    expect(problem?.severity).toBe('error');
    expect(problem?.line).toBeGreaterThan(0);
  });
});
