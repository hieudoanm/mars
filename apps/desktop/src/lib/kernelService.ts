import { invoke } from '@tauri-apps/api/core';
import type { KernelResponse, Notebook, Output } from '../types/types';

export const executeSource = async (
  source: string,
  timeoutMillis: number | null,
): Promise<Extract<KernelResponse, { type: 'execution' }>> => {
  const response =
    timeoutMillis === null
      ? await invoke<KernelResponse>('execute_cell', { source })
      : await invoke<KernelResponse>('execute_cell_with_timeout', {
          source,
          timeoutMillis,
        });
  if (response.type !== 'execution') {
    throw new Error(response.type === 'error' ? response.message : response.status);
  }
  return response;
};

export const cancelKernelExecution = async (): Promise<void> => {
  await invoke('cancel_execution');
};

export const resetKernel = async (): Promise<void> => {
  await invoke('reset_kernel');
};

export const configureKernel = async (content: string): Promise<void> => {
  await invoke('configure_kernel', { content });
};

export const loadNotebookFile = async (
  path: string,
): Promise<{ content: string; notebook: Notebook }> => {
  const content = await invoke<string>('open_notebook', { path });
  return { content, notebook: JSON.parse(content) as Notebook };
};

export const saveNotebookFile = async (path: string, content: string): Promise<void> => {
  await invoke('save_notebook', { path, content });
};

export const renameNotebookFile = async (oldPath: string, newPath: string): Promise<void> => {
  await invoke('rename_notebook_file', { oldPath, newPath });
};

export const exportMarkdownFile = async (path: string, content: string): Promise<void> => {
  await invoke('export_notebook_markdown', { path, content });
};

export const hasInterruptedOutput = (outputs: Output[]): boolean => {
  return outputs.some((output) => output.type === 'error' && output.name === 'Interrupted');
};
