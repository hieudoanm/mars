import { renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useSessionRestore } from '../useSessionRestore';
import { readSessionState, writeSessionState } from '../../lib/workspace';

vi.mock('../../lib/workspace', () => ({
  readSessionState: vi.fn(),
  writeSessionState: vi.fn(),
}));

const readMock = vi.mocked(readSessionState);
const writeMock = vi.mocked(writeSessionState);

const setup = (filePath: string | null = '/notes/a.mars') => {
  const scrollRef = { current: document.createElement('div') };
  const setSelectedId = vi.fn();
  const setMode = vi.fn();
  const setCollapsedCellIds = vi.fn();
  const utils = renderHook(
    ({
      path,
      selectedId,
      collapsed,
    }: {
      path: string | null;
      selectedId: string;
      collapsed: ReadonlySet<string>;
    }) =>
      useSessionRestore({
        filePath: path,
        scrollRef,
        setSelectedId,
        setMode,
        collapsedCellIds: collapsed,
        setCollapsedCellIds,
        selectedId,
      }),
    { initialProps: { path: filePath, selectedId: 'c1', collapsed: new Set<string>() } },
  );
  return { ...utils, scrollRef, setSelectedId, setMode, setCollapsedCellIds };
};

describe('useSessionRestore', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    readMock.mockReset();
    writeMock.mockReset();
    window.requestAnimationFrame = (cb) => {
      cb(0);
      return 0;
    };
  });
  afterEach(() => vi.useRealTimers());

  it('does nothing without a file path', () => {
    setup(null);
    expect(readMock).not.toHaveBeenCalled();
    vi.advanceTimersByTime(5000);
    expect(writeMock).not.toHaveBeenCalled();
  });

  it('restores selection, collapsed cells, and scroll position', () => {
    readMock.mockReturnValue({
      scrollTop: 120,
      selectedCellId: 'c9',
      collapsedCellIds: ['c2'],
    });
    const { scrollRef, setSelectedId, setMode, setCollapsedCellIds } = setup();
    expect(setSelectedId).toHaveBeenCalledWith('c9');
    expect(setMode).toHaveBeenCalledWith('command');
    expect(setCollapsedCellIds).toHaveBeenCalledWith(new Set(['c2']));
    expect(scrollRef.current.scrollTop).toBe(120);
  });

  it('does not restore twice for the same path', () => {
    readMock.mockReturnValue({ scrollTop: 5, selectedCellId: 'c1', collapsedCellIds: [] });
    const { rerender, setSelectedId } = setup();
    setSelectedId.mockClear();
    rerender({ path: '/notes/a.mars', selectedId: 'c1', collapsed: new Set() });
    expect(setSelectedId).not.toHaveBeenCalled();
    expect(readMock).toHaveBeenCalledTimes(1);
  });

  it('persists session state on interval and unmount', () => {
    readMock.mockReturnValue(null);
    const { unmount, rerender } = setup();
    vi.advanceTimersByTime(2100);
    expect(writeMock).toHaveBeenCalledWith(
      '/notes/a.mars',
      expect.objectContaining({ selectedCellId: 'c1' }),
    );
    unmount();
    expect(writeMock).toHaveBeenCalledTimes(2);
    void rerender;
  });

  it('restores again when the path changes', () => {
    readMock.mockReturnValue({ scrollTop: 1, selectedCellId: 'x', collapsedCellIds: [] });
    const { rerender, setSelectedId } = setup('/a.mars');
    setSelectedId.mockClear();
    rerender({ path: '/b.mars', selectedId: 'c1', collapsed: new Set() });
    expect(setSelectedId).toHaveBeenCalledWith('x');
  });
});
