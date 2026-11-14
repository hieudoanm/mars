import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { CommMessage } from '../types/types';
import type {
  CellDiagnostic,
  KernelResponse,
  KernelStatusResponse,
  KernelStreamMessage,
  KernelVariable,
  LspCellDiagnostic,
  Notebook,
  Output,
} from '../types/types';
import { parseNotebookJson } from './notebookOperations';

export type FileExplorerEntry = {
  name: string;
  path: string;
  isDirectory: boolean;
  children: FileExplorerEntry[];
};

export const executeSource = async (
  session: string,
  source: string,
  timeoutMillis: number | null,
  language = 'rust',
): Promise<Extract<KernelResponse, { type: 'execution' }>> => {
  const response =
    timeoutMillis === null
      ? await invoke<KernelResponse>('execute_cell', { session, source, language })
      : await invoke<KernelResponse>('execute_cell_with_timeout', {
          session,
          source,
          timeoutMillis,
          language,
        });
  if (response.type !== 'execution') {
    throw new Error(
      response.type === 'error'
        ? response.message
        : response.type === 'status' || response.type === 'reset'
          ? response.status
          : 'unexpected kernel response',
    );
  }
  return response;
};

export const cancelKernelExecution = async (session: string): Promise<void> => {
  await invoke('cancel_execution', { session });
};

export const resetKernel = async (session: string, language = 'rust'): Promise<void> => {
  await invoke('reset_kernel', { session, language });
};

export const kernelStatus = async (
  session: string,
  language = 'rust',
): Promise<KernelStatusResponse> => {
  const response = await invoke<KernelResponse>('kernel_status', { session, language });
  if (response.type !== 'status') {
    throw new Error(response.type === 'error' ? response.message : response.type);
  }
  return response;
};

export const inspectVariables = async (
  session: string,
  language = 'rust',
): Promise<KernelVariable[]> => {
  return invoke<KernelVariable[]>('inspect_variables', { session, language });
};

export const diagnoseCell = async (
  session: string,
  source: string,
  language = 'rust',
): Promise<CellDiagnostic[]> => {
  return invoke<CellDiagnostic[]>('diagnose_cell', { session, source, language });
};

export const sendKernelInput = async (
  session: string,
  text: string,
  language = 'rust',
): Promise<void> => {
  await invoke('send_kernel_input', { session, text, language });
};

export type LspSyncCell = {
  id: string;
  cell_type: string;
  source: string;
};

export const lspSync = async (
  session: string,
  cells: LspSyncCell[],
  dependencies: Record<string, string>,
): Promise<void> => {
  await invoke('lsp_sync', { request: { session, cells, dependencies } });
};

export type LspHoverResult = {
  contents?: { kind?: string; value?: string } | { kind?: string; value?: string }[];
};

export const lspHover = async (
  session: string,
  cellId: string,
  line: number,
  character: number,
): Promise<LspHoverResult | null> => {
  return invoke<LspHoverResult | null>('lsp_hover', {
    request: { session, cellId, line, character },
  });
};

export type LspCompletionItem = {
  label: string;
  kind?: number;
  detail?: string;
  insertText?: string;
};

export const lspCompletion = async (
  session: string,
  cellId: string,
  line: number,
  character: number,
): Promise<{ items?: LspCompletionItem[] } | LspCompletionItem[] | null> => {
  return invoke('lsp_completion', {
    request: { session, cellId, line, character },
  });
};

export const lspSignatureHelp = async (
  session: string,
  cellId: string,
  line: number,
  character: number,
): Promise<unknown> => {
  return invoke('lsp_signature_help', {
    request: { session, cellId, line, character },
  });
};

export type LspDefinitionTarget = {
  cellId: string;
  line: number;
  character: number;
};

export const lspDefinition = async (
  session: string,
  cellId: string,
  line: number,
  character: number,
): Promise<LspDefinitionTarget | null> => {
  return invoke<LspDefinitionTarget | null>('lsp_definition', {
    request: { session, cellId, line, character },
  });
};

export type LspCellTextEdit = {
  cellId: string;
  startLine: number;
  startCharacter: number;
  endLine: number;
  endCharacter: number;
  newText: string;
};

export type LspCodeAction = {
  title: string;
  kind?: string;
  edits: LspCellTextEdit[];
};

export const lspCodeActions = async (
  session: string,
  cellId: string,
  line: number,
  character: number,
): Promise<LspCodeAction[]> => {
  return invoke<LspCodeAction[]>('lsp_code_actions', {
    request: { session, cellId, line, character },
  });
};

export const lspDiagnostics = async (session: string): Promise<LspCellDiagnostic[]> => {
  return invoke<LspCellDiagnostic[]>('lsp_diagnostics', { session });
};

export const configureKernel = async (session: string, content: string): Promise<void> => {
  await invoke('configure_kernel', { session, content });
};

