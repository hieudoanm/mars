import { useState } from 'react';
import { useToasts } from './useToasts';

export type NotebookDialogKey = 'search' | 'dependencies' | 'checkpoints' | 'rename';

export const useNotebookUiState = () => {
  const [status, setStatus] = useState('Idle');
  const { toasts, notify, dismissToast } = useToasts();
  const [lastCommand, setLastCommand] = useState('');
  const [executionTimeout, setExecutionTimeout] = useState('');
  const [collapsedCellIds, setCollapsedCellIds] = useState<ReadonlySet<string>>(new Set());
  const [dialogs, setDialogs] = useState<Record<NotebookDialogKey, boolean>>({
    search: false,
    dependencies: false,
    checkpoints: false,
    rename: false,
  });
  const setDialogOpen = (key: NotebookDialogKey, open: boolean) =>
    setDialogs((current) => ({ ...current, [key]: open }));

  return {
    status,
    setStatus,
    toasts,
    notify,
    dismissToast,
    lastCommand,
    setLastCommand,
    executionTimeout,
    setExecutionTimeout,
    collapsedCellIds,
    setCollapsedCellIds,
    dialogs,
    setDialogOpen,
  };
};
