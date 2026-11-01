import { useState, type Dispatch, type SetStateAction } from "react";
import type { Notebook, NotebookCell } from "./types";
import { hasInterruptedOutput, executeSource, cancelKernelExecution, resetKernel } from "./kernelService";
import { parseExecutionTimeout } from "./diagnostics";

type ExecutionOptions = {
  setNotebook: Dispatch<SetStateAction<Notebook>>;
  timeout: string;
  setStatus: (status: string) => void;
};

export function useKernelExecution({ setNotebook, timeout, setStatus }: ExecutionOptions) {
  const [isExecuting, setIsExecuting] = useState(false);

  const executeCell = async (cell: NotebookCell, manageExecutionState = true): Promise<boolean> => {
    if (cell.cell_type !== "code") return true;
    setStatus("Running");
    if (manageExecutionState) setIsExecuting(true);
    try {
      const parsed = parseExecutionTimeout(timeout);
      if (parsed.error) {
        setStatus(parsed.error);
        return false;
      }
      const response = await executeSource(cell.source, parsed.value);
      const interrupted = hasInterruptedOutput(response.outputs);
      setNotebook((current) => ({
        ...current,
        cells: current.cells.map((item) =>
          item.id === cell.id
            ? { ...item, outputs: response.outputs, execution_count: response.execution_count }
            : item,
        ),
      }));
      setStatus(interrupted ? `Interrupted after ${response.duration_millis} ms` : `Succeeded in ${response.duration_millis} ms`);
      return !interrupted;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Execution failed");
      return false;
    } finally {
      if (manageExecutionState) setIsExecuting(false);
    }
  };

  const executeCells = async (cells: NotebookCell[]) => {
    setStatus("Running");
    setIsExecuting(true);
    try {
      for (const cell of cells) {
        if (!(await executeCell(cell, false))) return;
      }
      setStatus(`Executed ${cells.filter((cell) => cell.cell_type === "code").length} code cells`);
    } finally {
      setIsExecuting(false);
    }
  };

  const restart = async () => {
    await resetKernel();
    setNotebook((current) => ({
      ...current,
      cells: current.cells.map((cell) => ({ ...cell, execution_count: null, outputs: [] })),
    }));
    setStatus("Idle");
  };

  const cancelExecution = async () => {
    try {
      await cancelKernelExecution();
      setStatus("Cancelling");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Cancellation failed");
    }
  };

  return { isExecuting, executeCell, executeCells, restart, cancelExecution };
}
