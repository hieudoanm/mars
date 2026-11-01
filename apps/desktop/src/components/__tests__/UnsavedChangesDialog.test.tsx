import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const { onCancel, onDiscard, onSave } = vi.hoisted(() => ({
  onCancel: vi.fn(),
  onDiscard: vi.fn(),
  onSave: vi.fn(),
}));

import UnsavedChangesDialog from '../UnsavedChangesDialog';

describe('UnsavedChangesDialog', () => {
  beforeEach(() => {
    onCancel.mockReset();
    onDiscard.mockReset();
    onSave.mockReset();
  });

  it('renders opening confirmation dialog', () => {
    render(
      <UnsavedChangesDialog
        closeRequested={false}
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('UNSAVED NOTEBOOK')).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Save changes before opening?' }),
    ).toBeInTheDocument();
  });

  it('renders closing confirmation dialog', () => {
    render(
      <UnsavedChangesDialog
        closeRequested
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    expect(
      screen.getByRole('heading', { name: 'Save changes before closing?' }),
    ).toBeInTheDocument();
  });

  it('renders unsaved changes message', () => {
    render(
      <UnsavedChangesDialog
        closeRequested={false}
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    expect(
      screen.getByText('Your current notebook has unsaved changes. Choose how Mars should continue.'),
    ).toBeInTheDocument();
  });

  it('calls onCancel when Cancel is clicked', () => {
    render(
      <UnsavedChangesDialog
        closeRequested={false}
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('calls onDiscard with open-without-saving label', () => {
    render(
      <UnsavedChangesDialog
        closeRequested={false}
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Open without saving' }));
    expect(onDiscard).toHaveBeenCalledOnce();
  });

  it('calls onDiscard with close-without-saving label', () => {
    render(
      <UnsavedChangesDialog
        closeRequested
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Close without saving' }));
    expect(onDiscard).toHaveBeenCalledOnce();
  });

  it('calls onSave with save-and-open label', () => {
    render(
      <UnsavedChangesDialog
        closeRequested={false}
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Save and open' }));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('calls onSave with save-and-close label', () => {
    render(
      <UnsavedChangesDialog
        closeRequested
        onCancel={onCancel}
        onDiscard={onDiscard}
        onSave={onSave}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Save and close' }));
    expect(onSave).toHaveBeenCalledOnce();
  });
});
