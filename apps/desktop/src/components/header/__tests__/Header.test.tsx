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

import Header from '../Header';

const renderHeader = (status = 'Saved', mode: 'command' | 'edit' = 'command') =>
  render(
    <Header
      status={status}
      mode={mode}
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

describe('Header', () => {
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

  it('renders brand, title, and status', () => {
    renderHeader();
    expect(screen.getByAltText('Mars')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Mars' })).toBeInTheDocument();
    expect(screen.getByText('Explore ideas.')).toBeInTheDocument();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();
  });

  it('composes the brand and action regions', () => {
    const { container } = renderHeader();
    expect(container.querySelector('[data-component="Header"]')).toBeInTheDocument();
    expect(container.querySelector('[data-component="HeaderBrand"]')).toBeInTheDocument();
    expect(container.querySelector('[data-component="HeaderActions"]')).toBeInTheDocument();
  });

  it('renders edit mode status', () => {
    renderHeader('Unsaved', 'edit');
    expect(screen.getByText('Unsaved · Edit mode')).toBeInTheDocument();
  });

  it('updates the status when props change', () => {
    const { rerender } = renderHeader();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();

    rerender(
      <Header
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
    expect(screen.queryByText('Saved · Command mode')).not.toBeInTheDocument();
  });

  it('renders all primary actions', () => {
    renderHeader();
    expect(screen.getByRole('button', { name: 'Open' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Export Markdown' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Restart runtime' })).toBeInTheDocument();
  });

  it('calls onOpen when Open is clicked', () => {
    renderHeader();
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it('calls onSave when Save is clicked', () => {
    renderHeader();
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('calls onExport when Export Markdown is clicked', () => {
    renderHeader();
    fireEvent.click(screen.getByRole('button', { name: 'Export Markdown' }));
    expect(onExport).toHaveBeenCalledOnce();
  });

  it('calls onRestart when Restart runtime is clicked', () => {
    renderHeader();
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime' }));
    expect(onRestart).toHaveBeenCalledOnce();
  });
});
