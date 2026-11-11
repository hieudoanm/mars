const { invoke, listen } = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen }));

import {
  cancelKernelExecution,
  configureKernel,
  createDirectory,
  createNotebookCheckpoint,
  createNotebookFile,
  deletePath,
  diagnoseCell,
  executeSource,
  exportHtmlFile,
  exportIpynbFile,
  exportMarkdownFile,
  exportPdfFile,
  hasInterruptedOutput,
  importNotebookIpynb,
  inspectVariables,
  kernelStatus,
  listenKernelStream,
  listNotebookCheckpoints,
  listNotebookFiles,
  loadNotebookFile,
  lspCodeActions,
  lspCompletion,
  lspDefinition,
  lspDiagnostics,
  lspHover,
  lspInlayHints,
  lspSemanticTokens,
  lspSignatureHelp,
  lspSync,
  notebookFileMtime,
  openInTerminal,
  openPathExternal,
  readNotebookCheckpoint,
  renameNotebookFile,
  resetKernel,
  restoreNotebookCheckpoint,
  saveNotebookFile,
  sendKernelInput,
  writeTextFile,
} from '../kernelService';
import type { KernelResponse, KernelStatusResponse, Notebook } from '../../types/types';

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

  it('kernel lifecycle and query commands forward session and language', async () => {
    const status: KernelStatusResponse = {
      type: 'status',
      status: 'ready',
    } as KernelStatusResponse;
    invoke.mockResolvedValue(status);
    await expect(kernelStatus('s1', 'python')).resolves.toEqual(status);
    expect(invoke).toHaveBeenLastCalledWith('kernel_status', { session: 's1', language: 'python' });

    invoke.mockResolvedValue([]);
    await inspectVariables('s1');
    expect(invoke).toHaveBeenLastCalledWith('inspect_variables', {
      session: 's1',
      language: 'rust',
    });

    invoke.mockResolvedValue([]);
    await diagnoseCell('s1', 'src');
    expect(invoke).toHaveBeenLastCalledWith('diagnose_cell', {
      session: 's1',
      source: 'src',
      language: 'rust',
    });

    await sendKernelInput('s1', 'answer');
    expect(invoke).toHaveBeenLastCalledWith('send_kernel_input', {
      session: 's1',
      text: 'answer',
      language: 'rust',
    });

    await cancelKernelExecution('s1');
    expect(invoke).toHaveBeenLastCalledWith('cancel_execution', { session: 's1' });
    await resetKernel('s1', 'python');
    expect(invoke).toHaveBeenLastCalledWith('reset_kernel', { session: 's1', language: 'python' });
    await configureKernel('s1', 'content');
    expect(invoke).toHaveBeenLastCalledWith('configure_kernel', {
      session: 's1',
      content: 'content',
    });
  });

  it('kernelStatus rejects non-status responses', async () => {
    invoke.mockResolvedValueOnce({ type: 'error', message: 'boom' });
    await expect(kernelStatus('s')).rejects.toThrow('boom');
    invoke.mockResolvedValueOnce({
      type: 'execution',
      outputs: [],
      execution_count: 1,
      duration_millis: 1,
    });
    await expect(kernelStatus('s')).rejects.toThrow('execution');
  });

  it('lsp commands wrap requests in a request envelope', async () => {
    invoke.mockResolvedValue(null);
    await lspSync('s', [{ id: 'c1', cell_type: 'code', source: 'x' }], { serde: '1' });
    expect(invoke).toHaveBeenLastCalledWith('lsp_sync', {
      request: {
        session: 's',
        cells: [{ id: 'c1', cell_type: 'code', source: 'x' }],
        dependencies: { serde: '1' },
      },
    });

    await lspHover('s', 'c1', 2, 3);
    expect(invoke).toHaveBeenLastCalledWith('lsp_hover', {
      request: { session: 's', cellId: 'c1', line: 2, character: 3 },
    });
    await lspCompletion('s', 'c1', 2, 3);
    expect(invoke).toHaveBeenLastCalledWith('lsp_completion', {
      request: { session: 's', cellId: 'c1', line: 2, character: 3 },
    });
    await lspSignatureHelp('s', 'c1', 2, 3);
    expect(invoke).toHaveBeenLastCalledWith('lsp_signature_help', {
      request: { session: 's', cellId: 'c1', line: 2, character: 3 },
    });
    await lspDefinition('s', 'c1', 2, 3);
    expect(invoke).toHaveBeenLastCalledWith('lsp_definition', {
      request: { session: 's', cellId: 'c1', line: 2, character: 3 },
    });
    await lspCodeActions('s', 'c1', 2, 3);
    expect(invoke).toHaveBeenLastCalledWith('lsp_code_actions', {
      request: { session: 's', cellId: 'c1', line: 2, character: 3 },
    });
    await lspDiagnostics('s');
    expect(invoke).toHaveBeenLastCalledWith('lsp_diagnostics', { session: 's' });
    await lspInlayHints('s');
    expect(invoke).toHaveBeenLastCalledWith('lsp_inlay_hints', { session: 's' });
    await lspSemanticTokens('s');
    expect(invoke).toHaveBeenLastCalledWith('lsp_semantic_tokens', { session: 's' });
  });

  it('export and file-management commands forward path and content', async () => {
    await exportHtmlFile('/tmp/a.html', '<p>x</p>');
    expect(invoke).toHaveBeenLastCalledWith('export_notebook_html', {
      path: '/tmp/a.html',
      content: '<p>x</p>',
    });
    await exportPdfFile('/tmp/a.pdf', 'pdf');
    expect(invoke).toHaveBeenLastCalledWith('export_notebook_pdf', {
      path: '/tmp/a.pdf',
      content: 'pdf',
    });
    await exportMarkdownFile('/tmp/a.md', '# hi');
    expect(invoke).toHaveBeenLastCalledWith('export_notebook_markdown', {
      path: '/tmp/a.md',
      content: '# hi',
    });
    await saveNotebookFile('/tmp/a.mars', '{}');
    expect(invoke).toHaveBeenLastCalledWith('save_notebook', {
      path: '/tmp/a.mars',
      content: '{}',
    });
    await notebookFileMtime('/tmp/a.mars');
    expect(invoke).toHaveBeenLastCalledWith('notebook_file_mtime', { path: '/tmp/a.mars' });
    await renameNotebookFile('/tmp/a.mars', '/tmp/b.mars');
    expect(invoke).toHaveBeenLastCalledWith('rename_notebook_file', {
      oldPath: '/tmp/a.mars',
      newPath: '/tmp/b.mars',
    });
    await createNotebookFile('/tmp/a.mars');
    expect(invoke).toHaveBeenLastCalledWith('create_notebook_file', { path: '/tmp/a.mars' });
    await createDirectory('/tmp/dir');
    expect(invoke).toHaveBeenLastCalledWith('create_directory', { path: '/tmp/dir' });
    await deletePath('/tmp/a');
    expect(invoke).toHaveBeenLastCalledWith('delete_path', { path: '/tmp/a' });
    await writeTextFile('/tmp/a.txt', 'data');
    expect(invoke).toHaveBeenLastCalledWith('write_text_file', {
      path: '/tmp/a.txt',
      content: 'data',
    });
    await openInTerminal('/tmp/a.mars');
    expect(invoke).toHaveBeenLastCalledWith('open_in_terminal', { path: '/tmp/a.mars' });
    await openPathExternal('/tmp/main.rs');
    expect(invoke).toHaveBeenLastCalledWith('open_path_external', { path: '/tmp/main.rs' });
    await listNotebookFiles('/tmp');
    expect(invoke).toHaveBeenLastCalledWith('list_notebook_files', { path: '/tmp' });
    await exportIpynbFile('/tmp/a.ipynb', '{}');
    expect(invoke).toHaveBeenLastCalledWith('export_notebook_ipynb', {
      path: '/tmp/a.ipynb',
      content: '{}',
    });
  });

  it('checkpoint commands forward path and checkpoint path', async () => {
    invoke.mockResolvedValue('cp-1');
    await expect(createNotebookCheckpoint('/tmp/a.mars')).resolves.toBe('cp-1');
    expect(invoke).toHaveBeenLastCalledWith('create_notebook_checkpoint', { path: '/tmp/a.mars' });

    invoke.mockResolvedValue(['cp-1', 'cp-2']);
    await expect(listNotebookCheckpoints('/tmp/a.mars')).resolves.toEqual(['cp-1', 'cp-2']);
    expect(invoke).toHaveBeenLastCalledWith('list_notebook_checkpoints', { path: '/tmp/a.mars' });

    invoke.mockResolvedValue('{}');
    await expect(readNotebookCheckpoint('/tmp/a.mars', 'cp-1')).resolves.toBe('{}');
    expect(invoke).toHaveBeenLastCalledWith('read_notebook_checkpoint', {
      path: '/tmp/a.mars',
      checkpointPath: 'cp-1',
    });

    await restoreNotebookCheckpoint('/tmp/a.mars', 'cp-1');
    expect(invoke).toHaveBeenLastCalledWith('restore_notebook_checkpoint', {
      path: '/tmp/a.mars',
      checkpointPath: 'cp-1',
    });
  });

  it('listenKernelStream subscribes to kernel-stream events', async () => {
    (window as unknown as Record<string, unknown>)['__TAURI_INTERNALS__'] = {};
    const unlisten = vi.fn();
    listen.mockResolvedValue(unlisten);
    const handler = vi.fn();
    const stop = await listenKernelStream(handler);
    expect(listen).toHaveBeenCalledWith('kernel-stream', expect.any(Function));
    const callback = listen.mock.calls[0][1] as (e: { payload: unknown }) => void;
    callback({ payload: { type: 'phase' } });
    expect(handler).toHaveBeenCalledWith({ type: 'phase' });
    stop();
    expect(unlisten).toHaveBeenCalled();
  });
});
