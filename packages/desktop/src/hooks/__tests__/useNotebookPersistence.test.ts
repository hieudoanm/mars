import { act, renderHook, waitFor } from '@testing-library/react';

const {
  openDialog,
  saveDialog,
  configure,
  exportFile,
  exportHtmlMock,
  exportPdfMock,
  exportIpynbMock,
  importIpynbMock,
  loadFile,
  mtime,
  renameFile,
  saveFile,
  closeWindow,
  closeListener,
} = vi.hoisted(() => ({
  openDialog: vi.fn(),
  saveDialog: vi.fn(),
  configure: vi.fn(),
  exportFile: vi.fn(),
  exportHtmlMock: vi.fn(),
  exportPdfMock: vi.fn(),
  exportIpynbMock: vi.fn(),
  importIpynbMock: vi.fn(),
  loadFile: vi.fn(),
  mtime: vi.fn(),
  renameFile: vi.fn(),
  saveFile: vi.fn(),
  closeWindow: vi.fn(),
  closeListener: vi.fn(),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    onCloseRequested: closeListener,
    close: closeWindow,
  }),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openDialog, save: saveDialog }));
vi.mock('../../lib/kernelService', () => ({
  configureKernel: configure,
  exportHtmlFile: exportHtmlMock,
  exportIpynbFile: exportIpynbMock,
  exportMarkdownFile: exportFile,
  exportPdfFile: exportPdfMock,
  importNotebookIpynb: importIpynbMock,
  listenKernelStream: vi.fn(async () => () => {}),
  loadNotebookFile: loadFile,
  notebookFileMtime: mtime,
  renameNotebookFile: renameFile,
  saveNotebookFile: saveFile,
}));

import { useNotebookPersistence } from '../useNotebookPersistence';
import type { Notebook } from '../../types/types';

