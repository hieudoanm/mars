import { fireEvent, renderHook } from '@testing-library/react';
import { useKeyboardShortcuts } from '../useKeyboardShortcuts';
import type { NotebookCell } from '../../lib/types';

const cells: NotebookCell[] = [
  { id: 'a', cell_type: 'code', source: 'a', execution_count: null, outputs: [], metadata: {} },
  { id: 'b', cell_type: 'markdown', source: 'b', execution_count: null, outputs: [], metadata: {} },
];

function setup(overrides: Partial<Parameters<typeof useKeyboardShortcuts>[0]> = {}) {
  const actions = {
    setMode: vi.fn(),
    setSelectedId: vi.fn(),
    setLastCommand: vi.fn(),
    saveNotebook: vi.fn(async () => true),
    undo: vi.fn(),
    redo: vi.fn(),
    executeCell: vi.fn(async () => true),
    executeAll: vi.fn(),
    insertCell: vi.fn(),
    deleteSelected: vi.fn(),
    convertSelected: vi.fn(),
    moveSelected: vi.fn(),
    duplicateSelected: vi.fn(),
  };
  const options = {
    cells,
    selectedIndex: 0,
    mode: 'command' as const,
    lastCommand: '',
    ...actions,
    ...overrides,
  };
  const hook = renderHook(() => useKeyboardShortcuts(options));
  return { ...hook, actions };
}

describe('useKeyboardShortcuts', () => {
  it('handles save, undo, redo, and escape shortcuts', () => {
    const { actions } = setup();
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });
    fireEvent.keyDown(window, { key: 'z', metaKey: true });
    fireEvent.keyDown(window, { key: 'z', ctrlKey: true, shiftKey: true });
    fireEvent.keyDown(window, { key: 'y', ctrlKey: true });
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(actions.saveNotebook).toHaveBeenCalledOnce();
    expect(actions.undo).toHaveBeenCalledOnce();
    expect(actions.redo).toHaveBeenCalledTimes(2);
    expect(actions.setMode).toHaveBeenCalledWith('command');
  });

  it('executes the selected cell and inserts below for notebook shortcuts', () => {
    const { actions } = setup();
    fireEvent.keyDown(window, { key: 'Enter', shiftKey: true });
    expect(actions.executeCell).toHaveBeenCalledWith(cells[0]);
    expect(actions.setSelectedId).toHaveBeenCalledWith('b');
    fireEvent.keyDown(window, { key: 'Enter', ctrlKey: true });
    fireEvent.keyDown(window, { key: 'Enter', altKey: true });
    fireEvent.keyDown(window, { key: 'F6' });
    expect(actions.executeCell).toHaveBeenCalledTimes(3);
    expect(actions.insertCell).toHaveBeenCalledWith(1, 'code');
    expect(actions.executeAll).toHaveBeenCalledOnce();
  });

  it('supports command mode cell operations and ignores editor typing', () => {
    const { actions } = setup();
    for (const key of ['a', 'b', 'm', 'y', 'j', 'k', 'c']) {
      fireEvent.keyDown(window, { key });
    }
    expect(actions.insertCell).toHaveBeenNthCalledWith(1, 0, 'code');
    expect(actions.insertCell).toHaveBeenNthCalledWith(2, 1, 'code');
    expect(actions.convertSelected).toHaveBeenNthCalledWith(1, 'markdown');
    expect(actions.convertSelected).toHaveBeenNthCalledWith(2, 'code');
    expect(actions.moveSelected).toHaveBeenNthCalledWith(1, 1);
    expect(actions.moveSelected).toHaveBeenNthCalledWith(2, -1);
    expect(actions.duplicateSelected).toHaveBeenCalledOnce();

    fireEvent.keyDown(window, { key: 'd' });
    expect(actions.deleteSelected).not.toHaveBeenCalled();
    const second = setup({ lastCommand: 'd' });
    fireEvent.keyDown(window, { key: 'd' });
    expect(second.actions.deleteSelected).toHaveBeenCalledOnce();
    second.unmount();
    const editor = document.createElement('div');
    editor.className = 'cm-editor';
    document.body.append(editor);
    fireEvent.keyDown(editor, { key: 'a', bubbles: true });
    expect(actions.insertCell).toHaveBeenCalledTimes(2);
    editor.remove();
  });

  it('does nothing for navigation commands without a selected cell and removes listener', () => {
    const { actions, unmount } = setup({ selectedIndex: -1, cells: [] });
    fireEvent.keyDown(window, { key: 'Enter', shiftKey: true });
    expect(actions.executeCell).not.toHaveBeenCalled();
    unmount();
    fireEvent.keyDown(window, { key: 'a' });
    expect(actions.insertCell).not.toHaveBeenCalled();
  });
});
