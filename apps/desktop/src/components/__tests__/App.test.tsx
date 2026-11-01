import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

const { openDialog, saveDialog, saveFile, executeSource, resetKernel } = vi.hoisted(() => ({
  openDialog: vi.fn(),
  saveDialog: vi.fn(),
  saveFile: vi.fn(),
  executeSource: vi.fn(),
  resetKernel: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openDialog, save: saveDialog }));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ onCloseRequested: vi.fn(), close: vi.fn() }),
}));
vi.mock('../../lib/kernelService', () => ({
  configureKernel: vi.fn(),
  exportMarkdownFile: vi.fn(),
  loadNotebookFile: vi.fn(),
  saveNotebookFile: saveFile,
  executeSource,
  hasInterruptedOutput: () => false,
  resetKernel,
  cancelKernelExecution: vi.fn(),
}));
vi.mock('../CodeEditor', () => ({
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
vi.mock('../OutputView', () => ({
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

    expect(screen.getByRole('heading', { name: 'Explore ideas.' })).toBeInTheDocument();
    expect(screen.getByText('Untitled.mars')).toBeInTheDocument();
    expect(screen.getByText('2 cells')).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText('Write Rust code...'), {
      target: { value: '40 + 2' },
    });
    expect(screen.getAllByText('Unsaved changes')).toHaveLength(2);
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() =>
      expect(saveFile).toHaveBeenCalledWith('/notes/experiment.mars', expect.any(String)),
    );
    expect(screen.getByText('experiment.mars')).toBeInTheDocument();

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

  it('prompts before opening a notebook when edits are unsaved', async () => {
    openDialog.mockResolvedValue('/notes/other.mars');
    render(<App />);
    fireEvent.change(screen.getByLabelText('Write Rust code...'), {
      target: { value: 'changed' },
    });

    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Save changes before opening?')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});
