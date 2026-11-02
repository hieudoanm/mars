import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';

vi.mock('../../lib/kernelService', () => ({
  hasInterruptedOutput: vi.fn(() => false),
  executeSource: vi.fn(async () => ({ outputs: [], execution_count: 1, duration_millis: 5 })),
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
  const setNotebook = vi.fn((fn) => fn({ cells: [{ id: '1', cell_type: 'code' }] }));
  const setStatus = vi.fn();

  beforeEach(() => vi.clearAllMocks());

  it('non-code cell', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    const r = await result.current.executeCell({ cell_type: 'markdown', id: '1', source: 'x', outputs: [] } as any);
    expect(r).toBe(true);
  });

  it('success path', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    const r = await result.current.executeCell({ cell_type: 'code', id: '1', source: 'x', outputs: [] } as any);
    expect(r).toBe(true);
    expect(setStatus).toHaveBeenCalled();
  });

  it('timeout error path', async () => {
    (dg.parseExecutionTimeout as any).mockReturnValue({ error: 'bad' });
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: 'x', setStatus }));
    const r = await result.current.executeCell({ cell_type: 'code', id: '1', source: 'x', outputs: [] } as any);
    expect(r).toBe(false);
  });

  it('execute throws', async () => {
    (dg.parseExecutionTimeout as any).mockReturnValue({ value: 1000 });
    (ks.executeSource as any).mockRejectedValue(new Error('boom'));
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    const r = await result.current.executeCell({ cell_type: 'code', id: '1', source: 'x', outputs: [] } as any);
    expect(r).toBe(false);
    expect(setStatus).toHaveBeenCalledWith('boom');
  });

  it('interrupted output', async () => {
    (ks.hasInterruptedOutput as any).mockReturnValue(true);
    (ks.executeSource as any).mockResolvedValue({ outputs: [], execution_count: 1, duration_millis: 5 });
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    const r = await result.current.executeCell({ cell_type: 'code', id: '1', source: 'x', outputs: [] } as any);
    expect(r).toBe(false);
  });

  it('executeCells', async () => {
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    await act(async () => { await result.current.executeCells([{ cell_type: 'code', id: '1', source: 'x', outputs: [] } as any]); });
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
    (ks.cancelKernelExecution as any).mockRejectedValue(new Error('no'));
    const { result } = renderHook(() => useKernelExecution({ setNotebook, timeout: '1s', setStatus }));
    await act(async () => { await result.current.cancelExecution(); });
    expect(setStatus).toHaveBeenCalledWith('no');
  });
});
