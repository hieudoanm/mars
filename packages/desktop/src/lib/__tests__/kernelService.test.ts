const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

import {
  cancelKernelExecution,
  configureKernel,
  executeSource,
  exportIpynbFile,
  exportMarkdownFile,
  hasInterruptedOutput,
  importNotebookIpynb,
  listNotebookFiles,
  loadNotebookFile,
  renameNotebookFile,
  resetKernel,
  saveNotebookFile,
} from '../kernelService';
import type { KernelResponse, Notebook } from '../../types/types';

const notebook: Notebook = { format_version: 1, metadata: { language: 'rust' }, cells: [] };

describe('kernelService', () => {
  beforeEach(() => invoke.mockReset());

  it('executes with or without a timeout and validates kernel responses', async () => {
    const response: KernelResponse = {
      type: 'execution',
      outputs: [],
      execution_count: 1,
      duration_millis: 2,
    };
    invoke.mockResolvedValueOnce(response).mockResolvedValueOnce(response);
    await expect(executeSource('s', '1+1', null)).resolves.toEqual(response);
    expect(invoke).toHaveBeenNthCalledWith(1, 'execute_cell', {
      session: 's',
      source: '1+1',
      language: 'rust',
    });
    await executeSource('s', '2+2', 100);
    expect(invoke).toHaveBeenNthCalledWith(2, 'execute_cell_with_timeout', {
      session: 's',
      source: '2+2',
      timeoutMillis: 100,
      language: 'rust',
    });
    invoke.mockResolvedValueOnce({ type: 'error', message: 'compile error' });
    await expect(executeSource('s', 'bad', null)).rejects.toThrow('compile error');
    invoke.mockResolvedValueOnce({ type: 'status', status: 'busy', execution_count: 0 });
    await expect(executeSource('s', 'busy', null)).rejects.toThrow('busy');
  });

  it('forwards lifecycle and file operations to Tauri', async () => {
    invoke.mockResolvedValue(undefined);
    await cancelKernelExecution('s');
    await resetKernel('s');
    await configureKernel('s', 'cargo content');
    await saveNotebookFile('/book.mars', '{}');
    await renameNotebookFile('/book.mars', '/renamed.mars');
    await exportMarkdownFile('/book.md', '{}');
    await exportIpynbFile('/book.ipynb', '{}');
    await listNotebookFiles('/books');
    expect(invoke.mock.calls).toEqual([
      ['cancel_execution', { session: 's' }],
      ['reset_kernel', { session: 's', language: 'rust' }],
      ['configure_kernel', { session: 's', content: 'cargo content' }],
      ['save_notebook', { path: '/book.mars', content: '{}' }],
      ['rename_notebook_file', { oldPath: '/book.mars', newPath: '/renamed.mars' }],
      ['export_notebook_markdown', { path: '/book.md', content: '{}' }],
      ['export_notebook_ipynb', { path: '/book.ipynb', content: '{}' }],
      ['list_notebook_files', { path: '/books' }],
    ]);
  });

  it('imports an .ipynb file as a validated notebook', async () => {
    invoke.mockResolvedValueOnce(JSON.stringify(notebook));
    await expect(importNotebookIpynb('/book.ipynb')).resolves.toEqual({
      content: JSON.stringify(notebook),
      notebook,
    });
    expect(invoke).toHaveBeenCalledWith('import_notebook_ipynb', { path: '/book.ipynb' });
    invoke.mockResolvedValueOnce('{bad');
    await expect(importNotebookIpynb('/bad.ipynb')).rejects.toThrow('not valid JSON');
  });

  it('loads a notebook document and reports malformed JSON', async () => {
    invoke.mockResolvedValueOnce(JSON.stringify(notebook));
    await expect(loadNotebookFile('/book.mars')).resolves.toEqual({
      content: JSON.stringify(notebook),
      notebook,
    });
    invoke.mockResolvedValueOnce('{bad');
    await expect(loadNotebookFile('/bad.mars')).rejects.toThrow();
  });

  it('detects interrupted outputs', () => {
    expect(
      hasInterruptedOutput([{ type: 'error', name: 'Interrupted', message: '', traceback: [] }]),
    ).toBe(true);
    expect(hasInterruptedOutput([{ type: 'text', text: 'ok' }])).toBe(false);
  });
});
