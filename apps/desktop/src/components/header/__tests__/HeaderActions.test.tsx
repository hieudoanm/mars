import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const {
  onOpen,
  onSave,
  onExport,
  onImportIpynb,
  onExportIpynb,
  onRestart,
  onToggleSidebar,
  onToggleExplorer,
} = vi.hoisted(() => ({
  onOpen: vi.fn(),
  onSave: vi.fn(),
  onExport: vi.fn(),
  onImportIpynb: vi.fn(),
  onExportIpynb: vi.fn(),
  onRestart: vi.fn(),
  onToggleSidebar: vi.fn(),
  onToggleExplorer: vi.fn(),
}));

import HeaderActions from '../HeaderActions';

const renderActions = (
  status = 'Saved',
  mode: 'command' | 'edit' = 'command',
  sidebarOpen = true,
  explorerOpen = true,
) =>
  render(
    <HeaderActions
      status={status}
      mode={mode}
      onOpen={onOpen}
      onSave={onSave}
      onExport={onExport}
      onImportIpynb={onImportIpynb}
      onExportIpynb={onExportIpynb}
      onRestart={onRestart}
      sidebarOpen={sidebarOpen}
      onToggleSidebar={onToggleSidebar}
      explorerOpen={explorerOpen}
      onToggleExplorer={onToggleExplorer}
    />,
  );

describe('HeaderActions', () => {
  beforeEach(() => {
    onOpen.mockReset();
    onSave.mockReset();
    onExport.mockReset();
    onImportIpynb.mockReset();
    onExportIpynb.mockReset();
    onRestart.mockReset();
    onToggleSidebar.mockReset();
    onToggleExplorer.mockReset();
  });

  it('renders the command mode status', () => {
    renderActions();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();
  });

  it('renders the edit mode status', () => {
    renderActions('Unsaved', 'edit');
    expect(screen.getByText('Unsaved · Edit mode')).toBeInTheDocument();
  });

  it('updates the status when props change', () => {
    const { rerender } = renderActions();
    rerender(
      <HeaderActions
        status='Autosaved'
        mode='edit'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onImportIpynb={onImportIpynb}
        onExportIpynb={onExportIpynb}
        onRestart={onRestart}
        sidebarOpen
        onToggleSidebar={onToggleSidebar}
        explorerOpen
        onToggleExplorer={onToggleExplorer}
      />,
    );
    expect(screen.getByText('Autosaved · Edit mode')).toBeInTheDocument();
  });

  it('exposes the component marker', () => {
    const { container } = renderActions();
    expect(container.querySelector('[data-component="HeaderActions"]')).toBeInTheDocument();
  });

  it('shows a status dot', () => {
    const { container } = renderActions();
    expect(
      container.querySelector('[data-component="Badge"] [aria-hidden="true"]'),
    ).toBeInTheDocument();
  });

  it('renders a decorative icon for every action', () => {
    renderActions();
    const buttons = screen.getAllByRole('button');

    expect(buttons).toHaveLength(8);
    for (const button of buttons) {
      const icon = button.querySelector('svg');
      expect(icon).toBeInTheDocument();
      expect(icon).toHaveAttribute('aria-hidden', 'true');
    }
  });

  it('uses tooltips instead of visible button text', () => {
    renderActions();
    const buttons = screen.getAllByRole('button');
    const tooltips = screen.getAllByRole('tooltip');

    expect(tooltips).toHaveLength(8);
    expect(tooltips.map((tooltip) => tooltip.textContent)).toEqual([
      'Open',
      'Save',
      'Export Markdown',
      'Import .ipynb',
      'Export .ipynb',
      'Restart runtime',
      'Hide file explorer',
      'Hide notebook sidebar',
    ]);

    for (const [index, button] of buttons.entries()) {
      expect(button.textContent).toBe('');
      expect(button).toHaveAttribute('aria-label', tooltips[index].textContent);
      expect(button).toHaveAttribute('aria-describedby', tooltips[index].id);
    }
  });

  it('routes each action to its handler', () => {
    renderActions();
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    fireEvent.click(screen.getByRole('button', { name: 'Export Markdown' }));
    fireEvent.click(screen.getByRole('button', { name: 'Import .ipynb' }));
    fireEvent.click(screen.getByRole('button', { name: 'Export .ipynb' }));
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime' }));
    fireEvent.click(screen.getByRole('button', { name: 'Hide file explorer' }));
    fireEvent.click(screen.getByRole('button', { name: 'Hide notebook sidebar' }));

    expect(onOpen).toHaveBeenCalledOnce();
    expect(onSave).toHaveBeenCalledOnce();
    expect(onExport).toHaveBeenCalledOnce();
    expect(onImportIpynb).toHaveBeenCalledOnce();
    expect(onExportIpynb).toHaveBeenCalledOnce();
    expect(onRestart).toHaveBeenCalledOnce();
    expect(onToggleSidebar).toHaveBeenCalledOnce();
    expect(onToggleExplorer).toHaveBeenCalledOnce();
  });

  it('labels the sidebar toggle when closed', () => {
    renderActions('Saved', 'command', false);
    expect(screen.getByRole('button', { name: 'Show notebook sidebar' })).toBeInTheDocument();
  });

  it('labels the explorer toggle when closed', () => {
    renderActions('Saved', 'command', true, false);
    expect(screen.getByRole('button', { name: 'Show file explorer' })).toBeInTheDocument();
  });
});
