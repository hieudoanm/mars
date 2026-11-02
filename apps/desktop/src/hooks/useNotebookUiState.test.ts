import { renderHook, act } from '@testing-library/react';
import { useNotebookUiState } from './useNotebookUiState';

describe('useNotebookUiState', () => {
  it('defaults to Idle status', () => {
    const { result } = renderHook(() => useNotebookUiState());
    expect(result.current.status).toBe('Idle');
  });

  it('updates status', () => {
    const { result } = renderHook(() => useNotebookUiState());
    act(() => {
      result.current.setStatus('Running');
    });
    expect(result.current.status).toBe('Running');
  });
});
