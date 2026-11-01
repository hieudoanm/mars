import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import type { Dispatch, SetStateAction } from 'react';
import type { Notebook, NotebookCell } from '../../lib/types';

vi.mock('../../lib/kernelService', () => ({
  hasInterruptedOutput: vi.fn(() => false),
  executeSource: vi.fn(async () => ({ type: 'execution', outputs: [], execution_count: 1, duration_millis: 5 })),
  cancelKernelExecution: vi.fn(async () => {}),
  resetKernel: vi.fn(async () => {}),
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
    id: '1', cell_type, source: 'x', execution_count: null, outputs: [], metadata: {},
  });

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(dg.parseExecutionTimeout).mockReturnValue({ value: 1000, error: null });
    vi.mocked(ks.executeSource).mockResolvedValue({
      type: 'execution', outputs: [], execution_count: 1, duration_millis: 5,
    });
    vi.mocked(ks.hasInterruptedOutput).mockReturnValue(false);
    vi.mocked(ks.cancelKernelExecution).mockResolvedValue(undefined);
  });

  it('non-code cell', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    const r = await result.current.executeCell(cell('markdown'));
    expect(r).toBe(true);
  });

  it('success path', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    let r = false;
    await act(async () => { r = await result.current.executeCell(cell()); });
    expect(r).toBe(true);
    expect(setStatus).toHaveBeenCalled();
  });

  it('timeout error path', async () => {
    vi.mocked(dg.parseExecutionTimeout).mockReturnValue({ value: null, error: 'bad' });
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: 'x', setStatus }));
    let r = false;
    await act(async () => { r = await result.current.executeCell(cell()); });
    expect(r).toBe(false);
  });

  it('execute throws', async () => {
    vi.mocked(ks.executeSource).mockRejectedValue(new Error('boom'));
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    let r = false;
    await act(async () => { r = await result.current.executeCell(cell()); });
    expect(r).toBe(false);
    expect(setStatus).toHaveBeenCalledWith('boom');
  });

  it('interrupted output', async () => {
    vi.mocked(ks.hasInterruptedOutput).mockReturnValue(true);
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    let r = false;
    await act(async () => { r = await result.current.executeCell(cell()); });
    expect(r).toBe(false);
  });

  it('executeCells', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    await act(async () => { await result.current.executeCells([cell()]); });
    expect(setStatus).toHaveBeenCalled();
  });

  it('restart', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    await act(async () => { await result.current.restart(); });
    expect(ks.resetKernel).toHaveBeenCalled();
  });

  it('cancel', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    await act(async () => { await result.current.cancelExecution(); });
    expect(setStatus).toHaveBeenCalledWith('Cancelling');
  });

  it('cancel throws', async () => {
    vi.mocked(ks.cancelKernelExecution).mockRejectedValue(new Error('no'));
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    await act(async () => { await result.current.cancelExecution(); });
    expect(setStatus).toHaveBeenCalledWith('no');
  });
});
