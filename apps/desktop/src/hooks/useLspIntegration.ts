import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { NotebookCell } from '../types/types';
import {
  lspCodeActions,
  lspCompletion,
  lspDefinition,
  lspHover,
  lspInlayHints,
  lspSemanticTokens,
  lspSignatureHelp,
  lspSync,
  type LspCellTextEdit,
  type LspCodeAction,
  type LspCompletionItem,
  type LspDefinitionTarget,
  type LspInlayHint,
  type LspSemanticToken,
} from '../lib/kernelService';
import { lspHoverText, signatureHelpText } from '../lib/lsp';
import { isRunnableCodeCell } from '../lib/cellLanguage';
import { updateCompletionSources } from '../lib/rustCompletions';

export type LspHandlers = {
  completion: (
    cellId: string,
    line: number,
    character: number,
  ) => Promise<{ items?: LspCompletionItem[] } | LspCompletionItem[] | null>;
  signature: (cellId: string, line: number, character: number) => Promise<string | null>;
  definition: (
    cellId: string,
    line: number,
    character: number,
  ) => Promise<LspDefinitionTarget | null>;
  onDefinitionResult: (target: LspDefinitionTarget) => void;
  codeActions: (cellId: string, line: number, character: number) => Promise<LspCodeAction[]>;
  applyEdits?: (edits: LspCellTextEdit[]) => void;
};

type UseLspIntegrationOptions = {
  sessionId: string;
  cells: NotebookCell[];
  dependencies: Record<string, string> | undefined;
  onRevealCell: (cellId: string) => void;
};

export const useLspIntegration = ({
  sessionId,
  cells,
  dependencies,
  onRevealCell,
}: UseLspIntegrationOptions) => {
  const codeSources = useMemo(
    () => cells.filter(isRunnableCodeCell).map((cell) => cell.source),
    [cells],
  );

  useEffect(() => {
    updateCompletionSources(codeSources);
  }, [codeSources]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void lspSync(
        sessionId,
        cells.map((cell) => ({
          id: cell.id,
          cell_type: cell.cell_type,
          source: cell.source,
        })),
        dependencies ?? {},
      ).catch(() => {});
    }, 800);
    return () => window.clearTimeout(timer);
  }, [sessionId, cells, dependencies]);

  const handleHover = useCallback(
    (cellId: string, line: number, character: number) =>
      lspHover(sessionId, cellId, line, character)
        .then(lspHoverText)
        .catch(() => null),
    [sessionId],
  );

  const [pendingReveal, setPendingReveal] = useState<LspDefinitionTarget | null>(null);
  const [inlayHints, setInlayHints] = useState<LspInlayHint[]>([]);
  const [semanticTokens, setSemanticTokens] = useState<LspSemanticToken[]>([]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void lspInlayHints(sessionId)
        .then(setInlayHints)
        .catch(() => {});
      void lspSemanticTokens(sessionId)
        .then(setSemanticTokens)
        .catch(() => {});
    }, 1200);
    return () => window.clearTimeout(timer);
  }, [sessionId, cells]);
  const onRevealCellRef = useRef(onRevealCell);
  onRevealCellRef.current = onRevealCell;

  const handlers = useMemo<LspHandlers>(
    () => ({
      completion: (cellId, line, character) => lspCompletion(sessionId, cellId, line, character),
      signature: async (cellId, line, character) =>
        signatureHelpText(
          await lspSignatureHelp(sessionId, cellId, line, character).catch(() => null),
        ),
      definition: (cellId, line, character) =>
        lspDefinition(sessionId, cellId, line, character).catch(() => null),
      codeActions: (cellId, line, character) =>
        lspCodeActions(sessionId, cellId, line, character).catch(() => []),
      onDefinitionResult: (target) => {
        onRevealCellRef.current(target.cellId);
        setPendingReveal(target);
      },
    }),
    [sessionId],
  );

  const reveal = useCallback((target: LspDefinitionTarget) => {
    onRevealCellRef.current(target.cellId);
    setPendingReveal(target);
  }, []);

  return { handleHover, handlers, pendingReveal, reveal, inlayHints, semanticTokens };
};
