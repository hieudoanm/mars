import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import type { NotebookCell } from '../../../types/types';
import NotebookHeader from '../../header/NotebookHeader';
import NotebookSidebar from '../NotebookSidebar';
import NotebookStatusBar from '../NotebookStatusBar';
import UnsavedChangesDialog from '../../dialogs/UnsavedChangesDialog';

const cells: NotebookCell[] = [
  {
    id: 'md',
    cell_type: 'markdown',
    source: '# Intro',
    execution_count: null,
    outputs: [],
    metadata: {},
  },
  {
    id: 'code',
    cell_type: 'code',
    source: '1 + 1',
    execution_count: 1,
    outputs: [],
    metadata: {},
  },
];

describe('notebook chrome components', () => {
  it('shows header state and routes toolbar actions', () => {
    const onSave = vi.fn();
    render(
      <NotebookHeader
        status='Saved'
        mode='command'
        onOpen={vi.fn()}
        onSave={onSave}
        onExport={vi.fn()}
        onExportHtml={vi.fn()}
        onExportPdf={vi.fn()}
        onImportIpynb={vi.fn()}
        onExportIpynb={vi.fn()}
        onRestart={vi.fn()}
        onRestartRunAll={vi.fn()}
        sidebarOpen
        onToggleSidebar={vi.fn()}
      />,
    );
    expect(screen.getByRole('heading', { name: 'Mars' })).toBeInTheDocument();
    expect(screen.getByText('Explore ideas.')).toBeInTheDocument();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('renders notebook filename and cell selection state', () => {
    const onSelect = vi.fn();
    render(
      <NotebookSidebar
        cells={cells}
        selectedId='code'
        filePath='/notebooks/demo.mars'
        isDirty
        onSelect={onSelect}
        onRenameFile={vi.fn()}
      />,
    );
    expect(screen.getByText('demo.mars •')).toBeInTheDocument();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    const cellButtons = screen.getAllByRole('button');
    expect(cellButtons[2].className).toContain('bg-orange-950');
    fireEvent.click(cellButtons[1]);
    expect(onSelect).toHaveBeenCalledWith('md');
  });

  it('shows saved state and cell count in the status bar', () => {
    const { rerender } = render(<NotebookStatusBar cellCount={2} isDirty={false} />);
    expect(screen.getByText('2 cells')).toBeInTheDocument();
    expect(screen.getByText('All changes saved')).toBeInTheDocument();
    rerender(<NotebookStatusBar cellCount={3} isDirty />);
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });

  it('provides distinct actions for opening and closing confirmations', () => {
    const onDiscard = vi.fn();
    const { rerender } = render(
      <UnsavedChangesDialog
        closeRequested={false}
        onCancel={vi.fn()}
        onDiscard={onDiscard}
        onSave={vi.fn()}
      />,
    );
    expect(
      screen.getByRole('heading', { name: 'Save changes before opening?' }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Open without saving' }));
    expect(onDiscard).toHaveBeenCalledOnce();
    rerender(
      <UnsavedChangesDialog
        closeRequested
        onCancel={vi.fn()}
        onDiscard={onDiscard}
        onSave={vi.fn()}
      />,
    );
    expect(
      screen.getByRole('heading', { name: 'Save changes before closing?' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Close without saving' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save and close' })).toBeInTheDocument();
  });
});
