import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useToasts } from '../useToasts';

describe('useToasts', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('adds toasts with ids and default tone', () => {
    const { result } = renderHook(() => useToasts());
    act(() => result.current.notify('hello'));
    expect(result.current.toasts).toEqual([{ id: 1, message: 'hello', tone: 'info' }]);
    act(() => result.current.notify('done', 'success'));
    expect(result.current.toasts).toHaveLength(2);
    expect(result.current.toasts[1].tone).toBe('success');
  });

  it('dismisses a toast manually', () => {
    const { result } = renderHook(() => useToasts());
    act(() => result.current.notify('one'));
    act(() => result.current.notify('two'));
    act(() => result.current.dismissToast(1));
    expect(result.current.toasts.map((t) => t.message)).toEqual(['two']);
  });

  it('auto-dismisses after the tone TTL', () => {
    const { result } = renderHook(() => useToasts());
    act(() => result.current.notify('quick', 'success'));
    act(() => result.current.notify('slow', 'error'));
    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.toasts.map((t) => t.message)).toEqual(['slow']);
    act(() => vi.advanceTimersByTime(5000));
    expect(result.current.toasts).toHaveLength(0);
  });

  it('caps the toast list at four', () => {
    const { result } = renderHook(() => useToasts());
    act(() => {
      for (let i = 0; i < 6; i++) result.current.notify(`t${i}`);
    });
    expect(result.current.toasts).toHaveLength(4);
    expect(result.current.toasts[0].message).toBe('t2');
  });
});
