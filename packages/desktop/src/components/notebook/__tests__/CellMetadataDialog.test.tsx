import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import CellMetadataDialog from '../CellMetadataDialog';
import type { NotebookCell } from '../../../types/types';

const cell: NotebookCell = {
  id: 'c1',
  cell_type: 'code',
  source: 'let x = 1;',
  execution_count: null,
  outputs: [],
  metadata: { tags: ['a'], custom: 1 },
};

describe('CellMetadataDialog', () => {
  it('saves parsed JSON metadata', () => {
    const onSave = vi.fn();
    const onClose = vi.fn();
    render(<CellMetadataDialog cell={cell} onClose={onClose} onSave={onSave} />);
    const area = screen.getByLabelText('Cell metadata JSON');
    expect(area).toHaveValue(JSON.stringify(cell.metadata, null, 2));
    fireEvent.change(area, { target: { value: '{"tags":["b"]}' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save metadata' }));
    expect(onSave).toHaveBeenCalledWith({ tags: ['b'] });
    expect(onClose).toHaveBeenCalled();
  });

  it('rejects invalid JSON and non-objects', () => {
    const onSave = vi.fn();
    render(<CellMetadataDialog cell={cell} onClose={vi.fn()} onSave={onSave} />);
    const area = screen.getByLabelText('Cell metadata JSON');
    fireEvent.change(area, { target: { value: '{bad' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save metadata' }));
    expect(screen.getByText('Invalid JSON')).toBeInTheDocument();
    fireEvent.change(area, { target: { value: '[1,2]' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save metadata' }));
    expect(screen.getByText('Metadata must be a JSON object')).toBeInTheDocument();
    expect(onSave).not.toHaveBeenCalled();
  });
});
