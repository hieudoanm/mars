import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import type { Dispatch, SetStateAction } from 'react';
import type { Notebook, NotebookCell } from '../../types/types';

vi.mock('../../lib/kernelService', () => ({
  hasInterruptedOutput: vi.fn(() => false),
  executeSource: vi.fn(async () => ({
    type: 'execution',
    outputs: [],
    execution_count: 1,
    duration_millis: 5,
  })),
  cancelKernelExecution: vi.fn(async () => {}),
  listenKernelStream: vi.fn(async () => () => {}),
  resetKernel: vi.fn(async () => {}),
  sendKernelInput: vi.fn(async () => {}),
  kernelStatus: vi.fn(async () => ({
    type: 'status',
    status: 'idle',
    execution_count: 0,
  })),
}));
vi.mock('../../lib/diagnostics', () => ({
  parseExecutionTimeout: vi.fn(() => ({ value: 1000 })),
}));

import { useKernelExecution } from '../useKernelExecution';
import * as ks from '../../lib/kernelService';
import * as dg from '../../lib/diagnostics';

describe('useKernelExecution', () => {
  const setNotebook: Dispatch<SetStateAction<Notebook>> = vi.fn();
  const setStatus = vi.fn();
  const cell = (cell_type: NotebookCell['cell_type'] = 'code'): NotebookCell => ({
    id: '1',
    cell_type,
    source: 'x',
    execution_count: null,
    outputs: [],
    metadata: {},
  });

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(dg.parseExecutionTimeout).mockReturnValue({ value: 1000, error: null });
    vi.mocked(ks.executeSource).mockResolvedValue({
      type: 'execution',
      outputs: [],
      execution_count: 1,
      duration_millis: 5,
    });
    vi.mocked(ks.hasInterruptedOutput).mockReturnValue(false);
    vi.mocked(ks.cancelKernelExecution).mockResolvedValue(undefined);
  });

  it('non-code cell', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    const r = await result.current.executeCell(cell('markdown'));
    expect(r).toBe(true);
  });

  it('success path', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    let r = false;
    await act(async () => {
      r = await result.current.executeCell(cell());
    });
    expect(r).toBe(true);
    expect(setStatus).toHaveBeenCalled();
  });

  it('timeout error path', async () => {
    vi.mocked(dg.parseExecutionTimeout).mockReturnValue({ value: null, error: 'bad' });
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: 'x', setStatus }),
    );
    let r = false;
    await act(async () => {
      r = await result.current.executeCell(cell());
    });
    expect(r).toBe(false);
  });

  it('execute throws', async () => {
    vi.mocked(ks.executeSource).mockRejectedValue(new Error('boom'));
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    let r = false;
    await act(async () => {
      r = await result.current.executeCell(cell());
    });
    expect(r).toBe(false);
    expect(setStatus).toHaveBeenCalledWith('boom');
  });

  it('interrupted output', async () => {
    vi.mocked(ks.hasInterruptedOutput).mockReturnValue(true);
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    let r = false;
    await act(async () => {
      r = await result.current.executeCell(cell());
    });
    expect(r).toBe(false);
  });

  it('error output halts with failure status', async () => {
    vi.mocked(ks.executeSource).mockResolvedValue({
      type: 'execution',
      outputs: [
        {
          type: 'error',
          name: 'CompilationError',
          message: 'expected expression\nmore',
          traceback: [],
        },
      ],
      execution_count: 1,
      duration_millis: 7,
    });
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    let r = true;
    await act(async () => {
      r = await result.current.executeCell(cell());
    });
    expect(r).toBe(false);
    expect(setStatus).toHaveBeenCalledWith(
      'Failed in 7 ms — CompilationError: expected expression',
    );
  });

  it('executeCells stops at first failing cell and reports it', async () => {
    const failing = { ...cell(), id: 'bad' };
    const ok = { ...cell(), id: 'ok' };
    vi.mocked(ks.executeSource)
      .mockResolvedValueOnce({
        type: 'execution',
        outputs: [{ type: 'error', name: 'RuntimeError', message: 'boom', traceback: [] }],
        execution_count: 1,
        duration_millis: 3,
      })
      .mockResolvedValue({
        type: 'execution',
        outputs: [],
        execution_count: 2,
        duration_millis: 3,
      });
    const onExecutionHalted = vi.fn();
    const { result } = renderHook(() =>
      useKernelExecution({
        sessionId: 's',
        setNotebook,
        timeout: '1s',
        setStatus,
        onExecutionHalted,
      }),
    );
    await act(async () => {
      await result.current.executeCells([failing, ok]);
    });
    expect(ks.executeSource).toHaveBeenCalledOnce();
    expect(onExecutionHalted).toHaveBeenCalledWith(failing);
  });

  it('reports per-cell progress during batch execution', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    await act(async () => {
      await result.current.executeCells([
        { ...cell(), id: 'a' },
        { ...cell(), id: 'b' },
      ]);
    });
    expect(setStatus).toHaveBeenCalledWith('Running cell 1 of 2');
    expect(setStatus).toHaveBeenCalledWith('Running cell 2 of 2');
    expect(setStatus).toHaveBeenCalledWith('Executed 2 code cells');
  });

  it('executeCells', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    await act(async () => {
      await result.current.executeCells([cell()]);
    });
    expect(setStatus).toHaveBeenCalled();
  });

  it('restart', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    await act(async () => {
      await result.current.restart();
    });
    expect(ks.resetKernel).toHaveBeenCalled();
  });

  it('cancel', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    await act(async () => {
      await result.current.cancelExecution();
    });
    expect(setStatus).toHaveBeenCalledWith('Cancelling');
  });

  it('cancel throws', async () => {
    vi.mocked(ks.cancelKernelExecution).mockRejectedValue(new Error('no'));
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    await act(async () => {
      await result.current.cancelExecution();
    });
    expect(setStatus).toHaveBeenCalledWith('no');
  });

  it('routes python cells to the python language', async () => {
    const pythonCell = { ...cell(), metadata: { language: 'python' } };
    const { result } = renderHook(() =>
      useKernelExecution({
        sessionId: 's',
        notebookLanguage: 'rust',
        setNotebook,
        timeout: '1s',
        setStatus,
      }),
    );
    await act(async () => {
      await result.current.executeCell(pythonCell);
    });
    expect(ks.executeSource).toHaveBeenCalledWith('s', 'x', 1000, 'python');
  });

  it('falls back to the notebook language for unmarked cells', async () => {
    const { result } = renderHook(() =>
      useKernelExecution({
        sessionId: 's',
        notebookLanguage: 'python',
        setNotebook,
        timeout: '1s',
        setStatus,
      }),
    );
    await act(async () => {
      await result.current.executeCell(cell());
    });
    expect(ks.executeSource).toHaveBeenCalledWith('s', 'x', 1000, 'python');
  });

  it('prevents overlapping executions', async () => {
    let resolveExecution:
      ((value: Awaited<ReturnType<typeof ks.executeSource>>) => void) | undefined;
    vi.mocked(ks.executeSource).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveExecution = resolve;
        }),
    );
    const { result } = renderHook(() =>
      useKernelExecution({ sessionId: 's', setNotebook, timeout: '1s', setStatus }),
    );
    const first = result.current.executeCell(cell());
    await waitFor(() => expect(result.current.isExecuting).toBe(true));
    let secondResolved: boolean | undefined;
    const second = result.current.executeCell(cell()).then((ok) => {
      secondResolved = ok;
      return ok;
    });
    await act(async () => {
      await Promise.resolve();
    });
    expect(secondResolved).toBeUndefined();
    expect(ks.executeSource).toHaveBeenCalledOnce();
    await act(async () => {
      resolveExecution?.({
        type: 'execution',
        outputs: [],
        execution_count: 1,
        duration_millis: 1,
      });
      await first;
      await second;
    });
    expect(secondResolved).toBe(true);
    expect(ks.executeSource).toHaveBeenCalledTimes(2);
  });
});
