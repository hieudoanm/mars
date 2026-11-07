import '@testing-library/jest-dom/vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';

const { openDialog, saveDialog, saveFile, renameFile, executeSource, resetKernel } = vi.hoisted(
  () => ({
    openDialog: vi.fn(),
    saveDialog: vi.fn(),
    saveFile: vi.fn(),
    renameFile: vi.fn(),
    executeSource: vi.fn(),
    resetKernel: vi.fn(),
  }),
);

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openDialog, save: saveDialog }));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ onCloseRequested: vi.fn(), close: vi.fn() }),
}));
vi.mock('../lib/kernelService', () => ({
  configureKernel: vi.fn(),
  exportIpynbFile: vi.fn(),
  exportMarkdownFile: vi.fn(),
  importNotebookIpynb: vi.fn(),
  listenKernelStream: vi.fn(async () => () => {}),
  loadNotebookFile: vi.fn(),
  notebookFileMtime: vi.fn(async () => null),
  saveNotebookFile: saveFile,
  renameNotebookFile: renameFile,
  listNotebookFiles: vi.fn(),
  executeSource,
  hasInterruptedOutput: () => false,
  resetKernel,
  cancelKernelExecution: vi.fn(),
}));
vi.mock('../components/editor/CodeEditor', () => ({
  default: ({
    value,
    onChange,
    onFocus,
    placeholder,
  }: {
    value: string;
    onChange: (value: string) => void;
    onFocus: () => void;
    placeholder: string;
  }) => (
    <textarea
      aria-label={placeholder}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      onFocus={onFocus}
    />
  ),
}));
vi.mock('../components/output/OutputView', () => ({
  default: ({ output }: { output: { text?: string } }) => (
    <div>{output.text ?? 'Rendered output'}</div>
  ),
}));

import App from '../App';

describe('App', () => {
  beforeEach(() => {
    openDialog.mockReset();
    saveDialog.mockReset();
    saveFile.mockReset().mockResolvedValue(undefined);
    renameFile.mockReset().mockResolvedValue(undefined);
    executeSource.mockReset().mockResolvedValue({
      type: 'execution',
      outputs: [{ type: 'text', text: '42' }],
      execution_count: 1,
      duration_millis: 3,
    });
    resetKernel.mockReset().mockResolvedValue(undefined);
  });

  it('supports editing, saving, execution, clearing outputs, and runtime restart', async () => {
    saveDialog.mockResolvedValue('/notes/experiment.mars');
    render(<App />);

    expect(screen.getByRole('heading', { name: 'Mars' })).toBeInTheDocument();
    expect(screen.getAllByText('Untitled.mars')).toHaveLength(2);
    expect(screen.getByText('2 cells')).toBeInTheDocument();

    fireEvent.change(await screen.findByLabelText('Write Rust code...'), {
      target: { value: '40 + 2' },
    });
    expect(screen.getAllByText('Unsaved changes')).toHaveLength(2);
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() =>
      expect(saveFile).toHaveBeenCalledWith('/notes/experiment.mars', expect.any(String)),
    );
    expect(screen.getAllByText('experiment.mars')).toHaveLength(2);

    fireEvent.click(screen.getByRole('button', { name: /Run cell/ }));
    await screen.findByText('42');
    expect(executeSource).toHaveBeenCalledWith('40 + 2', null);

    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime' }));
    await waitFor(() => expect(resetKernel).toHaveBeenCalledOnce());
    fireEvent.click(screen.getByRole('button', { name: /Run all/ }));
    await waitFor(() =>
      expect(screen.getByText('Executed 1 code cells · Edit mode')).toBeInTheDocument(),
    );
  });

  it('renames the saved notebook file from the sidebar', async () => {
    saveDialog.mockResolvedValue('/notes/experiment.mars');
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(saveFile).toHaveBeenCalledOnce());

    fireEvent.click(screen.getByRole('button', { name: 'Rename file' }));
    const input = screen.getByLabelText('File name');
    fireEvent.change(input, { target: { value: 'analysis' } });
    fireEvent.blur(input);

    await waitFor(() =>
      expect(renameFile).toHaveBeenCalledWith('/notes/experiment.mars', '/notes/analysis.mars'),
    );
    expect(screen.getAllByText('analysis.mars')).toHaveLength(2);
  });

  it('renames the notebook title from the toolbar heading', () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: 'Rename notebook' }));
    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Data exploration' } });
    fireEvent.blur(input);

    expect(screen.getByRole('heading', { level: 2, name: 'Data exploration' })).toBeInTheDocument();
    expect(screen.getAllByText('Unsaved changes').length).toBeGreaterThan(0);
  });

  it('deletes a cell from its header and disables deleting the last cell', () => {
    render(<App />);

    const deleteButtons = screen.getAllByRole('button', { name: 'Delete cell (D, D)' });
    expect(deleteButtons).toHaveLength(2);

    fireEvent.click(deleteButtons[0]);

    expect(screen.getByText('1 cells')).toBeInTheDocument();
    expect(screen.getAllByRole('button', { name: 'Delete cell (D, D)' })[0]).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Delete selected cell (D, D)' })).toBeDisabled();
  });

  it('deletes the selected cell from the toolbar', async () => {
    render(<App />);

    fireEvent.click(screen.getByRole('button', { name: 'Delete selected cell (D, D)' }));

    expect(screen.getByText('1 cells')).toBeInTheDocument();
    expect(await screen.findByLabelText('Write Markdown...')).toBeInTheDocument();
    expect(screen.queryByLabelText('Write Rust code...')).not.toBeInTheDocument();
  });

  it('prompts before opening a notebook when edits are unsaved', async () => {
    openDialog.mockResolvedValue('/notes/other.mars');
    render(<App />);
    fireEvent.change(await screen.findByLabelText('Write Rust code...'), {
      target: { value: 'changed' },
    });

    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Save changes before opening?')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});
