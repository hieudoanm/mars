import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import LabDatasetPreviewDialog from '../LabDatasetPreviewDialog';
import type { LabDataset } from '../../../lib/kernelService';

const dataset = (overrides: Partial<LabDataset> = {}): LabDataset => ({
  id: 'ds-1',
  originalFilename: 'data.csv',
  sha256: 'abc',
  byteSize: 100,
  storageKey: 'key',
  columns: [
    { name: 'a', inferredDtype: 'f64', nullCount: 0, sampleValues: [] },
    { name: 'b', inferredDtype: 'i64', nullCount: 0, sampleValues: [] },
  ],
  rowCount: 1500,
  createdAt: '2026-01-01T00:00:00Z',
  ...overrides,
});

describe('LabDatasetPreviewDialog', () => {
  it('renders filename, shape, and preview table', () => {
    render(
      <LabDatasetPreviewDialog
        dataset={dataset()}
        preview={{
          columns: ['a', 'b'],
          rows: [
            ['1', '2'],
            ['3', null],
          ],
        }}
        onClose={() => {}}
      />,
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('data.csv')).toBeInTheDocument();
    expect(screen.getByText(/1,500 rows · 2 columns/)).toBeInTheDocument();
    expect(screen.getByText('a')).toBeInTheDocument();
    expect(screen.getByText('null')).toBeInTheDocument();
    expect(screen.getByText(/Showing first 2 rows/)).toBeInTheDocument();
  });

  it('shows a loading state without preview', () => {
    render(<LabDatasetPreviewDialog dataset={dataset()} preview={null} onClose={() => {}} />);
    expect(screen.getByText('Loading preview…')).toBeInTheDocument();
  });

  it('renders warnings with column prefixes', () => {
    render(
      <LabDatasetPreviewDialog
        dataset={dataset()}
        preview={null}
        warnings={[{ column: 'a', message: 'sparse column' }, { message: 'general warning' }]}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText('a: sparse column')).toBeInTheDocument();
    expect(screen.getByText('general warning')).toBeInTheDocument();
  });

  it('closes via the close button and backdrop click', () => {
    const onClose = vi.fn();
    render(<LabDatasetPreviewDialog dataset={dataset()} preview={null} onClose={onClose} />);
    fireEvent.click(screen.getByLabelText('Close preview'));
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole('presentation'));
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('does not close when clicking inside the dialog', () => {
    const onClose = vi.fn();
    render(<LabDatasetPreviewDialog dataset={dataset()} preview={null} onClose={onClose} />);
    fireEvent.click(screen.getByRole('dialog'));
    expect(onClose).not.toHaveBeenCalled();
  });
});
