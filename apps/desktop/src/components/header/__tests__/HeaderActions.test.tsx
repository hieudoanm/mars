import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const { onOpen, onSave, onExport, onRestart } = vi.hoisted(() => ({
  onOpen: vi.fn(),
  onSave: vi.fn(),
  onExport: vi.fn(),
  onRestart: vi.fn(),
}));

import HeaderActions from '../HeaderActions';

const renderActions = (status = 'Saved', mode: 'command' | 'edit' = 'command') =>
  render(
    <HeaderActions
      status={status}
      mode={mode}
      onOpen={onOpen}
      onSave={onSave}
      onExport={onExport}
      onRestart={onRestart}
    />,
  );

describe('HeaderActions', () => {
  beforeEach(() => {
    onOpen.mockReset();
    onSave.mockReset();
    onExport.mockReset();
    onRestart.mockReset();
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
        onRestart={onRestart}
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

    expect(buttons).toHaveLength(4);
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

    expect(tooltips).toHaveLength(4);
    expect(tooltips.map((tooltip) => tooltip.textContent)).toEqual([
      'Open',
      'Save',
      'Export Markdown',
      'Restart runtime',
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
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime' }));

    expect(onOpen).toHaveBeenCalledOnce();
    expect(onSave).toHaveBeenCalledOnce();
    expect(onExport).toHaveBeenCalledOnce();
    expect(onRestart).toHaveBeenCalledOnce();
  });
});
