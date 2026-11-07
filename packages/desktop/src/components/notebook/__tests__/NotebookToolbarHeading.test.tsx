import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import NotebookToolbarHeading from '../NotebookToolbarHeading';

const renderHeading = (isDirty = false) =>
  render(<NotebookToolbarHeading fileName='experiment.mars' isDirty={isDirty} />);

describe('NotebookToolbarHeading', () => {
  it('renders the workspace breadcrumb', () => {
    renderHeading();

    expect(screen.getByText('Workspace')).toBeInTheDocument();
    expect(screen.getByText('experiment.mars')).toHaveAttribute('title', 'experiment.mars');
    expect(screen.queryByRole('button', { name: 'Rename notebook' })).not.toBeInTheDocument();
  });

  it('shows the unsaved indicator only when dirty', () => {
    const { rerender } = renderHeading(true);
    expect(screen.getByTitle('Unsaved changes')).toHaveAttribute(
      'data-component',
      'NotebookDirtyIndicator',
    );

    rerender(<NotebookToolbarHeading fileName='experiment.mars' isDirty={false} />);
    expect(screen.queryByTitle('Unsaved changes')).not.toBeInTheDocument();
  });

  it('exposes the component marker', () => {
    const { container } = renderHeading();
    expect(
      container.querySelector('[data-component="NotebookToolbarHeading"]'),
    ).toBeInTheDocument();
  });
});
