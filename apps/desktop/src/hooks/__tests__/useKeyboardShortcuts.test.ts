import { fireEvent, renderHook } from '@testing-library/react';
import { useKeyboardShortcuts } from '../useKeyboardShortcuts';
import type { NotebookCell } from '../../types/types';

const cells: NotebookCell[] = [
  { id: 'a', cell_type: 'code', source: 'a', execution_count: null, outputs: [], metadata: {} },
  { id: 'b', cell_type: 'markdown', source: 'b', execution_count: null, outputs: [], metadata: {} },
];

const setup = (overrides: Partial<Parameters<typeof useKeyboardShortcuts>[0]> = {}) => {
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
    cutSelected: vi.fn(),
    copySelected: vi.fn(),
    pasteSelected: vi.fn(),
    interruptExecution: vi.fn(),
    toggleOutputs: vi.fn(),
    selectRelative: vi.fn(),
    setMarkdownHeading: vi.fn(),
    toggleLineNumbers: vi.fn(),
    openSettings: vi.fn(),
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
};

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
    for (const key of ['a', 'b', 'm', 'y', 'j', 'k']) {
      fireEvent.keyDown(window, { key });
    }
    expect(actions.insertCell).toHaveBeenNthCalledWith(1, 0, 'code');
    expect(actions.insertCell).toHaveBeenNthCalledWith(2, 1, 'code');
    expect(actions.convertSelected).toHaveBeenNthCalledWith(1, 'markdown');
    expect(actions.convertSelected).toHaveBeenNthCalledWith(2, 'code');
    expect(actions.moveSelected).toHaveBeenNthCalledWith(1, 1);
    expect(actions.moveSelected).toHaveBeenNthCalledWith(2, -1);
    fireEvent.keyDown(window, { key: 'c', shiftKey: true });
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

  it('supports Jupyter-style cell clipboard and selection commands', () => {
    const { actions } = setup();
    fireEvent.keyDown(window, { key: 'x' });
    fireEvent.keyDown(window, { key: 'c' });
    fireEvent.keyDown(window, { key: 'v' });
    fireEvent.keyDown(window, { key: 'v', shiftKey: true });
    expect(actions.cutSelected).toHaveBeenCalledOnce();
    expect(actions.copySelected).toHaveBeenCalledOnce();
    expect(actions.pasteSelected).toHaveBeenNthCalledWith(1, true);
    expect(actions.pasteSelected).toHaveBeenNthCalledWith(2, false);
    fireEvent.keyDown(window, { key: 'ArrowUp' });
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    expect(actions.selectRelative).toHaveBeenNthCalledWith(1, -1);
    expect(actions.selectRelative).toHaveBeenNthCalledWith(2, 1);
  });

  it('interrupts the kernel on a double-press of the interrupt key', () => {
    const { actions } = setup();
    fireEvent.keyDown(window, { key: 'i' });
    expect(actions.interruptExecution).not.toHaveBeenCalled();
    const second = setup({ lastCommand: 'i' });
    fireEvent.keyDown(window, { key: 'i' });
    expect(second.actions.interruptExecution).toHaveBeenCalledOnce();
    second.unmount();
  });

  it('toggles outputs, undoes, and applies markdown headings in command mode', () => {
    const { actions } = setup();
    fireEvent.keyDown(window, { key: 'o' });
    fireEvent.keyDown(window, { key: 'z' });
    fireEvent.keyDown(window, { key: '3' });
    expect(actions.toggleOutputs).toHaveBeenCalledOnce();
    expect(actions.undo).toHaveBeenCalledOnce();
    expect(actions.setMarkdownHeading).toHaveBeenCalledWith(3);
    fireEvent.keyDown(window, { key: 'h' });
    expect(actions.openSettings).toHaveBeenCalledOnce();
  });

  it('toggles line numbers with shift+l in any mode', () => {
    const { actions } = setup({ mode: 'edit' });
    fireEvent.keyDown(window, { key: 'L', shiftKey: true });
    expect(actions.toggleLineNumbers).toHaveBeenCalledOnce();
  });

  it('opens settings for the shortcuts-help command key', () => {
    const { actions } = setup();
    fireEvent.keyDown(window, { key: '?' });
    expect(actions.openSettings).toHaveBeenCalledOnce();
  });

  it('does nothing for navigation commands without a selected cell and removes listener', () => {
    const { actions, unmount } = setup({ selectedIndex: -1, cells: [] });
    fireEvent.keyDown(window, { key: 'Enter', shiftKey: true });
    expect(actions.executeCell).not.toHaveBeenCalled();
    unmount();
    fireEvent.keyDown(window, { key: 'a' });
    expect(actions.insertCell).not.toHaveBeenCalled();
  });

  it('does not interpret typing in ordinary form controls as notebook commands', () => {
    const { actions } = setup();
    const input = document.createElement('input');
    document.body.append(input);
    fireEvent.keyDown(input, { key: 'a', bubbles: true });
    fireEvent.keyDown(input, { key: 'd', bubbles: true });
    expect(actions.insertCell).not.toHaveBeenCalled();
    expect(actions.deleteSelected).not.toHaveBeenCalled();
    input.remove();
  });
});
