import { useState } from 'react';

export function useNotebookUiState() {
  const [status, setStatus] = useState('Idle');
  const [lastCommand, setLastCommand] = useState('');
  const [executionTimeout, setExecutionTimeout] = useState('');

  return {
    status,
    setStatus,
    lastCommand,
    setLastCommand,
    executionTimeout,
    setExecutionTimeout,
  };
}
