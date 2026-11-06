import { act, renderHook, waitFor } from '@testing-library/react';

const {
  openDialog,
  saveDialog,
  configure,
  exportFile,
  loadFile,
  renameFile,
  saveFile,
  closeWindow,
  closeListener,
} = vi.hoisted(() => ({
  openDialog: vi.fn(),
  saveDialog: vi.fn(),
  configure: vi.fn(),
  exportFile: vi.fn(),
  loadFile: vi.fn(),
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
  exportMarkdownFile: exportFile,
  loadNotebookFile: loadFile,
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

const setup = (data = initial) => {
  const setNotebook = vi.fn();
  const setStatus = vi.fn();
  const resetHistory = vi.fn();
  const hook = renderHook(
    ({ notebook }) =>
      useNotebookPersistence({
        notebook,
        setNotebook,
        setStatus,
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
    loadFile.mockReset();
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
    expect(configure).toHaveBeenCalledWith('{notebook}');
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
});
