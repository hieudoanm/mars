import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { listCheckpoints, createCheckpoint, readCheckpoint, restoreCheckpoint } = vi.hoisted(() => ({
  listCheckpoints: vi.fn(),
  createCheckpoint: vi.fn(),
  readCheckpoint: vi.fn(),
  restoreCheckpoint: vi.fn(),
}));

vi.mock('../../../lib/kernelService', () => ({
  createNotebookCheckpoint: createCheckpoint,
  listNotebookCheckpoints: listCheckpoints,
  readNotebookCheckpoint: readCheckpoint,
  restoreNotebookCheckpoint: restoreCheckpoint,
}));

import CheckpointsDialog from '../CheckpointsDialog';

const currentContent = JSON.stringify({
  format_version: 1,
  metadata: {},
  cells: [
    {
      id: 'c1',
      cell_type: 'code',
      source: 'new',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
});

const oldContent = JSON.stringify({
  format_version: 1,
  metadata: {},
  cells: [
    {
      id: 'c1',
      cell_type: 'code',
      source: 'old',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
});

const setup = (overrides: Partial<Parameters<typeof CheckpointsDialog>[0]> = {}) => {
  const props = {
    open: true,
    filePath: '/books/test.mars',
    currentContent,
    onRestored: vi.fn(),
    onStatus: vi.fn(),
    onClose: vi.fn(),
    ...overrides,
  };
  render(<CheckpointsDialog {...props} />);
  return props;
};

describe('CheckpointsDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listCheckpoints.mockResolvedValue([]);
    createCheckpoint.mockResolvedValue('cp');
    readCheckpoint.mockResolvedValue(oldContent);
    restoreCheckpoint.mockResolvedValue(undefined);
  });

  it('renders nothing when closed', () => {
    const { container } = render(
      <CheckpointsDialog
        open={false}
        filePath='/books/test.mars'
        currentContent={currentContent}
        onRestored={vi.fn()}
        onStatus={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('prompts to save when there is no file path', async () => {
    setup({ filePath: null });
    expect(screen.getByText(/Save the notebook/)).toBeInTheDocument();
    expect(listCheckpoints).not.toHaveBeenCalled();
  });

  it('lists checkpoints and creates a new one', async () => {
    listCheckpoints.mockResolvedValue(['/books/.mars-checkpoints/checkpoint-1700000000000.mars']);
    const props = setup();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Diff' })).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Create checkpoint now' }));
    await waitFor(() => expect(createCheckpoint).toHaveBeenCalledWith('/books/test.mars'));
    expect(props.onStatus).toHaveBeenCalledWith('Checkpoint created');
    expect(listCheckpoints).toHaveBeenCalledTimes(2);
  });

  it('shows an empty state when no checkpoints exist', async () => {
    setup();
    await waitFor(() => expect(screen.getByText('No checkpoints yet.')).toBeInTheDocument());
  });

  it('restores a checkpoint and closes', async () => {
    listCheckpoints.mockResolvedValue(['/books/cp1.mars']);
    const props = setup();
    await waitFor(() => screen.getByText('cp1.mars'));
    fireEvent.click(screen.getByRole('button', { name: 'Restore' }));
    await waitFor(() =>
      expect(restoreCheckpoint).toHaveBeenCalledWith('/books/test.mars', '/books/cp1.mars'),
    );
    expect(props.onStatus).toHaveBeenCalledWith('Checkpoint restored');
    expect(props.onRestored).toHaveBeenCalled();
    expect(props.onClose).toHaveBeenCalled();
  });

  it('shows a diff with changed lines and cell chips', async () => {
    listCheckpoints.mockResolvedValue(['/books/cp1.mars']);
    setup();
    await waitFor(() => screen.getByText('cp1.mars'));
    fireEvent.click(screen.getByRole('button', { name: 'Diff' }));
    await waitFor(() =>
      expect(readCheckpoint).toHaveBeenCalledWith('/books/test.mars', '/books/cp1.mars'),
    );
    await waitFor(() => expect(screen.getByText(/changed line/)).toBeInTheDocument());
    // cell diff chip toggles per-cell view
    const chip = await screen.findByRole('button', { name: /Cell 1/ });
    fireEvent.click(chip);
    expect(screen.getByText('Show full notebook diff')).toBeInTheDocument();
    fireEvent.click(screen.getByText('Show full notebook diff'));
    expect(screen.queryByText('Show full notebook diff')).toBeNull();
  });

  it('reports create errors through onStatus', async () => {
    const props = setup();
    createCheckpoint.mockRejectedValue(new Error('create failed'));
    fireEvent.click(screen.getByRole('button', { name: 'Create checkpoint now' }));
    await waitFor(() => expect(props.onStatus).toHaveBeenCalledWith('create failed'));
  });

  it('reports diff errors through onStatus', async () => {
    listCheckpoints.mockResolvedValue(['/books/cp1.mars']);
    readCheckpoint.mockRejectedValue(new Error('diff failed'));
    const props = setup();
    await waitFor(() => screen.getByText('cp1.mars'));
    fireEvent.click(screen.getByRole('button', { name: 'Diff' }));
    await waitFor(() => expect(props.onStatus).toHaveBeenCalledWith('diff failed'));
  });

  it('closes via the close button', () => {
    const props = setup();
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(props.onClose).toHaveBeenCalled();
  });
});
