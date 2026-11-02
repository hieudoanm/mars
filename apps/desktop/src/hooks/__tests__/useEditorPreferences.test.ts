import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useEditorPreferences } from '../useEditorPreferences';

describe('useEditorPreferences', () => {
  beforeEach(() => localStorage.clear());

  it('defaults', () => {
    const { result } = renderHook(() => useEditorPreferences());
    expect(result.current.fontSize).toBe(14);
    expect(result.current.preferences).toEqual({ autocomplete: true, folding: true });
  });

  it('reads stored', () => {
    localStorage.setItem('mars.editor.fontSize', '20');
    localStorage.setItem('mars.editor.preferences', JSON.stringify({ autocomplete: false, folding: true }));
    const { result } = renderHook(() => useEditorPreferences());
    expect(result.current.fontSize).toBe(20);
    expect(result.current.preferences.autocomplete).toBe(false);
  });

  it('invalid stored falls back', () => {
    localStorage.setItem('mars.editor.fontSize', '100');
    localStorage.setItem('mars.editor.preferences', '{bad');
    const { result } = renderHook(() => useEditorPreferences());
    expect(result.current.fontSize).toBe(14);
    expect(result.current.preferences).toEqual({ autocomplete: true, folding: true });
  });

  it('updateFontSize valid/invalid', () => {
    const { result } = renderHook(() => useEditorPreferences());
    act(() => result.current.updateFontSize(18));
    expect(result.current.fontSize).toBe(18);
    act(() => result.current.updateFontSize(5));
    expect(result.current.fontSize).toBe(18);
  });

  it('setPreferences persists', () => {
    const { result } = renderHook(() => useEditorPreferences());
    act(() => result.current.setPreferences({ autocomplete: false, folding: false }));
    expect(JSON.parse(localStorage.getItem('mars.editor.preferences')!)).toEqual({ autocomplete: false, folding: false });
  });
});
