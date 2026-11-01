import '@testing-library/jest-dom/vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';

const { openDialog, listFiles, openExternal } = vi.hoisted(() => ({
  openDialog: vi.fn(),
  listFiles: vi.fn(),
  openExternal: vi.fn(async () => {}),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openDialog }));
vi.mock('../../../lib/kernelService', () => ({
  listNotebookFiles: listFiles,
  openPathExternal: openExternal,
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
});
