import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { Diagnostic } from '@codemirror/lint';
import type { NotebookCell } from '../types/types';
import { diagnoseCell, inspectVariables, lspDiagnostics } from '../lib/kernelService';
import { liveDiagnosticsFor, lspDiagnosticsFor } from '../lib/diagnostics';
import { isRunnableCodeCell } from '../lib/cellLanguage';
import type { KernelVariable } from '../types/types';

type UseNotebookInspectionOptions = {
  sessionId: string;
  cells: NotebookCell[];
  selectedIndex: number;
  executionCount: number;
};

export const useNotebookInspection = ({
  sessionId,
  cells,
  selectedIndex,
  executionCount,
}: UseNotebookInspectionOptions) => {
  const [liveVariables, setLiveVariables] = useState<KernelVariable[]>([]);
  const [preDiagnostics, setPreDiagnostics] = useState<Record<string, Diagnostic[]>>({});
  const [lspDiagnosticsByCell, setLspDiagnosticsByCell] = useState<Record<string, Diagnostic[]>>(
    {},
  );

  const refreshVariables = useCallback(() => {
    void inspectVariables(sessionId)
      .then(setLiveVariables)
      .catch(() => setLiveVariables([]));
  }, [sessionId]);

  useEffect(() => {
    if (executionCount === 0) {
      setLiveVariables([]);
      return;
    }
    const timer = window.setTimeout(refreshVariables, 400);
    return () => window.clearTimeout(timer);
  }, [executionCount, refreshVariables]);

  const selectedCell = cells[selectedIndex];
  const selectedSource =
    selectedCell && isRunnableCodeCell(selectedCell) ? selectedCell.source : null;
  useEffect(() => {
    if (!selectedCell || selectedSource === null) {
      setPreDiagnostics({});
      return;
    }
    const cellId = selectedCell.id;
    const timer = window.setTimeout(() => {
      void diagnoseCell(sessionId, selectedSource)
        .then((report) => {
          const own = report.filter((item) => item.cellIndex === executionCount);
          setPreDiagnostics((current) => ({
            ...current,
            [cellId]: liveDiagnosticsFor(selectedSource, own),
          }));
        })
        .catch(() => {});
    }, 700);
    return () => window.clearTimeout(timer);
  }, [sessionId, selectedCell?.id, selectedSource, executionCount]);

  const cellsRef = useRef(cells);
  cellsRef.current = cells;
  useEffect(() => {
    let disposed = false;
    const poll = () => {
      void lspDiagnostics(sessionId)
        .then((items) => {
          if (disposed) return;
          const byCell: Record<string, typeof items> = {};
          for (const item of items) {
            (byCell[item.cellId] ??= []).push(item);
          }
          const mapped: Record<string, Diagnostic[]> = {};
          for (const cell of cellsRef.current) {
            const own = byCell[cell.id];
            if (own?.length) mapped[cell.id] = lspDiagnosticsFor(cell.source, own);
          }
          setLspDiagnosticsByCell(mapped);
        })
        .catch(() => {});
    };
    poll();
    const timer = window.setInterval(poll, 4000);
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, [sessionId]);

  const problems = useMemo(
    () =>
      cells.flatMap((cell, cellIndex) =>
        [...(preDiagnostics[cell.id] ?? []), ...(lspDiagnosticsByCell[cell.id] ?? [])].map(
          (item) => ({
            cellId: cell.id,
            cellIndex,
            severity: item.severity,
            message: item.message,
          }),
        ),
      ),
    [cells, preDiagnostics, lspDiagnosticsByCell],
  );

  return {
    liveVariables,
    refreshVariables,
    preDiagnostics,
    lspDiagnosticsByCell,
    problems,
  };
};
