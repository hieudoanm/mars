import '@testing-library/jest-dom/vitest';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';

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
vi.mock('../lib/initialNotebook', () => ({
  createInitialNotebook: () => ({
    format_version: 2,
    metadata: { language: 'rust', title: 'Untitled notebook' },
    cells: [
      {
        id: 'cell-md',
        cell_type: 'markdown',
        source: '# Welcome',
        execution_count: null,
        outputs: [],
        metadata: {},
      },
      {
        id: 'cell-code',
        cell_type: 'code',
        source: 'println!("Hello from Mars!");',
        execution_count: null,
        outputs: [],
        metadata: {},
      },
    ],
  }),
}));
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
  kernelStatus: vi.fn(async () => ({ type: 'status', status: 'idle', execution_count: 0 })),
  inspectVariables: vi.fn(async () => []),
  diagnoseCell: vi.fn(async () => []),
  sendKernelInput: vi.fn(async () => {}),
  lspSync: vi.fn(async () => {}),
  lspHover: vi.fn(async () => null),
  lspDiagnostics: vi.fn(async () => []),
  openInTerminal: vi.fn(async () => {}),
  createNotebookFile: vi.fn(),
  createDirectory: vi.fn(),
  deletePath: vi.fn(),
  writeTextFile: vi.fn(),
  exportHtmlFile: vi.fn(),
  exportPdfFile: vi.fn(),
  createNotebookCheckpoint: vi.fn(async () => '/tmp/checkpoint.mars'),
  listNotebookCheckpoints: vi.fn(async () => []),
  readNotebookCheckpoint: vi.fn(async () => '{}'),
  restoreNotebookCheckpoint: vi.fn(),
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
    window.localStorage.clear();
    const untitledId = crypto.randomUUID();
    window.localStorage.setItem(
      'mars.workspace.tabs',
      JSON.stringify({
        version: 2,
        entries: [{ key: `untitled:${untitledId}`, path: null, untitledId }],
        activeKey: `untitled:${untitledId}`,
      }),
    );
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
    expect(screen.getAllByText('experiment.mars')).toHaveLength(3);

    fireEvent.click(screen.getByRole('button', { name: /Run cell/ }));
    await screen.findByText('42');
    expect(executeSource).toHaveBeenCalledWith(expect.any(String), '40 + 2', null, 'rust');

    fireEvent.click(screen.getByRole('button', { name: 'Restart runtime and clear outputs' }));
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

  it('renames the notebook title from the command palette', () => {
    render(<App />);

    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    fireEvent.click(screen.getByText('Rename notebook'));

    const input = screen.getByLabelText('Notebook title');
    fireEvent.change(input, { target: { value: 'Data exploration' } });
    fireEvent.click(screen.getByRole('button', { name: 'Rename' }));

    expect(screen.queryByLabelText('Notebook title')).not.toBeInTheDocument();
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

  it('shows the welcome view for stale tab state from older builds', () => {
    window.localStorage.setItem(
      'mars.workspace.tabs',
      JSON.stringify({ entries: [{ key: 'untitled:old', path: null, untitledId: 'old' }] }),
    );
    window.localStorage.setItem('mars.workspace.lastPath', '/notes/old.mars');
    render(<App />);

    expect(
      screen.getByText('Explore ideas. Execute code. Discover possibilities.'),
    ).toBeInTheDocument();
    expect(screen.queryByText('2 cells')).not.toBeInTheDocument();
  });

  it('shows the welcome view when no notebooks are open and starts one', () => {
    window.localStorage.clear();
    const { container } = render(<App />);

    const welcome = container.querySelector('[data-component="WelcomeView"]') as HTMLElement;
    expect(
      within(welcome).getByText('Explore ideas. Execute code. Discover possibilities.'),
    ).toBeVisible();
    expect(screen.queryByText('2 cells')).not.toBeInTheDocument();

    fireEvent.click(within(welcome).getByRole('button', { name: 'New notebook' }));
    expect(screen.getByText('2 cells')).toBeInTheDocument();
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
