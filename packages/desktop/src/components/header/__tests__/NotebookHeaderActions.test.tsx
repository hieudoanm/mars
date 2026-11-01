import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const {
  onOpen,
  onSave,
  onExport,
  onExportHtml,
  onExportPdf,
  onImportIpynb,
  onExportIpynb,
  onRestart,
  onRestartRunAll,
  onToggleSidebar,
} = vi.hoisted(() => ({
  onOpen: vi.fn(),
  onSave: vi.fn(),
  onExport: vi.fn(),
  onExportHtml: vi.fn(),
  onExportPdf: vi.fn(),
  onImportIpynb: vi.fn(),
  onExportIpynb: vi.fn(),
  onRestart: vi.fn(),
  onRestartRunAll: vi.fn(),
  onToggleSidebar: vi.fn(),
}));

import NotebookHeaderActions from '../NotebookHeaderActions';

const renderActions = (
  status = 'Saved',
  mode: 'command' | 'edit' = 'command',
  sidebarOpen = true,
) =>
  render(
    <NotebookHeaderActions
      status={status}
      mode={mode}
      onOpen={onOpen}
      onSave={onSave}
      onExport={onExport}
      onExportHtml={onExportHtml}
      onExportPdf={onExportPdf}
      onImportIpynb={onImportIpynb}
      onExportIpynb={onExportIpynb}
      onRestart={onRestart}
      onRestartRunAll={onRestartRunAll}
      sidebarOpen={sidebarOpen}
      onToggleSidebar={onToggleSidebar}
    />,
  );

describe('NotebookHeaderActions', () => {
  beforeEach(() => {
    onOpen.mockReset();
    onSave.mockReset();
    onExport.mockReset();
    onExportHtml.mockReset();
    onExportPdf.mockReset();
    onImportIpynb.mockReset();
    onExportIpynb.mockReset();
    onRestart.mockReset();
    onRestartRunAll.mockReset();
    onToggleSidebar.mockReset();
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
      <NotebookHeaderActions
        status='Autosaved'
        mode='edit'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onExportHtml={onExportHtml}
        onExportPdf={onExportPdf}
        onImportIpynb={onImportIpynb}
        onExportIpynb={onExportIpynb}
        onRestart={onRestart}
        onRestartRunAll={onRestartRunAll}
        sidebarOpen
        onToggleSidebar={onToggleSidebar}
      />,
    );
    expect(screen.getByText('Autosaved · Edit mode')).toBeInTheDocument();
  });

  it('exposes the component marker', () => {
    const { container } = renderActions();
    expect(container.querySelector('[data-component="NotebookHeaderActions"]')).toBeInTheDocument();
  });

  it('renders a decorative icon for every action', () => {
    renderActions();
    const buttons = screen.getAllByRole('button');

    expect(buttons).toHaveLength(10);
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

    expect(tooltips).toHaveLength(10);
    expect(tooltips.map((tooltip) => tooltip.textContent)).toEqual([
      'Open',
      'Save',
      'Export Markdown',
      'Export HTML',
      'Export PDF',
      'Import .ipynb',
      'Export .ipynb',
      'Restart runtime and clear outputs',
      'Restart runtime and run all',
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
    fireEvent.click(screen.getByRole('button', { name: 'Export HTML' }));
    fireEvent.click(screen.getByRole('button', { name: 'Export PDF' }));
    fireEvent.click(screen.getByRole('button', { name: 'Import .ipynb' }));
    fireEvent.click(screen.getByRole('button', { name: 'Export .ipynb' }));
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime and clear outputs' }));
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime and run all' }));
    fireEvent.click(screen.getByRole('button', { name: 'Hide notebook sidebar' }));

    expect(onOpen).toHaveBeenCalledOnce();
    expect(onSave).toHaveBeenCalledOnce();
    expect(onExport).toHaveBeenCalledOnce();
    expect(onExportHtml).toHaveBeenCalledOnce();
    expect(onExportPdf).toHaveBeenCalledOnce();
    expect(onImportIpynb).toHaveBeenCalledOnce();
    expect(onExportIpynb).toHaveBeenCalledOnce();
    expect(onRestart).toHaveBeenCalledOnce();
    expect(onRestartRunAll).toHaveBeenCalledOnce();
    expect(onToggleSidebar).toHaveBeenCalledOnce();
  });

  it('labels the sidebar toggle when closed', () => {
    renderActions('Saved', 'command', false);
    expect(screen.getByRole('button', { name: 'Show notebook sidebar' })).toBeInTheDocument();
  });
});
