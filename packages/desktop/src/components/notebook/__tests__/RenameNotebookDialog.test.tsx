import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import RenameNotebookDialog from '../RenameNotebookDialog';

const onRename = vi.fn();
const onClose = vi.fn();

const renderDialog = () =>
  render(
    <RenameNotebookDialog open={true} title='My Notebook' onRename={onRename} onClose={onClose} />,
  );

describe('RenameNotebookDialog', () => {
  beforeEach(() => {
    onRename.mockReset();
    onClose.mockReset();
  });

  it('renames the notebook on commit', () => {
    renderDialog();

    const input = screen.getByLabelText('Notebook title');
    expect(input).toHaveValue('My Notebook');

    fireEvent.change(input, { target: { value: '  Analysis  ' } });
    fireEvent.click(screen.getByRole('button', { name: 'Rename' }));

    expect(onRename).toHaveBeenCalledWith('Analysis');
    expect(onClose).toHaveBeenCalled();
  });

  it('commits the title with Enter', () => {
    renderDialog();

    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Renamed' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onRename).toHaveBeenCalledWith('Renamed');
    expect(onClose).toHaveBeenCalled();
  });

  it('closes without renaming on Escape or Cancel', () => {
    renderDialog();

    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Discard me' } });
    fireEvent.keyDown(input, { key: 'Escape' });

    expect(onRename).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();

    onClose.mockReset();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onRename).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it('ignores blank titles', () => {
    renderDialog();

    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: '   ' } });
    fireEvent.click(screen.getByRole('button', { name: 'Rename' }));

    expect(onRename).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it('renders nothing when closed', () => {
    render(
      <RenameNotebookDialog
        open={false}
        title='My Notebook'
        onRename={onRename}
        onClose={onClose}
      />,
    );
    expect(screen.queryByLabelText('Notebook title')).not.toBeInTheDocument();
  });
});