const initial: Notebook = {
  format_version: 1,
  metadata: { language: 'rust' },
  cells: [
    { id: 'a', cell_type: 'code', source: '1', execution_count: null, outputs: [], metadata: {} },
  ],
};
const loaded: Notebook = { ...initial, metadata: { language: 'rust', title: 'Loaded' } };
// Markdown-only notebook: notebookHasCode is false, so it loads without a trust prompt.
const markdownOnly: Notebook = {
  ...loaded,
  cells: [
    {
      id: 'm',
      cell_type: 'markdown',
      source: '# notes',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
};
const markdownContent = JSON.stringify(markdownOnly);

const setup = (data = initial, notify?: (message: string, tone?: string) => void) => {
  const setNotebook = vi.fn();
  const setStatus = vi.fn();
  const resetHistory = vi.fn();
  const hook = renderHook(
    ({ notebook }) =>
      useNotebookPersistence({
        sessionId: 's',
        notebook,
        setNotebook,
        setStatus,
        notify,
        resetHistory,
      }),
    { initialProps: { notebook: data } },
  );
  return { ...hook, setNotebook, setStatus, resetHistory };
};

describe('useNotebookPersistence', () => {
  beforeEach(() => {
    vi.stubGlobal('__TAURI_INTERNALS__', undefined);
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    openDialog.mockReset();
    saveDialog.mockReset();
    configure.mockReset();
    exportFile.mockReset();
    exportHtmlMock.mockReset();
    exportPdfMock.mockReset();
    exportIpynbMock.mockReset();
    importIpynbMock.mockReset();
    loadFile.mockReset();
    mtime.mockReset();
    mtime.mockResolvedValue(null);
    renameFile.mockReset();
    saveFile.mockReset();
    closeWindow.mockReset();
    closeListener.mockReset();
    saveFile.mockResolvedValue(undefined);
    renameFile.mockResolvedValue(undefined);
    configure.mockResolvedValue(undefined);
    exportFile.mockResolvedValue(undefined);
  });

  it('saves to a chosen path, tracks dirty state, and exports Markdown', async () => {
    saveDialog.mockResolvedValueOnce('/books/test.mars').mockResolvedValueOnce('/books/test.md');
    const { result, setStatus } = setup();
    await act(async () => expect(await result.current.saveNotebook()).toBe(true));
    expect(result.current.filePath).toBe('/books/test.mars');
    expect(result.current.isDirty).toBe(false);
    expect(saveFile).toHaveBeenCalledWith('/books/test.mars', JSON.stringify(initial));
    expect(setStatus).toHaveBeenCalledWith('Saved');
    await act(async () => expect(await result.current.exportMarkdown()).toBe(true));
    expect(exportFile).toHaveBeenCalledWith('/books/test.md', JSON.stringify(initial));
  });

  it('renames the saved notebook file and validates names', async () => {
    saveDialog.mockResolvedValue('/books/test.mars');
    const { result, setStatus } = setup();
    await act(async () => result.current.saveNotebook());

    await act(async () => expect(await result.current.renameFile('analysis')).toBe(true));
    expect(renameFile).toHaveBeenCalledWith('/books/test.mars', '/books/analysis.mars');
    expect(result.current.filePath).toBe('/books/analysis.mars');
    expect(setStatus).toHaveBeenCalledWith('Renamed');

    await act(async () => expect(await result.current.renameFile('../escape')).toBe(false));
    expect(setStatus).toHaveBeenCalledWith('Invalid file name');
    expect(renameFile).toHaveBeenCalledTimes(1);

    renameFile.mockRejectedValueOnce(new Error('permission denied'));
    await act(async () => expect(await result.current.renameFile('denied')).toBe(false));
    expect(setStatus).toHaveBeenCalledWith('permission denied');
  });

  it('requires a saved file before renaming', async () => {
    const { result, setStatus } = setup();
    await act(async () => expect(await result.current.renameFile('analysis')).toBe(false));
    expect(renameFile).not.toHaveBeenCalled();
    expect(setStatus).toHaveBeenCalledWith('Save the notebook before renaming the file');
  });

  it('handles canceled dialogs and save/export failures', async () => {
    saveDialog
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce('/fail.mars')
      .mockResolvedValueOnce('/fail.md');
    const { result, setStatus } = setup();
    await act(async () => expect(await result.current.saveNotebook()).toBe(false));
    saveFile.mockRejectedValueOnce(new Error('disk full'));
    await act(async () => expect(await result.current.saveNotebook()).toBe(false));
    expect(setStatus).toHaveBeenCalledWith('disk full');
    exportFile.mockRejectedValueOnce('bad export');
    await act(async () => expect(await result.current.exportMarkdown()).toBe(false));
    expect(setStatus).toHaveBeenCalledWith('Markdown export failed');
  });

  it('opens clean notebooks and defers opening while dirty', async () => {
    openDialog.mockResolvedValueOnce('/books/loaded.mars');
    loadFile.mockResolvedValue({ content: '{notebook}', notebook: loaded });
    const clean = setup();
    await act(async () => clean.result.current.openNotebook());
    expect(configure).toHaveBeenCalledWith('s', '{notebook}', '/books');
    expect(clean.setNotebook).toHaveBeenCalledWith(loaded);
    expect(clean.resetHistory).toHaveBeenCalledWith(loaded);
    expect(clean.result.current.filePath).toBe('/books/loaded.mars');

    openDialog.mockResolvedValueOnce('/books/next.mars');
    const dirty = setup();
    dirty.rerender({
      notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'edited' }] },
    });
    await act(async () => dirty.result.current.openNotebook());
    expect(dirty.result.current.pendingOpenPath).toBe('/books/next.mars');
    dirty.result.current.discardAndOpen();
    await waitFor(() => expect(loadFile).toHaveBeenCalledWith('/books/next.mars'));
  });

  it('saves before opening, and closes with or without saving', async () => {
    const { result } = setup();
    act(() => result.current.setPendingOpenPath('/books/next.mars'));
    saveDialog.mockResolvedValue('/books/current.mars');
    loadFile.mockResolvedValue({ content: '{}', notebook: loaded });
    await act(async () => result.current.saveAndOpen());
    expect(result.current.filePath).toBe('/books/next.mars');

    await act(async () => result.current.discardAndClose());
    expect(closeWindow).toHaveBeenCalledOnce();
    saveDialog.mockResolvedValue('/books/current.mars');
    await act(async () => result.current.saveAndClose());
    expect(closeWindow).toHaveBeenCalledTimes(2);
  });

  it('autosaves dirty changes after the debounce and cancels on cleanup', async () => {
    vi.useFakeTimers();
    const { result, rerender, setStatus } = setup();
    saveDialog.mockResolvedValue('/books/autosave.mars');
    await act(async () => result.current.saveNotebook());
    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'edited' }] } });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(saveFile).toHaveBeenCalledTimes(2);
    expect(setStatus).toHaveBeenLastCalledWith('Autosaved');
    vi.useRealTimers();
  });

  it('pauses autosave when the file changes on disk', async () => {
    vi.useFakeTimers();
    mtime.mockResolvedValue(1000);
    const { result, rerender, setStatus } = setup();
    saveDialog.mockResolvedValue('/books/autosave.mars');
    await act(async () => result.current.saveNotebook());
    mtime.mockResolvedValue(2000);
    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'edited' }] } });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(saveFile).toHaveBeenCalledTimes(1);
    expect(setStatus).toHaveBeenLastCalledWith('File changed on disk; autosave paused');
    vi.useRealTimers();
  });

  it('queues the latest snapshot while an autosave is still running', async () => {
    vi.useFakeTimers();
    let finishFirstSave: (() => void) | undefined;
    const { result, rerender } = setup();
    saveDialog.mockResolvedValue('/books/autosave.mars');
    await act(async () => result.current.saveNotebook());
    saveFile.mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finishFirstSave = resolve;
        }),
    );
    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'first' }] } });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'latest' }] } });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    await act(async () => {
      finishFirstSave?.();
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(saveFile).toHaveBeenCalledTimes(3);
    expect(saveFile).toHaveBeenLastCalledWith(
      '/books/autosave.mars',
      expect.stringContaining('latest'),
    );
    vi.useRealTimers();
  });

  it('prompts for trust before loading an untrusted notebook with code', async () => {
    const content = JSON.stringify(initial);
    loadFile.mockResolvedValue({ content, notebook: loaded });
    const { result, setStatus } = setup();
    await act(async () => result.current.openNotebookAtPath('/tmp/untrusted.mars'));
    expect(result.current.pendingTrustPath).toBe('/tmp/untrusted.mars');
    expect(setStatus).toHaveBeenCalledWith('Review notebook trust before executing');
    expect(configure).not.toHaveBeenCalled();
  });

  it('trustAndLoad applies the pending notebook and cancelTrust aborts', async () => {
    const content = JSON.stringify(loaded);
    loadFile.mockResolvedValue({ content, notebook: loaded });
    const { result, setNotebook, setStatus, rerender } = setup();
    await act(async () => result.current.openNotebookAtPath('/tmp/untrusted.mars'));

    await act(async () => result.current.trustAndLoad());
    expect(setNotebook).toHaveBeenCalledWith(loaded);
    expect(result.current.pendingTrustPath).toBeNull();
    expect(result.current.filePath).toBe('/tmp/untrusted.mars');
    expect(setStatus).toHaveBeenCalledWith('Opened');

    // second untrusted load → cancel path (rerender so the loaded notebook is clean)
    rerender({ notebook: loaded });
    await act(async () => result.current.openNotebookAtPath('/tmp/other.mars'));
    expect(result.current.pendingTrustPath).toBe('/tmp/other.mars');
    act(() => result.current.cancelTrust());
    expect(result.current.pendingTrustPath).toBeNull();
    expect(setStatus).toHaveBeenLastCalledWith('Open cancelled');
  });

  it('loads trusted notebooks without prompting', async () => {
    const { trustNotebook } = await import('../../lib/trust');
    trustNotebook('/tmp/trusted.mars');
    const content = JSON.stringify(initial);
    loadFile.mockResolvedValue({ content, notebook: loaded });
    const { result, setNotebook } = setup();
    await act(async () => result.current.openNotebookAtPath('/tmp/trusted.mars'));
    expect(result.current.pendingTrustPath).toBeNull();
    expect(setNotebook).toHaveBeenCalledWith(loaded);
  });

  it('openNotebook uses the file dialog and defers while dirty', async () => {
    loadFile.mockResolvedValue({ content: markdownContent, notebook: markdownOnly });
    const { result, rerender } = setup();
    openDialog.mockResolvedValueOnce('/tmp/clean.mars');
    await act(async () => result.current.openNotebook());
    expect(loadFile).toHaveBeenCalledWith('/tmp/clean.mars');
    expect(result.current.filePath).toBe('/tmp/clean.mars');

    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'dirty' }] } });
    openDialog.mockResolvedValueOnce('/tmp/next.mars');
    await act(async () => result.current.openNotebook());
    expect(result.current.pendingOpenPath).toBe('/tmp/next.mars');
    expect(loadFile).toHaveBeenCalledTimes(1);
  });

  it('importIpynb routes through the Jupyter dialog and import command', async () => {
    const { result, setStatus } = setup();
    openDialog.mockResolvedValueOnce('/tmp/book.ipynb');
    importIpynbMock.mockResolvedValue({ content: '{}', notebook: loaded });
    await act(async () => result.current.importIpynb());
    expect(importIpynbMock).toHaveBeenCalledWith('/tmp/book.ipynb');
    expect(result.current.filePath).toBeNull();
    expect(setStatus).toHaveBeenCalledWith('Imported .ipynb');
  });

  it('exports HTML, PDF, and ipynb through the save dialog', async () => {
    const { result } = setup();
    saveDialog
      .mockResolvedValueOnce('/books/out.html')
      .mockResolvedValueOnce('/books/out.pdf')
      .mockResolvedValueOnce('/books/out.ipynb');
    await act(async () => expect(await result.current.exportHtml()).toBe(true));
    expect(exportHtmlMock).toHaveBeenCalledWith('/books/out.html', JSON.stringify(initial));
    await act(async () => expect(await result.current.exportPdf()).toBe(true));
    expect(exportPdfMock).toHaveBeenCalledWith('/books/out.pdf', JSON.stringify(initial));
    await act(async () => expect(await result.current.exportIpynb()).toBe(true));
    expect(exportIpynbMock).toHaveBeenCalledWith('/books/out.ipynb', JSON.stringify(initial));
  });

  it('exportFile returns false when the dialog is canceled', async () => {
    const { result } = setup();
    saveDialog.mockResolvedValueOnce(null);
    await act(async () => expect(await result.current.exportHtml()).toBe(false));
    expect(exportHtmlMock).not.toHaveBeenCalled();
  });

  it('discardAndOpen loads the pending path without saving', async () => {
    loadFile.mockResolvedValue({ content: markdownContent, notebook: markdownOnly });
    const { result, rerender } = setup();
    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'dirty' }] } });
    await act(async () => result.current.openNotebookAtPath('/tmp/next.mars'));
    expect(result.current.pendingOpenPath).toBe('/tmp/next.mars');
    await act(async () => result.current.discardAndOpen());
    expect(loadFile).toHaveBeenCalledWith('/tmp/next.mars');
    expect(result.current.pendingOpenPath).toBeNull();
  });

  it('saveAndOpen saves first, then loads the pending path', async () => {
    loadFile.mockResolvedValue({ content: markdownContent, notebook: markdownOnly });
    saveDialog.mockResolvedValue('/books/current.mars');
    const { result, rerender } = setup();
    rerender({ notebook: { ...initial, cells: [{ ...initial.cells[0], source: 'dirty' }] } });
    await act(async () => result.current.openNotebookAtPath('/tmp/next.mars'));
    await act(async () => result.current.saveAndOpen());
    expect(saveFile).toHaveBeenCalled();
    expect(loadFile).toHaveBeenCalledWith('/tmp/next.mars');
    expect(result.current.filePath).toBe('/tmp/next.mars');
  });

  it('reloadFromDisk reloads the current file path', async () => {
    const content = JSON.stringify(initial);
    loadFile.mockResolvedValue({ content, notebook: loaded });
    saveDialog.mockResolvedValue('/books/current.mars');
    const { result } = setup();
    await act(async () => result.current.saveNotebook());
    loadFile.mockClear();
    await act(async () => result.current.reloadFromDisk());
    expect(loadFile).toHaveBeenCalledWith('/books/current.mars');
  });

  it('handles close requests while dirty and clean', async () => {
    (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    let handler: ((event: { preventDefault: () => void }) => void) | undefined;
    closeListener.mockImplementation(async (h: typeof handler) => {
      handler = h;
      return vi.fn();
    });
    loadFile.mockResolvedValue({ content: markdownContent, notebook: markdownOnly });
    const { result, rerender } = setup();
    await act(async () => {
      await result.current.openNotebookAtPath('/tmp/a.mars');
    });
    expect(result.current.filePath).toBe('/tmp/a.mars');

    // sync the prop with the loaded notebook → clean
    rerender({ notebook: markdownOnly });
    expect(result.current.isDirty).toBe(false);
    const cleanPrevent = vi.fn();
    act(() => handler?.({ preventDefault: cleanPrevent }));
    expect(cleanPrevent).not.toHaveBeenCalled();
    expect(result.current.closeRequested).toBe(false);

    // dirty close request → prevented + closeRequested
    rerender({
      notebook: { ...markdownOnly, metadata: { ...markdownOnly.metadata, title: 'edited' } },
    });
    expect(result.current.isDirty).toBe(true);
    const dirtyPrevent = vi.fn();
    act(() => handler?.({ preventDefault: dirtyPrevent }));
    expect(dirtyPrevent).toHaveBeenCalled();
    expect(result.current.closeRequested).toBe(true);
  });

  it('reports autosave failures through status and notify', async () => {
    vi.useFakeTimers();
    try {
      const notify = vi.fn();
      loadFile.mockResolvedValue({ content: markdownContent, notebook: markdownOnly });
      saveFile.mockRejectedValue(new Error('disk full'));
      const { result, rerender, setStatus } = setup(initial, notify);
      await act(async () => {
        await result.current.openNotebookAtPath('/tmp/a.mars');
      });
      rerender({
        notebook: { ...markdownOnly, metadata: { ...markdownOnly.metadata, title: 'edited' } },
      });
      await act(async () => {
        vi.advanceTimersByTime(1500);
        await Promise.resolve();
        await Promise.resolve();
      });
      expect(setStatus).toHaveBeenCalledWith('disk full');
      expect(notify).toHaveBeenCalledWith('disk full', 'error');
    } finally {
      vi.useRealTimers();
    }
  });

  it('reloadFromDisk is a no-op without a file path', async () => {
    const { result } = setup();
    await act(async () => result.current.reloadFromDisk());
    expect(loadFile).not.toHaveBeenCalled();
  });
});
