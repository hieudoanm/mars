import { invoke } from "@tauri-apps/api/core";
import type { KernelResponse, Notebook, Output } from "./types";

export async function executeSource(
  source: string,
  timeoutMillis: number | null,
): Promise<Extract<KernelResponse, { type: "execution" }>> {
  const response =
    timeoutMillis === null
      ? await invoke<KernelResponse>("execute_cell", { source })
      : await invoke<KernelResponse>("execute_cell_with_timeout", {
          source,
          timeoutMillis,
        });
  if (response.type !== "execution") {
    throw new Error(response.type === "error" ? response.message : response.status);
  }
  return response;
}

export async function cancelKernelExecution(): Promise<void> {
  await invoke("cancel_execution");
}

export async function resetKernel(): Promise<void> {
  await invoke("reset_kernel");
}

export async function configureKernel(content: string): Promise<void> {
  await invoke("configure_kernel", { content });
}

export async function loadNotebookFile(path: string): Promise<{ content: string; notebook: Notebook }> {
  const content = await invoke<string>("open_notebook", { path });
  return { content, notebook: JSON.parse(content) as Notebook };
}

export async function saveNotebookFile(path: string, content: string): Promise<void> {
  await invoke("save_notebook", { path, content });
}

export async function exportMarkdownFile(path: string, content: string): Promise<void> {
  await invoke("export_notebook_markdown", { path, content });
}

export function hasInterruptedOutput(outputs: Output[]): boolean {
  return outputs.some((output) => output.type === "error" && output.name === "Interrupted");
}
