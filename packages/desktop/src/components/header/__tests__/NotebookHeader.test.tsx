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

import NotebookHeader from '../NotebookHeader';

const actionProps = {
  onOpen,
  onSave,
  onExport,
  onExportHtml,
  onExportPdf,
  onImportIpynb,
  onExportIpynb,
  onRestart,
  onRestartRunAll,
  sidebarOpen: true,
  onToggleSidebar,
};

const renderHeader = (status = 'Saved', mode: 'command' | 'edit' = 'command') =>
  render(<NotebookHeader status={status} mode={mode} {...actionProps} />);

describe('NotebookHeader', () => {
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

  it('renders brand, title, and status', () => {
    renderHeader();
    expect(screen.getByAltText('Mars')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Mars' })).toBeInTheDocument();
    expect(screen.getByText('Explore ideas.')).toBeInTheDocument();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();
  });

  it('hides the brand when showBrand is false', () => {
    render(<NotebookHeader status='Saved' mode='command' showBrand={false} {...actionProps} />);
    expect(screen.queryByRole('heading', { name: 'Mars' })).not.toBeInTheDocument();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();
  });

  it('composes the brand and action regions', () => {
    const { container } = renderHeader();
    expect(container.querySelector('[data-component="NotebookHeader"]')).toBeInTheDocument();
    expect(container.querySelector('[data-component="NotebookHeaderBrand"]')).toBeInTheDocument();
    expect(container.querySelector('[data-component="NotebookHeaderActions"]')).toBeInTheDocument();
  });

  it('renders edit mode status', () => {
    renderHeader('Unsaved', 'edit');
    expect(screen.getByText('Unsaved · Edit mode')).toBeInTheDocument();
  });

  it('updates the status when props change', () => {
    const { rerender } = renderHeader();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();

    rerender(<NotebookHeader status='Autosaved' mode='edit' {...actionProps} />);
    expect(screen.getByText('Autosaved · Edit mode')).toBeInTheDocument();
    expect(screen.queryByText('Saved · Command mode')).not.toBeInTheDocument();
  });

  it('renders all primary actions', () => {
    renderHeader();
    expect(screen.getByRole('button', { name: 'Open' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Export Markdown' })).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Restart runtime and clear outputs' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Restart runtime and run all' })).toBeInTheDocument();
  });

  it('calls onSave when Save is clicked', () => {
    renderHeader();
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('calls onRestartRunAll when restart-and-run is clicked', () => {
    renderHeader();
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime and run all' }));
    expect(onRestartRunAll).toHaveBeenCalledOnce();
  });
});
