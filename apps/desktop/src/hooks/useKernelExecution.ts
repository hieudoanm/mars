import { useRef, useState, type Dispatch, type SetStateAction } from 'react';
import type { Notebook, NotebookCell, Output } from '../types/types';
import {
  hasInterruptedOutput,
  executeSource,
  cancelKernelExecution,
  listenKernelStream,
  resetKernel,
} from '../lib/kernelService';
import { parseExecutionTimeout } from '../lib/diagnostics';

type ExecutionOptions = {
  setNotebook: Dispatch<SetStateAction<Notebook>>;
  timeout: string;
  setStatus: (status: string) => void;
  onExecutionHalted?: (cell: NotebookCell) => void;
};

const firstErrorOutput = (outputs: Output[]): Extract<Output, { type: 'error' }> | undefined =>
  outputs.find((output): output is Extract<Output, { type: 'error' }> => output.type === 'error');

const summarizeError = (output: Extract<Output, { type: 'error' }>): string => {
  const firstLine = output.message.split('\n', 1)[0] ?? '';
  const detail = `${output.name}: ${firstLine}`.trim();
  return detail.length > 140 ? `${detail.slice(0, 137)}…` : detail;
};

export type KernelStream = {
  cellId: string;
  stdout: string;
  stderr: string;
};

export const useKernelExecution = ({
  setNotebook,
  timeout,
  setStatus,
  onExecutionHalted,
}: ExecutionOptions) => {
  const [isExecuting, setIsExecuting] = useState(false);
  const [activeCellId, setActiveCellId] = useState<string | null>(null);
  const [stream, setStream] = useState<KernelStream | null>(null);
  const executionInFlight = useRef(false);

  const executeCell = async (
    cell: NotebookCell,
    manageExecutionState = true,
    label?: string,
  ): Promise<boolean> => {
    if (cell.cell_type !== 'code') return true;
    if (manageExecutionState && executionInFlight.current) {
      setStatus('Execution already in progress');
      return false;
    }
    if (manageExecutionState) executionInFlight.current = true;
    setStatus(label ? `Running ${label}` : 'Running');
    setActiveCellId(cell.id);
    if (manageExecutionState) setIsExecuting(true);
    try {
      const parsed = parseExecutionTimeout(timeout);
      if (parsed.error) {
        setStatus(parsed.error);
        return false;
      }
      let stdout = '';
      let stderr = '';
      const unlisten = await listenKernelStream((event) => {
        if (event.type === 'phase') {
          const phase = event.phase === 'compiling' ? 'Compiling' : 'Running';
          setStatus(label ? `${phase} ${label}…` : `${phase}…`);
          return;
        }
        if (event.stream === 'stdout') stdout += event.data;
        else stderr += event.data;
        setStream({ cellId: cell.id, stdout, stderr });
      });
      let response: Awaited<ReturnType<typeof executeSource>>;
      try {
        response = await executeSource(cell.source, parsed.value);
      } finally {
        unlisten();
        setStream(null);
      }
      const interrupted = hasInterruptedOutput(response.outputs);
      const errorOutput = firstErrorOutput(response.outputs);
      setNotebook((current) => ({
        ...current,
        cells: current.cells.map((item) =>
          item.id === cell.id
            ? { ...item, outputs: response.outputs, execution_count: response.execution_count }
            : item,
        ),
      }));
      setStatus(
        interrupted
          ? `Interrupted after ${response.duration_millis} ms`
          : errorOutput
            ? `Failed in ${response.duration_millis} ms — ${summarizeError(errorOutput)}`
            : `Succeeded in ${response.duration_millis} ms`,
      );
      return !interrupted && errorOutput === undefined;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Execution failed');
      return false;
    } finally {
      if (manageExecutionState) {
        executionInFlight.current = false;
        setActiveCellId(null);
        setIsExecuting(false);
      }
    }
  };

  const executeCells = async (cells: NotebookCell[]) => {
    if (executionInFlight.current) {
      setStatus('Execution already in progress');
      return;
    }
    executionInFlight.current = true;
    setIsExecuting(true);
    try {
      const runnable = cells.filter((cell) => cell.cell_type === 'code');
      for (const [index, cell] of runnable.entries()) {
        const label = `cell ${index + 1} of ${runnable.length}`;
        if (!(await executeCell(cell, false, label))) {
          onExecutionHalted?.(cell);
          return;
        }
      }
      setStatus(`Executed ${runnable.length} code cells`);
    } finally {
      executionInFlight.current = false;
      setActiveCellId(null);
      setIsExecuting(false);
    }
  };

  const restart = async () => {
    if (executionInFlight.current) {
      setStatus('Stop execution before restarting the runtime');
      return;
    }
    await resetKernel();
    setNotebook((current) => ({
      ...current,
      cells: current.cells.map((cell) => ({ ...cell, execution_count: null, outputs: [] })),
    }));
    setStatus('Idle');
  };

  const cancelExecution = async () => {
    try {
      await cancelKernelExecution();
      setStatus('Cancelling');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Cancellation failed');
    }
  };

  return { isExecuting, activeCellId, stream, executeCell, executeCells, restart, cancelExecution };
};
