import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import NotebookSessionOverlays from '../NotebookSessionOverlays';
import type { Notebook } from '../../../types/types';

vi.mock('../../../lib/kernelService', () => ({
  createNotebookCheckpoint: vi.fn(async () => '/cp/1.mars'),
  listNotebookCheckpoints: vi.fn(async () => []),
  readNotebookCheckpoint: vi.fn(async () => null),
  restoreNotebookCheckpoint: vi.fn(async () => {}),
}));

const notebook: Notebook = {
  format_version: 1,
  metadata: { language: 'rust', title: 'Demo' },
  cells: [
    {
      id: 'c1',
      cell_type: 'code',
      source: 'println!("x")',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
};

const makePersistence = (overrides: Record<string, unknown> = {}) => ({
  pendingOpenPath: null,
  closeRequested: false,
  pendingTrustPath: null,
  filePath: '/books/demo.mars',
  setPendingOpenPath: vi.fn(),
  setCloseRequested: vi.fn(),
  trustAndLoad: vi.fn(async () => {}),
  cancelTrust: vi.fn(),
  discardAndClose: vi.fn(),
  discardAndOpen: vi.fn(),
  saveAndClose: vi.fn(async () => {}),
  saveAndOpen: vi.fn(async () => {}),
  saveNotebook: vi.fn(async () => true),
  reloadFromDisk: vi.fn(async () => {}),
  ...overrides,
});

const makeEditor = () => ({
  notebook,
  cells: notebook.cells,
  setNotebook: vi.fn(),
  setSelectedId: vi.fn(),
  setMode: vi.fn(),
  setDependencies: vi.fn(),
  renameNotebook: vi.fn(),
});

type Props = Parameters<typeof NotebookSessionOverlays>[0];

const setup = (
  overrides: Omit<Partial<Props>, 'persistence'> & { persistence?: Record<string, unknown> } = {},
) => {
  const { persistence: persistenceOverrides, ...rest } = overrides;
  const persistence = makePersistence(persistenceOverrides as Record<string, unknown>);
  const editor = makeEditor();
  const props: Props = {
    persistence: persistence as Props['persistence'],
    editor,
    pendingPrompt: null,
    respondToPrompt: vi.fn(async () => {}),
    showTabCloseDialog: false,
    onTabClosed: vi.fn(),
    onTabCloseCancel: vi.fn(),
    showSearch: false,
    setShowSearch: vi.fn(),
    showDependencies: false,
    setShowDependencies: vi.fn(),
    showCheckpoints: false,
    setShowCheckpoints: vi.fn(),
    showRename: false,
    setShowRename: vi.fn(),
    setStatus: vi.fn(),
    ...rest,
  };
  render(<NotebookSessionOverlays {...props} />);
  return { props, persistence, editor };
};

describe('NotebookSessionOverlays', () => {
  it('renders UnsavedChangesDialog for a pending open path', () => {
    const { persistence } = setup({ persistence: { pendingOpenPath: '/tmp/next.mars' } });
    expect(screen.getByText('UNSAVED NOTEBOOK')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Open without saving' }));
    expect(persistence.discardAndOpen).toHaveBeenCalled();
  });

  it('routes close-requested actions to close handlers', () => {
    const { persistence } = setup({ persistence: { closeRequested: true } });
    fireEvent.click(screen.getByRole('button', { name: 'Save and close' }));
    expect(persistence.saveAndClose).toHaveBeenCalled();
  });

  it('routes tab-close actions to tab handlers', () => {
    const { props } = setup({
      showTabCloseDialog: true,
      persistence: { closeRequested: true },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Close without saving' }));
    expect(props.onTabClosed).toHaveBeenCalled();
  });

  it('cancels pending open state', () => {
    const { persistence } = setup({ persistence: { pendingOpenPath: '/tmp/x.mars' } });
    fireEvent.click(screen.getByRole('button', { name: /cancel/i }));
    expect(persistence.setPendingOpenPath).toHaveBeenCalledWith(null);
  });

  it('shows TrustDialog and forwards actions', () => {
    const { persistence } = setup({ persistence: { pendingTrustPath: '/tmp/evil.mars' } });
    expect(screen.getByText('evil.mars')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /trust & open/i }));
    expect(persistence.trustAndLoad).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(persistence.cancelTrust).toHaveBeenCalled();
  });

  it('shows PromptDialog and submits the answer', async () => {
    const respondToPrompt = vi.fn(async () => {});
    setup({ pendingPrompt: 'Continue?', respondToPrompt });
    const input = screen.getByLabelText('Prompt answer');
    fireEvent.change(input, { target: { value: 'yes' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(respondToPrompt).toHaveBeenCalledWith('yes');
  });

  it('search dialog replace-all updates the notebook', () => {
    const { editor, props } = setup({ showSearch: true });
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'println' } });
    fireEvent.change(screen.getByLabelText('Replace with'), { target: { value: 'print' } });
    fireEvent.click(screen.getByRole('button', { name: 'Replace all occurrences' }));
    fireEvent.click(screen.getByRole('button', { name: 'Replace all' }));
    expect(editor.setNotebook).toHaveBeenCalled();
    expect(props.setStatus).toHaveBeenCalledWith('Replaced 1 occurrence(s)');
  });

  it('search replace with no matches reports status', () => {
    const { props } = setup({ showSearch: true });
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'zzz' } });
    expect(screen.getByRole('button', { name: 'Replace all occurrences' })).toBeDisabled();
    fireEvent.change(screen.getByLabelText('Replace with'), { target: { value: 'x' } });
    fireEvent.keyDown(screen.getByLabelText('Replace with'), { key: 'Enter' });
    expect(props.setStatus).toHaveBeenCalledWith('No occurrences found');
  });

  it('search select uses onRevealMatch when provided', () => {
    const onRevealMatch = vi.fn();
    const { editor } = setup({ showSearch: true, onRevealMatch });
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'println' } });
    fireEvent.click(screen.getAllByRole('button', { name: /Cell \d/ })[0]);
    expect(onRevealMatch).toHaveBeenCalledWith('c1', 0, expect.any(Number));
    expect(editor.setSelectedId).not.toHaveBeenCalled();
  });

  it('search select falls back to editor selection without onRevealMatch', () => {
    const { editor } = setup({ showSearch: true });
    fireEvent.change(screen.getByLabelText('Search query'), { target: { value: 'println' } });
    fireEvent.click(screen.getAllByRole('button', { name: /Cell \d/ })[0]);
    expect(editor.setSelectedId).toHaveBeenCalledWith('c1');
    expect(editor.setMode).toHaveBeenCalledWith('command');
  });

  it('dependencies dialog saves through the editor and closes', () => {
    const { editor, props } = setup({ showDependencies: true });
    expect(screen.getByText('Cargo dependencies')).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText('Dependency name'), { target: { value: 'serde' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add dependency' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save dependencies' }));
    expect(editor.setDependencies).toHaveBeenCalledWith({ serde: '*' });
    expect(props.setShowDependencies).toHaveBeenCalledWith(false);
  });

  it('rename dialog renames the notebook and closes', () => {
    const { editor, props } = setup({ showRename: true });
    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Renamed' } });
    fireEvent.click(screen.getByRole('button', { name: /rename|save/i }));
    expect(editor.renameNotebook).toHaveBeenCalledWith('Renamed');
    expect(props.setShowRename).toHaveBeenCalledWith(false);
  });

  it('checkpoints dialog opens with the current file path', async () => {
    setup({ showCheckpoints: true });
    expect((await screen.findAllByText(/checkpoint/i)).length).toBeGreaterThan(0);
  });
});
