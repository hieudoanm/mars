import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import NotebookToolbarHeading from '../NotebookToolbarHeading';

const onRename = vi.fn();

const renderHeading = (isDirty = false) =>
  render(
    <NotebookToolbarHeading
      fileName='experiment.mars'
      title='My Notebook'
      isDirty={isDirty}
      onRename={onRename}
    />,
  );

describe('NotebookToolbarHeading', () => {
  beforeEach(() => {
    onRename.mockReset();
  });

  it('renders the workspace breadcrumb and notebook title', () => {
    renderHeading();

    expect(screen.getByText('Workspace')).toBeInTheDocument();
    expect(screen.getByText('experiment.mars')).toHaveAttribute('title', 'experiment.mars');
    expect(screen.getByRole('heading', { level: 2, name: 'My Notebook' })).toHaveAttribute(
      'title',
      'My Notebook',
    );
    expect(screen.getByRole('button', { name: 'Rename notebook' })).toBeInTheDocument();
  });

  it('renames the notebook on commit', () => {
    renderHeading();

    fireEvent.click(screen.getByRole('button', { name: 'Rename notebook' }));
    const input = screen.getByLabelText('Notebook title');
    expect(input).toHaveValue('My Notebook');

    fireEvent.change(input, { target: { value: '  Analysis  ' } });
    fireEvent.blur(input);

    expect(onRename).toHaveBeenCalledWith('Analysis');
    expect(screen.queryByLabelText('Notebook title')).not.toBeInTheDocument();
  });

  it('commits the title with Enter', () => {
    renderHeading();

    fireEvent.click(screen.getByRole('button', { name: 'Rename notebook' }));
    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Renamed' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onRename).toHaveBeenCalledWith('Renamed');
  });

  it('cancels renaming with Escape', () => {
    renderHeading();

    fireEvent.click(screen.getByRole('button', { name: 'Rename notebook' }));
    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Discard me' } });
    fireEvent.keyDown(input, { key: 'Escape' });

    expect(onRename).not.toHaveBeenCalled();
    expect(screen.queryByLabelText('Notebook title')).not.toBeInTheDocument();
  });

  it('ignores blank titles', () => {
    renderHeading();

    fireEvent.click(screen.getByRole('button', { name: 'Rename notebook' }));
    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: '   ' } });
    fireEvent.blur(input);

    expect(onRename).not.toHaveBeenCalled();
    expect(screen.queryByLabelText('Notebook title')).not.toBeInTheDocument();
  });

  it('shows the unsaved indicator only when dirty', () => {
    const { rerender } = renderHeading(true);
    expect(screen.getByTitle('Unsaved changes')).toHaveAttribute(
      'data-component',
      'NotebookDirtyIndicator',
    );

    rerender(
      <NotebookToolbarHeading
        fileName='experiment.mars'
        title='My Notebook'
        isDirty={false}
        onRename={onRename}
      />,
    );
    expect(screen.queryByTitle('Unsaved changes')).not.toBeInTheDocument();
  });

  it('exposes the component marker', () => {
    const { container } = renderHeading();
    expect(
      container.querySelector('[data-component="NotebookToolbarHeading"]'),
    ).toBeInTheDocument();
  });
});
