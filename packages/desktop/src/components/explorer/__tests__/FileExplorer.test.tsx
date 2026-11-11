import '@testing-library/jest-dom/vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';

const {
  openDialog,
  listFiles,
  openExternal,
  createDir,
  createNotebook,
  deletePathMock,
  renamePath,
} = vi.hoisted(() => ({
  openDialog: vi.fn(),
  listFiles: vi.fn(),
  openExternal: vi.fn(async () => {}),
  createDir: vi.fn(async () => {}),
  createNotebook: vi.fn(async () => {}),
  deletePathMock: vi.fn(async () => {}),
  renamePath: vi.fn(async () => {}),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openDialog }));
vi.mock('../../../lib/kernelService', () => ({
  listNotebookFiles: listFiles,
  openPathExternal: openExternal,
  createDirectory: createDir,
  createNotebookFile: createNotebook,
  deletePath: deletePathMock,
  renameNotebookFile: renamePath,
}));

import FileExplorer from '../FileExplorer';

describe('FileExplorer', () => {
  it('opens a folder, shows nested notebooks, and selects a notebook', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      {
        name: 'examples',
        path: '/workspace/examples',
        isDirectory: true,
        children: [
          {
            name: 'analysis.mars',
            path: '/workspace/examples/analysis.mars',
            isDirectory: false,
            children: [],
          },
        ],
      },
    ]);
    const onOpenNotebook = vi.fn();
    const onStatus = vi.fn();

    render(
      <FileExplorer currentFilePath={null} onOpenNotebook={onOpenNotebook} onStatus={onStatus} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));

    await waitFor(() => expect(listFiles).toHaveBeenCalledWith('/workspace'));
    expect(screen.getByText('analysis.mars')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'analysis.mars' }));
    expect(onOpenNotebook).toHaveBeenCalledWith('/workspace/examples/analysis.mars');
  });

  it('opens non-notebook project files externally', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      {
        name: 'Cargo.toml',
        path: '/workspace/Cargo.toml',
        isDirectory: false,
        children: [],
      },
      {
        name: 'src',
        path: '/workspace/src',
        isDirectory: true,
        children: [
          {
            name: 'main.rs',
            path: '/workspace/src/main.rs',
            isDirectory: false,
            children: [],
          },
        ],
      },
    ]);
    const onOpenNotebook = vi.fn();
    const onStatus = vi.fn();

    render(
      <FileExplorer currentFilePath={null} onOpenNotebook={onOpenNotebook} onStatus={onStatus} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    await waitFor(() => expect(screen.getByText('Cargo.toml')).toBeInTheDocument());

    fireEvent.click(screen.getByRole('button', { name: 'Cargo.toml' }));
    await waitFor(() => expect(openExternal).toHaveBeenCalledWith('/workspace/Cargo.toml'));
    expect(onOpenNotebook).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'main.rs' }));
    await waitFor(() => expect(openExternal).toHaveBeenCalledWith('/workspace/src/main.rs'));
  });

  it('creates notebooks and folders with unique names', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      { name: 'untitled.mars', path: '/workspace/untitled.mars', isDirectory: false, children: [] },
    ]);
    const onOpenNotebook = vi.fn();
    const onStatus = vi.fn();
    render(
      <FileExplorer currentFilePath={null} onOpenNotebook={onOpenNotebook} onStatus={onStatus} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    await waitFor(() => expect(listFiles).toHaveBeenCalled());

    fireEvent.click(screen.getByRole('button', { name: 'New notebook' }));
    await waitFor(() => expect(createNotebook).toHaveBeenCalledWith('/workspace/untitled.mars-2'));
    expect(onOpenNotebook).toHaveBeenCalledWith('/workspace/untitled.mars-2');
    expect(onStatus).toHaveBeenCalledWith('Notebook created');

    fireEvent.click(screen.getByRole('button', { name: 'New folder' }));
    await waitFor(() => expect(createDir).toHaveBeenCalledWith('/workspace/new-folder'));
    expect(onStatus).toHaveBeenCalledWith('Folder created');
  });

  it('prompts to open a folder before creating entries', async () => {
    const onStatus = vi.fn();
    render(<FileExplorer currentFilePath={null} onOpenNotebook={vi.fn()} onStatus={onStatus} />);
    fireEvent.click(screen.getByRole('button', { name: 'New notebook' }));
    expect(onStatus).toHaveBeenCalledWith('Open a folder first');
    expect(createNotebook).not.toHaveBeenCalled();
  });

  it('loads a requested root path from props', async () => {
    listFiles.mockResolvedValue([]);
    const onStatus = vi.fn();
    render(
      <FileExplorer
        currentFilePath={null}
        requestedRootPath='/requested'
        onOpenNotebook={vi.fn()}
        onStatus={onStatus}
      />,
    );
    await waitFor(() => expect(listFiles).toHaveBeenCalledWith('/requested'));
    expect(onStatus).toHaveBeenCalledWith('Folder opened');
  });

  it('renames an entry after editing the draft', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      { name: 'old.mars', path: '/workspace/old.mars', isDirectory: false, children: [] },
    ]);
    const onStatus = vi.fn();
    render(<FileExplorer currentFilePath={null} onOpenNotebook={vi.fn()} onStatus={onStatus} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    await waitFor(() => expect(screen.getByText('old.mars')).toBeInTheDocument());

    fireEvent.click(screen.getByRole('button', { name: 'Rename old.mars' }));
    const input = await screen.findByLabelText('Rename old.mars');
    fireEvent.change(input, { target: { value: 'new.mars' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    await waitFor(() =>
      expect(renamePath).toHaveBeenCalledWith('/workspace/old.mars', '/workspace/new.mars'),
    );
    expect(onStatus).toHaveBeenCalledWith('Renamed');
  });

  it('deletes an entry after confirmation', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      { name: 'old.mars', path: '/workspace/old.mars', isDirectory: false, children: [] },
    ]);
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    const onStatus = vi.fn();
    render(<FileExplorer currentFilePath={null} onOpenNotebook={vi.fn()} onStatus={onStatus} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    await waitFor(() => expect(screen.getByText('old.mars')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Delete old.mars' }));
    await waitFor(() => expect(deletePathMock).toHaveBeenCalledWith('/workspace/old.mars'));
    expect(onStatus).toHaveBeenCalledWith('Deleted');
    confirmSpy.mockRestore();
  });

  it('expands and collapses directories', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      {
        name: 'src',
        path: '/workspace/src',
        isDirectory: true,
        children: [
          { name: 'main.rs', path: '/workspace/src/main.rs', isDirectory: false, children: [] },
        ],
      },
    ]);
    render(<FileExplorer currentFilePath={null} onOpenNotebook={vi.fn()} onStatus={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    const dirButton = await screen.findByRole('button', { name: 'src' });
    expect(dirButton).toHaveAttribute('aria-expanded', 'true');
    expect(await screen.findByText('main.rs')).toBeInTheDocument();
    fireEvent.click(dirButton);
    expect(dirButton).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(dirButton);
    expect(dirButton).toHaveAttribute('aria-expanded', 'true');
  });

  it('cancels a rename with Escape and skips empty names', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      { name: 'old.mars', path: '/workspace/old.mars', isDirectory: false, children: [] },
    ]);
    render(<FileExplorer currentFilePath={null} onOpenNotebook={vi.fn()} onStatus={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    await waitFor(() => expect(screen.getByText('old.mars')).toBeInTheDocument());

    fireEvent.click(screen.getByRole('button', { name: 'Rename old.mars' }));
    const input = await screen.findByLabelText('Rename old.mars');
    fireEvent.keyDown(input, { key: 'Escape' });
    expect(renamePath).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Rename old.mars' }));
    const input2 = await screen.findByLabelText('Rename old.mars');
    fireEvent.change(input2, { target: { value: '   ' } });
    fireEvent.keyDown(input2, { key: 'Enter' });
    expect(renamePath).not.toHaveBeenCalled();
  });

  it('does not delete when confirmation is declined', async () => {
    openDialog.mockResolvedValue('/workspace');
    listFiles.mockResolvedValue([
      { name: 'old.mars', path: '/workspace/old.mars', isDirectory: false, children: [] },
    ]);
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    render(<FileExplorer currentFilePath={null} onOpenNotebook={vi.fn()} onStatus={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: 'Open folder' }));
    await waitFor(() => expect(screen.getByText('old.mars')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Delete old.mars' }));
    expect(deletePathMock).not.toHaveBeenCalled();
    confirmSpy.mockRestore();
  });
});
