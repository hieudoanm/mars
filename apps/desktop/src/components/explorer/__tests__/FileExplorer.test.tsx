import '@testing-library/jest-dom/vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';

const { openDialog, listFiles } = vi.hoisted(() => ({ openDialog: vi.fn(), listFiles: vi.fn() }));

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openDialog }));
vi.mock('../../../lib/kernelService', () => ({ listNotebookFiles: listFiles }));

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
    fireEvent.click(screen.getByRole('button', { name: /analysis\.mars/ }));
    expect(onOpenNotebook).toHaveBeenCalledWith('/workspace/examples/analysis.mars');
  });
});
