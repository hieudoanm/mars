import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const { onOpen, onSave, onExport, onRestart } = vi.hoisted(() => ({
  onOpen: vi.fn(),
  onSave: vi.fn(),
  onExport: vi.fn(),
  onRestart: vi.fn(),
}));

import AppHeader from '../AppHeader';

describe('AppHeader', () => {
  beforeEach(() => {
    onOpen.mockReset();
    onSave.mockReset();
    onExport.mockReset();
    onRestart.mockReset();
  });

  it('renders brand, title, and status', () => {
    render(
      <AppHeader
        status='Saved'
        mode='command'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
      />,
    );
    expect(screen.getByAltText('Mars')).toBeInTheDocument();
    expect(screen.getByText('MARS NOTEBOOK')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Explore ideas.' })).toBeInTheDocument();
    expect(screen.getByText('Saved · Command mode')).toBeInTheDocument();
  });

  it('renders edit mode status', () => {
    render(
      <AppHeader
        status='Unsaved'
        mode='edit'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
      />,
    );
    expect(screen.getByText('Unsaved · Edit mode')).toBeInTheDocument();
  });

  it('calls onOpen when Open is clicked', () => {
    render(
      <AppHeader
        status='Saved'
        mode='command'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it('calls onSave when Save is clicked', () => {
    render(
      <AppHeader
        status='Saved'
        mode='command'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('calls onExport when Export Markdown is clicked', () => {
    render(
      <AppHeader
        status='Saved'
        mode='command'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Export Markdown' }));
    expect(onExport).toHaveBeenCalledOnce();
  });

  it('calls onRestart when Restart runtime is clicked', () => {
    render(
      <AppHeader
        status='Saved'
        mode='command'
        onOpen={onOpen}
        onSave={onSave}
        onExport={onExport}
        onRestart={onRestart}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime' }));
    expect(onRestart).toHaveBeenCalledOnce();
  });
});
