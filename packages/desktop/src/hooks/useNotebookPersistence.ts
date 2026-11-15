import { getCurrentWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { Notebook } from '../types/types';
import {
  configureKernel,
  exportHtmlFile,
  exportIpynbFile,
  exportMarkdownFile,
  exportPdfFile,
  importNotebookIpynb,
  loadNotebookFile,
  notebookFileMtime,
  renameNotebookFile,
  saveNotebookFile,
} from '../lib/kernelService';
import { isNotebookTrusted, notebookHasCode, trustNotebook } from '../lib/trust';
import { renamedNotebookPath } from '../lib/filePath';
import { parseNotebookJson, serializeNotebook } from '../lib/notebookOperations';

type PersistenceOptions = {
  sessionId: string;
  notebook: Notebook;
  setNotebook: (notebook: Notebook) => void;
  setStatus: (status: string) => void;
  notify?: (message: string, tone?: 'info' | 'success' | 'error') => void;
  resetHistory: (notebook: Notebook) => void;
  initialPath?: string | null;
  onPathChange?: (path: string | null) => void;
};

export const useNotebookPersistence = ({
  sessionId,
  notebook,
  setNotebook,
  setStatus,
  notify,
  resetHistory,
  initialPath = null,
  onPathChange,
}: PersistenceOptions) => {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [savedSnapshot, setSavedSnapshot] = useState(() => serializeNotebook(notebook));
  const [pendingOpenPath, setPendingOpenPath] = useState<string | null>(null);
  const [closeRequested, setCloseRequested] = useState(false);
  const [pendingTrustPath, setPendingTrustPath] = useState<string | null>(null);
  const [pendingTrustContent, setPendingTrustContent] = useState<string | null>(null);
  const [pendingTrustIsIpynb, setPendingTrustIsIpynb] = useState(false);
  const autosaveInFlight = useRef<Promise<void> | null>(null);
  const autosaveQueuedSnapshot = useRef<string | null>(null);
  const fileStamp = useRef<number | null>(null);
  const allowClose = useRef(false);
  const snapshot = useMemo(() => serializeNotebook(notebook), [notebook]);
  const isDirty = snapshot !== savedSnapshot;

  useEffect(() => {
    onPathChange?.(filePath);
  }, [filePath, onPathChange]);

  const applyLoadedNotebook = async (
    path: string,
    isIpynb: boolean,
    loadedFile: { content: string; notebook: Notebook },
  ) => {
    await configureKernel(sessionId, loadedFile.content);
    setNotebook(loadedFile.notebook);
    resetHistory(loadedFile.notebook);
    setSavedSnapshot(isIpynb ? '' : serializeNotebook(loadedFile.notebook));
    setFilePath(isIpynb ? null : path);
    fileStamp.current = isIpynb ? null : await notebookFileMtime(path).catch(() => null);
    setStatus(isIpynb ? 'Imported .ipynb' : 'Opened');
    setPendingOpenPath(null);
  };

  const loadNotebook = async (path: string) => {
    try {
      const isIpynb = path.toLowerCase().endsWith('.ipynb');
      const loadedFile = isIpynb ? await importNotebookIpynb(path) : await loadNotebookFile(path);
      if (!isIpynb && notebookHasCode(loadedFile.content) && !isNotebookTrusted(path)) {
        setPendingTrustPath(path);
        setPendingTrustContent(loadedFile.content);
        setPendingTrustIsIpynb(isIpynb);
        setStatus('Review notebook trust before executing');
        return;
      }
      await applyLoadedNotebook(path, isIpynb, loadedFile);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Open failed');
      notify?.(error instanceof Error ? error.message : 'Open failed', 'error');
    }
  };

  const trustAndLoad = async () => {
    if (!pendingTrustPath || !pendingTrustContent) return;
    const path = pendingTrustPath;
    const isIpynb = pendingTrustIsIpynb;
    const content = pendingTrustContent;
    setPendingTrustPath(null);
    setPendingTrustContent(null);
    trustNotebook(path);
    try {
      const loadedFile = isIpynb
        ? await importNotebookIpynb(path)
        : { content, notebook: parseNotebookJson(content) };
      await applyLoadedNotebook(path, isIpynb, loadedFile);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Open failed');
      notify?.(error instanceof Error ? error.message : 'Open failed', 'error');
    }
  };

  const cancelTrust = () => {
    setPendingTrustPath(null);
    setPendingTrustContent(null);
    setStatus('Open cancelled');
  };

  const openNotebook = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: 'Project Mars', extensions: ['mars'] }],
      });
      if (!path || Array.isArray(path)) return;
      if (isDirty) setPendingOpenPath(path);
      else await loadNotebook(path);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Open failed');
      notify?.(error instanceof Error ? error.message : 'Open failed', 'error');
    }
  };

  const importIpynb = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: 'Jupyter', extensions: ['ipynb'] }],
      });
      if (!path || Array.isArray(path)) return;
      if (isDirty) setPendingOpenPath(path);
      else await loadNotebook(path);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Import failed');
      notify?.(error instanceof Error ? error.message : 'Import failed', 'error');
    }
  };

  const openNotebookAtPath = (path: string) => {
    if (isDirty) setPendingOpenPath(path);
    else void loadNotebook(path);
  };

  useEffect(() => {
    if (initialPath) void loadNotebook(initialPath);
  }, [initialPath]);

  const saveNotebook = async (): Promise<boolean> => {
    try {
      const path =
        filePath ??
        (await save({
          defaultPath: 'Untitled.mars',
          filters: [{ name: 'Project Mars', extensions: ['mars'] }],
        }));
      if (!path) return false;
      await saveNotebookFile(path, snapshot);
      fileStamp.current = await notebookFileMtime(path).catch(() => null);
      setFilePath(path);
      setSavedSnapshot(snapshot);
      setStatus('Saved');
      notify?.('Notebook saved', 'success');
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Save failed');
      notify?.(error instanceof Error ? error.message : 'Save failed', 'error');
      return false;
    }
  };

  const renameFile = async (fileName: string): Promise<boolean> => {
    if (!filePath) {
      setStatus('Save the notebook before renaming the file');
      return false;
    }
    const nextPath = renamedNotebookPath(filePath, fileName);
    if (!nextPath) {
      setStatus('Invalid file name');
      return false;
    }
    if (nextPath === filePath) return true;
    try {
      await renameNotebookFile(filePath, nextPath);
      fileStamp.current = null;
      setFilePath(nextPath);
      setSavedSnapshot(snapshot);
      setStatus('Renamed');
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Rename failed');
      notify?.(error instanceof Error ? error.message : 'Rename failed', 'error');
      return false;
    }
  };

  const exportFile = async (
    kind: 'markdown' | 'html' | 'pdf' | 'ipynb',
    extensions: string[],
    name: string,
    defaultPath: string,
  ): Promise<boolean> => {
    try {
      const path = await save({ defaultPath, filters: [{ name, extensions }] });
      if (!path) return false;
      if (kind === 'markdown') await exportMarkdownFile(path, snapshot);
      else if (kind === 'html') await exportHtmlFile(path, snapshot);
      else if (kind === 'pdf') await exportPdfFile(path, snapshot);
      else await exportIpynbFile(path, snapshot);
      setStatus(`Exported ${name}`);
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : `${name} export failed`);
      notify?.(error instanceof Error ? error.message : `${name} export failed`, 'error');
      return false;
    }
  };

  const exportMarkdown = () =>
    exportFile(
      'markdown',
      ['md'],
      'Markdown',
      filePath ? filePath.replace(/\.mars$/i, '.md') : 'Untitled.md',
    );
  const exportHtml = () =>
    exportFile(
      'html',
      ['html'],
      'HTML',
      filePath ? filePath.replace(/\.mars$/i, '.html') : 'Untitled.html',
    );
  const exportPdf = () =>
    exportFile(
      'pdf',
      ['pdf'],
      'PDF',
      filePath ? filePath.replace(/\.mars$/i, '.pdf') : 'Untitled.pdf',
    );
  const exportIpynb = () =>
    exportFile(
      'ipynb',
      ['ipynb'],
      'Jupyter',
      filePath ? filePath.replace(/\.mars$/i, '.ipynb') : 'Untitled.ipynb',
    );

  const autosave = (nextSnapshot: string) => {
    if (!filePath) return;
    autosaveQueuedSnapshot.current = nextSnapshot;
    if (autosaveInFlight.current) return;
    autosaveInFlight.current = (async () => {
      while (autosaveQueuedSnapshot.current) {
        const snapshotToSave = autosaveQueuedSnapshot.current;
        autosaveQueuedSnapshot.current = null;
        try {
          if (fileStamp.current !== null) {
            const current = await notebookFileMtime(filePath).catch(() => null);
            if (current !== fileStamp.current) {
              setStatus('File changed on disk; autosave paused');
              continue;
            }
          }
          await saveNotebookFile(filePath, snapshotToSave);
          fileStamp.current = await notebookFileMtime(filePath).catch(() => null);
          setSavedSnapshot(snapshotToSave);
          setStatus('Autosaved');
        } catch (error) {
          setStatus(error instanceof Error ? error.message : 'Autosave failed');
          notify?.(error instanceof Error ? error.message : 'Autosave failed', 'error');
        }
      }
      autosaveInFlight.current = null;
    })();
  };

  useEffect(() => {
    if (!filePath || !isDirty) return;
    const timer = window.setTimeout(() => {
      autosave(snapshot);
    }, 1000);
    return () => window.clearTimeout(timer);
  }, [filePath, isDirty, snapshot, savedSnapshot]);

  useEffect(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void getCurrentWindow()
      .onCloseRequested((event) => {
        if (allowClose.current || !isDirty) return;
        event.preventDefault();
        setCloseRequested(true);
      })
      .then((cleanup) => {
        if (disposed) cleanup();
        else unlisten = cleanup;
      })
      .catch((error: unknown) => {
        setStatus(error instanceof Error ? error.message : 'Close handling unavailable');
        notify?.(error instanceof Error ? error.message : 'Close handling unavailable', 'error');
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [isDirty]);

  const discardAndOpen = () => {
    if (pendingOpenPath) void loadNotebook(pendingOpenPath);
  };
  const saveAndOpen = async () => {
    if (pendingOpenPath && (await saveNotebook())) await loadNotebook(pendingOpenPath);
  };
  const discardAndClose = async () => {
    allowClose.current = true;
    await getCurrentWindow().close();
  };
  const saveAndClose = async () => {
    if (!(await saveNotebook())) return;
    allowClose.current = true;
    await getCurrentWindow().close();
  };

  return {
    filePath,
    isDirty,
    pendingOpenPath,
    closeRequested,
    pendingTrustPath,
    setPendingOpenPath,
    setCloseRequested,
    openNotebook,
    openNotebookAtPath,
    saveNotebook,
    renameFile,
    exportMarkdown,
    exportHtml,
    exportPdf,
    exportIpynb,
    importIpynb,
    trustAndLoad,
    cancelTrust,
    discardAndOpen,
    saveAndOpen,
    discardAndClose,
    saveAndClose,
    reloadFromDisk: () => (filePath ? loadNotebook(filePath) : Promise.resolve()),
  };
};
