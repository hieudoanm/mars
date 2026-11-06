import { getCurrentWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { Notebook } from '../types/types';
import {
  configureKernel,
  exportMarkdownFile,
  loadNotebookFile,
  renameNotebookFile,
  saveNotebookFile,
} from '../lib/kernelService';
import { renamedNotebookPath } from '../lib/filePath';
import { serializeNotebook } from '../lib/notebookOperations';

type PersistenceOptions = {
  notebook: Notebook;
  setNotebook: (notebook: Notebook) => void;
  setStatus: (status: string) => void;
  resetHistory: (notebook: Notebook) => void;
};

export const useNotebookPersistence = ({
  notebook,
  setNotebook,
  setStatus,
  resetHistory,
}: PersistenceOptions) => {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [savedSnapshot, setSavedSnapshot] = useState(() => serializeNotebook(notebook));
  const [pendingOpenPath, setPendingOpenPath] = useState<string | null>(null);
  const [closeRequested, setCloseRequested] = useState(false);
  const autosaveInFlight = useRef<Promise<void> | null>(null);
  const autosaveQueuedSnapshot = useRef<string | null>(null);
  const allowClose = useRef(false);
  const snapshot = useMemo(() => serializeNotebook(notebook), [notebook]);
  const isDirty = snapshot !== savedSnapshot;

  const saveNotebook = async (): Promise<boolean> => {
    try {
      const path =
        filePath ??
        (await save({
          defaultPath: 'Untitled.mars',
          filters: [{ name: 'Mars', extensions: ['mars'] }],
        }));
      if (!path) return false;
      await saveNotebookFile(path, snapshot);
      setFilePath(path);
      setSavedSnapshot(snapshot);
      setStatus('Saved');
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Save failed');
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
      setFilePath(nextPath);
      setStatus('Renamed');
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Rename failed');
      return false;
    }
  };

  const exportMarkdown = async (): Promise<boolean> => {
    try {
      const path = await save({
        defaultPath: filePath ? filePath.replace(/\.mars$/i, '.md') : 'Untitled.md',
        filters: [{ name: 'Markdown', extensions: ['md'] }],
      });
      if (!path) return false;
      await exportMarkdownFile(path, snapshot);
      setStatus('Exported Markdown');
      return true;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Markdown export failed');
      return false;
    }
  };

  const loadNotebook = async (path: string) => {
    try {
      const loadedFile = await loadNotebookFile(path);
      await configureKernel(loadedFile.content);
      setNotebook(loadedFile.notebook);
      resetHistory(loadedFile.notebook);
      setSavedSnapshot(serializeNotebook(loadedFile.notebook));
      setFilePath(path);
      setStatus('Opened');
      setPendingOpenPath(null);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Open failed');
    }
  };

  const openNotebook = async () => {
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: 'Mars', extensions: ['mars'] }],
      });
      if (!path || Array.isArray(path)) return;
      if (isDirty) setPendingOpenPath(path);
      else await loadNotebook(path);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Open failed');
    }
  };

  const openNotebookAtPath = (path: string) => {
    if (isDirty) setPendingOpenPath(path);
    else void loadNotebook(path);
  };

  const autosave = (nextSnapshot: string) => {
    if (!filePath) return;
    autosaveQueuedSnapshot.current = nextSnapshot;
    if (autosaveInFlight.current) return;
    autosaveInFlight.current = (async () => {
      while (autosaveQueuedSnapshot.current) {
        const snapshotToSave = autosaveQueuedSnapshot.current;
        autosaveQueuedSnapshot.current = null;
        try {
          await saveNotebookFile(filePath, snapshotToSave);
          setSavedSnapshot(snapshotToSave);
          setStatus('Autosaved');
        } catch (error) {
          setStatus(error instanceof Error ? error.message : 'Autosave failed');
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
    notebookSnapshot: snapshot,
    pendingOpenPath,
    closeRequested,
    setPendingOpenPath,
    setCloseRequested,
    openNotebook,
    openNotebookAtPath,
    saveNotebook,
    renameFile,
    exportMarkdown,
    discardAndOpen,
    saveAndOpen,
    discardAndClose,
    saveAndClose,
  };
};
