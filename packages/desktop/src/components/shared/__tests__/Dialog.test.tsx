import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Dialog from '../Dialog';

describe('Dialog', () => {
  it('renders an accessible dialog shell', () => {
    render(
      <Dialog labelledBy='dialog-title'>
        <h2 id='dialog-title'>Confirm</h2>
      </Dialog>,
    );
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAttribute('aria-labelledby', 'dialog-title');
    expect(dialog).toHaveAccessibleName('Confirm');
  });

  it('renders the default component marker', () => {
    const { container } = render(
      <Dialog labelledBy='dialog-title'>
        <h2 id='dialog-title'>Confirm</h2>
      </Dialog>,
    );
    expect(container.querySelector('[data-component="Dialog"]')).toBeInTheDocument();
  });

  it('supports a custom component marker and panel class', () => {
    const { container } = render(
      <Dialog labelledBy='dialog-title' component='UnsavedChangesDialog' className='custom'>
        <h2 id='dialog-title'>Confirm</h2>
      </Dialog>,
    );
    expect(container.querySelector('[data-component="UnsavedChangesDialog"]')).toBeInTheDocument();
    expect(screen.getByRole('dialog').className).toContain('custom');
  });
});