export const loadNotebookFile = async (
  path: string,
): Promise<{ content: string; notebook: Notebook }> => {
  const content = await invoke<string>('open_notebook', { path });
  return { content, notebook: parseNotebookJson(content) };
};

export const importNotebookIpynb = async (
  path: string,
): Promise<{ content: string; notebook: Notebook }> => {
  const content = await invoke<string>('import_notebook_ipynb', { path });
  return { content, notebook: parseNotebookJson(content) };
};

export const exportIpynbFile = async (path: string, content: string): Promise<void> => {
  await invoke('export_notebook_ipynb', { path, content });
};

export const exportHtmlFile = async (path: string, content: string): Promise<void> => {
  await invoke('export_notebook_html', { path, content });
};

export const exportPdfFile = async (path: string, content: string): Promise<void> => {
  await invoke('export_notebook_pdf', { path, content });
};

export const listenKernelStream = async (
  handler: (event: KernelStreamMessage) => void,
): Promise<() => void> => {
  if (!('__TAURI_INTERNALS__' in window)) return () => {};
  return listen<KernelStreamMessage>('kernel-stream', (event) => handler(event.payload));
};

export const saveNotebookFile = async (path: string, content: string): Promise<void> => {
  await invoke('save_notebook', { path, content });
};

export const notebookFileMtime = async (path: string): Promise<number | null> => {
  return invoke<number | null>('notebook_file_mtime', { path });
};

export const renameNotebookFile = async (oldPath: string, newPath: string): Promise<void> => {
  await invoke('rename_notebook_file', { oldPath, newPath });
};

export const createNotebookFile = async (path: string): Promise<void> => {
  await invoke('create_notebook_file', { path });
};

export const createDirectory = async (path: string): Promise<void> => {
  await invoke('create_directory', { path });
};

export const deletePath = async (path: string): Promise<void> => {
  await invoke('delete_path', { path });
};

export const writeTextFile = async (path: string, content: string): Promise<void> => {
  await invoke('write_text_file', { path, content });
};

export const openInTerminal = async (path: string): Promise<void> => {
  await invoke('open_in_terminal', { path });
};

export const openPathExternal = async (path: string): Promise<void> => {
  await invoke('open_path_external', { path });
};

export const exportMarkdownFile = async (path: string, content: string): Promise<void> => {
  await invoke('export_notebook_markdown', { path, content });
};

export const listNotebookFiles = async (path: string): Promise<FileExplorerEntry[]> => {
  return invoke<FileExplorerEntry[]>('list_notebook_files', { path });
};

export const hasInterruptedOutput = (outputs: Output[]): boolean => {
  return outputs.some((output) => output.type === 'error' && output.name === 'Interrupted');
};

export const createNotebookCheckpoint = async (path: string): Promise<string> => {
  return invoke<string>('create_notebook_checkpoint', { path });
};

export const listNotebookCheckpoints = async (path: string): Promise<string[]> => {
  return invoke<string[]>('list_notebook_checkpoints', { path });
};

export const readNotebookCheckpoint = async (
  path: string,
  checkpointPath: string,
): Promise<string> => {
  return invoke<string>('read_notebook_checkpoint', { path, checkpointPath });
};

export const restoreNotebookCheckpoint = async (
  path: string,
  checkpointPath: string,
): Promise<void> => {
  await invoke('restore_notebook_checkpoint', { path, checkpointPath });
};

export type LspInlayHint = {
  cellId: string;
  line: number;
  character: number;
  label: string;
  kind?: string;
  paddingLeft?: boolean;
  paddingRight?: boolean;
};

export const lspInlayHints = async (session: string): Promise<LspInlayHint[]> => {
  return invoke<LspInlayHint[]>('lsp_inlay_hints', { session });
};

export type LspSemanticToken = {
  cellId: string;
  line: number;
  character: number;
  length: number;
  tokenType: string;
};

export const lspSemanticTokens = async (session: string): Promise<LspSemanticToken[]> => {
  return invoke<LspSemanticToken[]>('lsp_semantic_tokens', { session });
};

/// Evaluates `application/javascript` notebook output in the webview,
/// mirroring Jupyter's execute-and-display semantics. Routed through the
/// native webview eval so the page CSP stays restrictive.
export const evalNotebookScript = async (script: string): Promise<void> => {
  await invoke('eval_output_script', { script });
};

/// Delivers a Jupyter-style comm message (`comm_msg`/`comm_close`) to the
/// kernel owning `commId` and returns the comm traffic published in reply.
export const sendKernelComm = async (
  session: string,
  commId: string,
  commType: string,
  data: Record<string, unknown>,
  language: string,
): Promise<CommMessage[]> => {
  const response = await invoke<KernelResponse>('send_kernel_comm', {
    session,
    commId,
    commType,
    data,
    language,
  });
  if (response.type === 'comm') return response.comms;
  return [];
};
