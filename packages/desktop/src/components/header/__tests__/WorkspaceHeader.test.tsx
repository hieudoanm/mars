import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import WorkspaceHeader, { type WorkspaceTab } from '../WorkspaceHeader';

const tabs: WorkspaceTab[] = [
  { id: 't1', label: 'first.mars', isDirty: false },
  { id: 't2', label: 'second.mars', isDirty: true },
];

const setup = (overrides: Partial<Parameters<typeof WorkspaceHeader>[0]> = {}) => {
  const props = {
    tabs,
    activeId: 't1',
    explorerOpen: true,
    onSelectTab: vi.fn(),
    onCloseTab: vi.fn(),
    onNewTab: vi.fn(),
    onToggleExplorer: vi.fn(),
    onOpenSettings: vi.fn(),
    ...overrides,
  };
  render(<WorkspaceHeader {...props} />);
  return props;
};

describe('WorkspaceHeader', () => {
  it('selects and closes tabs', () => {
    const props = setup();
    fireEvent.click(screen.getAllByRole('button', { name: /second\.mars/ })[0]);
    expect(props.onSelectTab).toHaveBeenCalledWith('t2');
    fireEvent.click(screen.getByRole('button', { name: /Close second\.mars/ }));
    expect(props.onCloseTab).toHaveBeenCalledWith('t2');
  });

  it('marks the active tab and dirty indicator', () => {
    setup();
    const active = screen.getAllByRole('button', { name: /first\.mars/ })[0];
    expect(active).toHaveAttribute('aria-current', 'page');
    expect(screen.getByLabelText('Unsaved changes')).toBeInTheDocument();
  });

  it('creates a new tab and toggles explorer/settings', () => {
    const props = setup();
    fireEvent.click(screen.getByRole('button', { name: 'New notebook tab' }));
    expect(props.onNewTab).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Hide file explorer' }));
    expect(props.onToggleExplorer).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(props.onOpenSettings).toHaveBeenCalled();
  });

  it('shows the status text when provided', () => {
    setup({ status: 'Saved' });
    expect(screen.getByText('Saved')).toBeInTheDocument();
  });

  it('opens the recents menu and selects a recent notebook', () => {
    const props = setup({
      recents: ['/books/alpha.mars', '/books/beta.mars'],
      onOpenRecent: vi.fn(),
    });
    fireEvent.click(screen.getByRole('button', { name: 'Open recent notebook' }));
    expect(screen.getByText('Recent notebooks')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /alpha\.mars/ }));
    expect(props.onOpenRecent).toHaveBeenCalledWith('/books/alpha.mars');
    // menu closes after selection
    expect(screen.queryByText('Recent notebooks')).toBeNull();
  });

  it('hides the recents button when the list is empty', () => {
    setup({ recents: [], onOpenRecent: vi.fn() });
    expect(screen.queryByRole('button', { name: 'Open recent notebook' })).toBeNull();
  });

  it('reorders tabs via drag and drop', () => {
    const props = setup({ onReorderTabs: vi.fn() });
    const first = screen
      .getAllByRole('button', { name: /first\.mars/ })[0]
      .closest('div[draggable]')!;
    const second = screen
      .getAllByRole('button', { name: /second\.mars/ })[0]
      .closest('div[draggable]')!;
    const dataTransfer = { effectAllowed: '', dropEffect: '' };
    fireEvent.dragStart(first, { dataTransfer });
    fireEvent.dragOver(second, { dataTransfer });
    fireEvent.drop(second, { dataTransfer });
    expect(props.onReorderTabs).toHaveBeenCalledWith('t1', 't2');
  });

  it('closes a tab with the middle mouse button', () => {
    const props = setup();
    fireEvent(
      screen.getAllByRole('button', { name: /second\.mars/ })[0],
      new MouseEvent('auxclick', { bubbles: true, button: 1 }),
    );
    expect(props.onCloseTab).toHaveBeenCalledWith('t2');
  });
});
