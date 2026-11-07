import { useEffect, useRef, type Dispatch, type RefObject, type SetStateAction } from 'react';
import { readSessionState, writeSessionState } from '../lib/workspace';

export type CellCollapseState = ReadonlySet<string>;

type UseSessionRestoreOptions = {
  filePath: string | null;
  scrollRef: RefObject<HTMLDivElement | null>;
  setSelectedId: (id: string) => void;
  setMode: (mode: 'edit' | 'command') => void;
  collapsedCellIds: CellCollapseState;
  setCollapsedCellIds: Dispatch<SetStateAction<CellCollapseState>>;
  selectedId: string;
};

export const useSessionRestore = ({
  filePath,
  scrollRef,
  setSelectedId,
  setMode,
  collapsedCellIds,
  setCollapsedCellIds,
  selectedId,
}: UseSessionRestoreOptions) => {
  const restoredPathRef = useRef<string | null>(null);

  useEffect(() => {
    if (!filePath || restoredPathRef.current === filePath) return;
    restoredPathRef.current = filePath;
    const state = readSessionState(filePath);
    if (!state) return;
    if (state.selectedCellId) {
      setSelectedId(state.selectedCellId);
      setMode('command');
    }
    if (state.collapsedCellIds?.length) {
      setCollapsedCellIds(new Set(state.collapsedCellIds));
    }
    window.requestAnimationFrame(() => {
      if (scrollRef.current) scrollRef.current.scrollTop = state.scrollTop;
    });
  }, [filePath, scrollRef, setSelectedId, setMode, setCollapsedCellIds]);

  useEffect(() => {
    if (!filePath) return;
    const container = scrollRef.current;
    const save = () => {
      writeSessionState(filePath, {
        scrollTop: container?.scrollTop ?? 0,
        selectedCellId: selectedId,
        collapsedCellIds: [...collapsedCellIds],
      });
    };
    const interval = window.setInterval(save, 2000);
    window.addEventListener('beforeunload', save);
    return () => {
      window.clearInterval(interval);
      window.removeEventListener('beforeunload', save);
      save();
    };
  }, [filePath, scrollRef, collapsedCellIds, selectedId]);
};
