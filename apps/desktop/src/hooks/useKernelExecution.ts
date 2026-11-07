import { useEffect, useRef, useState, type Dispatch, type SetStateAction } from 'react';
import type { Notebook, NotebookCell, Output } from '../types/types';
import {
  hasInterruptedOutput,
  executeSource,
  cancelKernelExecution,
  kernelStatus,
  listenKernelStream,
  resetKernel,
  sendKernelInput,
} from '../lib/kernelService';
import { parseExecutionTimeout } from '../lib/diagnostics';
import { parseCellDirectives } from '../lib/cellDirectives';
import { isRunnableCodeCell } from '../lib/cellLanguage';

type ExecutionOptions = {
  sessionId: string;
  setNotebook: Dispatch<SetStateAction<Notebook>>;
  timeout: string;
  setStatus: (status: string) => void;
  onExecutionHalted?: (cell: NotebookCell) => void;
  onCellDirectives?: (cell: NotebookCell, dependencies: [string, string][]) => void;
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

export type KernelInfo = {
  status: string;
  executionCount: number;
};

export type CellRunInfo = {
  status: 'success' | 'error' | 'interrupted';
  durationMs: number;
};

export const useKernelExecution = ({
  sessionId,
  setNotebook,
  timeout,
  setStatus,
  onExecutionHalted,
  onCellDirectives,
}: ExecutionOptions) => {
  const [isExecuting, setIsExecuting] = useState(false);
  const [activeCellId, setActiveCellId] = useState<string | null>(null);
  const [stream, setStream] = useState<KernelStream | null>(null);
  const [kernelInfo, setKernelInfo] = useState<KernelInfo | null>(null);
  const [pendingPrompt, setPendingPrompt] = useState<string | null>(null);
  const [queueLength, setQueueLength] = useState(0);
  const [runInfo, setRunInfo] = useState<Record<string, CellRunInfo>>({});
  const executionInFlight = useRef(false);
  const queueRef = useRef<{ cell: NotebookCell; label?: string; resolve: (ok: boolean) => void }[]>(
    [],
  );
  const drainingRef = useRef(false);

  useEffect(() => {
    let disposed = false;
    const poll = () => {
      void kernelStatus(sessionId)
        .then((response) => {
          if (!disposed) {
            setKernelInfo({ status: response.status, executionCount: response.execution_count });
          }
        })
        .catch(() => {});
    };
    poll();
    const timer = window.setInterval(poll, 5000);
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, [sessionId]);

  const executeCell = async (
    cell: NotebookCell,
    manageExecutionState = true,
    label?: string,
  ): Promise<boolean> => {
    if (cell.cell_type !== 'code') return true;
    if (manageExecutionState && executionInFlight.current) {
      setStatus('Queued');
      setQueueLength(queueRef.current.length + 1);
      return new Promise<boolean>((resolve) => {
        queueRef.current.push({ cell, label, resolve });
      });
    }
    if (manageExecutionState) executionInFlight.current = true;
    setStatus(label ? `Running ${label}` : 'Running');
    setActiveCellId(cell.id);
    if (manageExecutionState) setIsExecuting(true);
    try {
      const directives = parseCellDirectives(cell.source);
      if (directives.dependencies.length > 0) {
        onCellDirectives?.(cell, directives.dependencies);
      }
      const parsed = parseExecutionTimeout(timeout);
      if (parsed.error) {
        setStatus(parsed.error);
        return false;
      }
      const effectiveTimeout = directives.timeoutMillis ?? parsed.value;
      let stdout = '';
      let stderr = '';
      const unlisten = await listenKernelStream((message) => {
        if (message.session !== sessionId) return;
        if (message.type === 'phase') {
          const phase = message.phase === 'compiling' ? 'Compiling' : 'Running';
          setStatus(label ? `${phase} ${label}…` : `${phase}…`);
          return;
        }
        if (message.type === 'prompt') {
          setPendingPrompt(message.message);
          setStatus('Waiting for input');
          return;
        }
        if (message.stream === 'stdout') stdout += message.data;
        else stderr += message.data;
        setStream({ cellId: cell.id, stdout, stderr });
      });
      let response: Awaited<ReturnType<typeof executeSource>>;
      try {
        response = await executeSource(sessionId, cell.source, effectiveTimeout);
      } finally {
        unlisten();
        setStream(null);
        setPendingPrompt(null);
      }
      const interrupted = hasInterruptedOutput(response.outputs);
      const errorOutput = firstErrorOutput(response.outputs);
      setRunInfo((current) => ({
        ...current,
        [cell.id]: {
          status: interrupted ? 'interrupted' : errorOutput ? 'error' : 'success',
          durationMs: response.duration_millis,
        },
      }));
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
      void kernelStatus(sessionId)
        .then((info) =>
          setKernelInfo({ status: info.status, executionCount: info.execution_count }),
        )
        .catch(() => {});
      return !interrupted && errorOutput === undefined;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Execution failed');
      return false;
    } finally {
      if (manageExecutionState) {
        executionInFlight.current = false;
        setActiveCellId(null);
        setIsExecuting(false);
        drainQueue();
      }
    }
  };

  const drainQueue = () => {
    if (drainingRef.current || queueRef.current.length === 0) return;
    const next = queueRef.current.shift();
    setQueueLength(queueRef.current.length);
    if (!next) return;
    drainingRef.current = true;
    void executeCell(next.cell, true, next.label)
      .then(next.resolve)
      .finally(() => {
        drainingRef.current = false;
        drainQueue();
      });
  };

  const executeCells = async (cells: NotebookCell[]) => {
    if (executionInFlight.current) {
      setStatus('Execution already in progress');
      return;
    }
    executionInFlight.current = true;
    setIsExecuting(true);
    try {
      const runnable = cells.filter(isRunnableCodeCell);
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
      drainQueue();
    }
  };

  const restart = async () => {
    if (executionInFlight.current) {
      setStatus('Stop execution before restarting the runtime');
      return;
    }
    await resetKernel(sessionId);
    setNotebook((current) => ({
      ...current,
      cells: current.cells.map((cell) => ({ ...cell, execution_count: null, outputs: [] })),
    }));
    setRunInfo({});
    setStatus('Idle');
    void kernelStatus(sessionId)
      .then((info) => setKernelInfo({ status: info.status, executionCount: info.execution_count }))
      .catch(() => {});
  };

  const respondToPrompt = async (text: string) => {
    setPendingPrompt(null);
    try {
      await sendKernelInput(sessionId, text);
      setStatus('Running');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to send input');
    }
  };

  const cancelExecution = async () => {
    try {
      await cancelKernelExecution(sessionId);
      const queued = queueRef.current.splice(0);
      setQueueLength(0);
      for (const item of queued) item.resolve(false);
      setStatus('Cancelling');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Cancellation failed');
    }
  };

  return {
    isExecuting,
    queueLength,
    activeCellId,
    stream,
    kernelInfo,
    runInfo,
    executeCell,
    executeCells,
    restart,
    cancelExecution,
    pendingPrompt,
    respondToPrompt,
  };
};
